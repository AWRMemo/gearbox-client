use crate::db::open_db;
use crate::types::{CreatorAnalytics, CreatorProfile, MonetizedStream};

pub fn ensure_table(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS creator_profiles (
            user_id TEXT PRIMARY KEY,
            stripe_connect_account_id TEXT,
            is_verified INTEGER NOT NULL DEFAULT 0,
            display_name TEXT,
            bio TEXT,
            platform_fee_percent INTEGER NOT NULL DEFAULT 10,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            last_modified TEXT NOT NULL DEFAULT (datetime('now')),
            sync_status TEXT NOT NULL DEFAULT 'local'
        );

        CREATE TABLE IF NOT EXISTS monetized_streams (
            stream_id TEXT PRIMARY KEY REFERENCES streams(id),
            creator_id TEXT NOT NULL,
            monthly_price_cents INTEGER NOT NULL DEFAULT 200,
            subscriber_count INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            last_modified TEXT NOT NULL DEFAULT (datetime('now')),
            sync_status TEXT NOT NULL DEFAULT 'local'
        );

        CREATE INDEX IF NOT EXISTS idx_monetized_creator ON monetized_streams(creator_id);",
    )
    .map_err(|e| format!("creator tables: {e}"))
}

pub fn get_profile(user_id: &str) -> Result<Option<CreatorProfile>, String> {
    let conn = open_db()?;
    ensure_table(&conn)?;
    let result = conn.query_row(
        "SELECT user_id, stripe_connect_account_id, is_verified, display_name, bio, platform_fee_percent, created_at
         FROM creator_profiles WHERE user_id = ?1",
        rusqlite::params![user_id],
        |row| {
            Ok(CreatorProfile {
                user_id: row.get(0)?,
                stripe_connect_account_id: row.get(1)?,
                is_verified: row.get::<_, i64>(2)? != 0,
                display_name: row.get(3)?,
                bio: row.get(4)?,
                platform_fee_percent: row.get(5)?,
                created_at: row.get(6)?,
            })
        },
    );
    match result {
        Ok(profile) => Ok(Some(profile)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("get creator profile: {e}")),
    }
}

pub fn upsert_profile(user_id: &str, stripe_connect_id: &str) -> Result<(), String> {
    let conn = open_db()?;
    ensure_table(&conn)?;
    conn.execute(
        "INSERT INTO creator_profiles (user_id, stripe_connect_account_id) VALUES (?1, ?2)
         ON CONFLICT(user_id) DO UPDATE SET stripe_connect_account_id = excluded.stripe_connect_account_id",
        rusqlite::params![user_id, stripe_connect_id],
    )
    .map(|_| ())
    .map_err(|e| format!("upsert creator profile: {e}"))
}

pub fn verify_creator(user_id: &str) -> Result<(), String> {
    let conn = open_db()?;
    ensure_table(&conn)?;
    let rows = conn.execute(
        "UPDATE creator_profiles SET is_verified = 1 WHERE user_id = ?1",
        rusqlite::params![user_id],
    )
    .map_err(|e| format!("verify creator: {e}"))?;
    if rows == 0 {
        return Err("creator profile not found".to_string());
    }
    Ok(())
}

pub fn monetize_stream(
    stream_id: &str,
    creator_id: &str,
    price_cents: i64,
) -> Result<(), String> {
    let conn = open_db()?;
    ensure_table(&conn)?;
    if !(200..=5000).contains(&price_cents) {
        return Err("price must be between $2.00 and $50.00".to_string());
    }
    let profile = get_profile(creator_id)?
        .ok_or_else(|| "creator profile not found".to_string())?;
    if !profile.is_verified {
        return Err("only verified creators can monetize streams".to_string());
    }
    conn.execute(
        "INSERT INTO monetized_streams (stream_id, creator_id, monthly_price_cents) VALUES (?1, ?2, ?3)
         ON CONFLICT(stream_id) DO UPDATE SET monthly_price_cents = excluded.monthly_price_cents",
        rusqlite::params![stream_id, creator_id, price_cents],
    )
    .map_err(|e| format!("monetize stream: {e}"))?;
    Ok(())
}

pub fn list_monetized(user_id: &str) -> Result<Vec<MonetizedStream>, String> {
    let conn = open_db()?;
    ensure_table(&conn)?;
    let mut stmt = conn
        .prepare(
            "SELECT stream_id, creator_id, monthly_price_cents, subscriber_count, is_active
             FROM monetized_streams WHERE creator_id = ?1",
        )
        .map_err(|e| format!("list monetized: {e}"))?;
    let rows = stmt
        .query_map(rusqlite::params![user_id], |row| {
            Ok(MonetizedStream {
                stream_id: row.get(0)?,
                creator_id: row.get(1)?,
                monthly_price_cents: row.get(2)?,
                subscriber_count: row.get(3)?,
                is_active: row.get::<_, i64>(4)? != 0,
            })
        })
        .map_err(|e| format!("query monetized: {e}"))?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| format!("row error: {e}"))?);
    }
    Ok(result)
}

pub fn get_analytics(user_id: &str) -> Result<CreatorAnalytics, String> {
    let conn = open_db()?;
    ensure_table(&conn)?;
    let subscriber_count: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(subscriber_count), 0) FROM monetized_streams WHERE creator_id = ?1",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let stream_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM monetized_streams WHERE creator_id = ?1",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let monthly_revenue_cents: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(monthly_price_cents * subscriber_count), 0) FROM monetized_streams WHERE creator_id = ?1 AND is_active = 1",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(CreatorAnalytics {
        subscriber_count,
        monthly_revenue_cents,
        stream_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db;

    #[test]
    fn test_creator_lifecycle() {
        let conn = open_test_db().unwrap();
        ensure_table(&conn).unwrap();

        // Initially no profile
        assert!(get_profile("u1").unwrap().is_none());

        // Register
        upsert_profile("u1", "acct_123").unwrap();
        let p = get_profile("u1").unwrap().unwrap();
        assert!(!p.is_verified);
        assert_eq!(p.stripe_connect_account_id.as_deref(), Some("acct_123"));

        // Verify
        verify_creator("u1").unwrap();
        let p = get_profile("u1").unwrap().unwrap();
        assert!(p.is_verified);

        // Monetize stream
        monetize_stream("s1", "u1", 500).unwrap();

        // Verify price validation
        assert!(monetize_stream("s2", "u1", 100).is_err()); // below $2
        assert!(monetize_stream("s3", "u1", 6000).is_err()); // above $50

        // List monetized
        let streams = list_monetized("u1").unwrap();
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].monthly_price_cents, 500);

        // Unverified user can't monetize
        upsert_profile("u2", "acct_456").unwrap();
        assert!(monetize_stream("s4", "u2", 500).is_err());
        // But a non-existent profile can't monetize either
        assert!(monetize_stream("s5", "nonexistent", 500).is_err());

        // Analytics
        let analytics = get_analytics("u1").unwrap();
        assert_eq!(analytics.monthly_revenue_cents, 0); // zero because subscriber_count = 0
    }
}
