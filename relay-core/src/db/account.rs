use crate::db::open_db;
pub use crate::types::UserProfile;

fn ensure_table(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS user_profile (
            user_id TEXT PRIMARY KEY,
            email TEXT,
            display_name TEXT,
            tier TEXT NOT NULL DEFAULT 'free',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .map_err(|e| format!("Failed to create user_profile table: {e}"))?;
    Ok(())
}

pub fn get_profile(user_id: &str) -> Result<UserProfile, String> {
    let conn = open_db()?;
    ensure_table(&conn)?;

    let result = conn.query_row(
        "SELECT user_id, email, display_name, tier, created_at FROM user_profile WHERE user_id = ?1",
        rusqlite::params![user_id],
        |row| {
            Ok(UserProfile {
                user_id: row.get(0)?,
                email: row.get(1)?,
                display_name: row.get(2)?,
                tier: row.get(3)?,
                created_at: row.get(4)?,
            })
        },
    );

    match result {
        Ok(profile) => Ok(profile),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let conn = open_db()?;
            ensure_table(&conn)?;
            conn.execute(
                "INSERT INTO user_profile (user_id) VALUES (?1)",
                rusqlite::params![user_id],
            )
            .map_err(|e| format!("Failed to create profile: {e}"))?;
            get_profile(user_id)
        }
        Err(e) => Err(format!("Failed to get profile: {e}")),
    }
}

pub fn set_email(user_id: &str, email: &str) -> Result<(), String> {
    let conn = open_db()?;
    ensure_table(&conn)?;

    conn.execute(
        "INSERT INTO user_profile (user_id, email) VALUES (?1, ?2)
         ON CONFLICT(user_id) DO UPDATE SET email = excluded.email",
        rusqlite::params![user_id, email],
    )
    .map_err(|e| format!("Failed to set email: {e}"))?;

    Ok(())
}

pub fn is_pro(user_id: &str) -> Result<bool, String> {
    let profile = get_profile(user_id)?;
    Ok(profile.tier == "pro")
}

pub fn set_tier(user_id: &str, tier: &str) -> Result<(), String> {
    let conn = open_db()?;
    ensure_table(&conn)?;

    conn.execute(
        "INSERT INTO user_profile (user_id, tier) VALUES (?1, ?2)
         ON CONFLICT(user_id) DO UPDATE SET tier = excluded.tier",
        rusqlite::params![user_id, tier],
    )
    .map_err(|e| format!("Failed to set tier: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db;

    fn setup() -> rusqlite::Connection {
        let conn = open_test_db().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS user_profile (
                user_id TEXT PRIMARY KEY,
                email TEXT,
                display_name TEXT,
                tier TEXT NOT NULL DEFAULT 'free',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_set_and_get_profile() {
        let conn = setup();
        conn.execute(
            "INSERT INTO user_profile (user_id, email, tier) VALUES (?1, ?2, ?3)",
            rusqlite::params!["u1", "test@example.com", "free"],
        )
        .unwrap();

        let profile: UserProfile = conn
            .query_row(
                "SELECT user_id, email, display_name, tier, created_at FROM user_profile WHERE user_id = ?1",
                rusqlite::params!["u1"],
                |row| {
                    Ok(UserProfile {
                        user_id: row.get(0)?,
                        email: row.get(1)?,
                        display_name: row.get(2)?,
                        tier: row.get(3)?,
                        created_at: row.get(4)?,
                    })
                },
            )
            .unwrap();

        assert_eq!(profile.email, Some("test@example.com".to_string()));
        assert_eq!(profile.tier, "free");
    }

    #[test]
    fn test_default_tier_is_free() {
        let conn = setup();
        conn.execute(
            "INSERT INTO user_profile (user_id) VALUES (?1)",
            rusqlite::params!["u2"],
        )
        .unwrap();

        let tier: String = conn
            .query_row(
                "SELECT tier FROM user_profile WHERE user_id = ?1",
                rusqlite::params!["u2"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tier, "free");
    }

    #[test]
    fn test_set_tier_to_pro() {
        let conn = setup();
        conn.execute(
            "INSERT INTO user_profile (user_id) VALUES (?1)",
            rusqlite::params!["u3"],
        )
        .unwrap();
        conn.execute(
            "UPDATE user_profile SET tier = ?1 WHERE user_id = ?2",
            rusqlite::params!["pro", "u3"],
        )
        .unwrap();

        let tier: String = conn
            .query_row(
                "SELECT tier FROM user_profile WHERE user_id = ?1",
                rusqlite::params!["u3"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tier, "pro");
    }
}
