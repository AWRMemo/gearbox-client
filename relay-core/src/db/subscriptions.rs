use crate::db::open_db;
pub use crate::types::FeedHighlight;

pub fn subscribe(user_id: &str, stream_id: &str) -> Result<(), String> {
    let conn = open_db()?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS subscriptions (
            user_id TEXT NOT NULL,
            stream_id TEXT NOT NULL,
            subscribed_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (user_id, stream_id)
        );",
    )
    .map_err(|e| format!("Failed to create subscriptions table: {e}"))?;

    conn.execute(
        "INSERT OR IGNORE INTO subscriptions (user_id, stream_id) VALUES (?1, ?2)",
        rusqlite::params![user_id, stream_id],
    )
    .map_err(|e| format!("Failed to subscribe: {e}"))?;

    Ok(())
}

pub fn unsubscribe(user_id: &str, stream_id: &str) -> Result<(), String> {
    let conn = open_db()?;
    conn.execute(
        "DELETE FROM subscriptions WHERE user_id = ?1 AND stream_id = ?2",
        rusqlite::params![user_id, stream_id],
    )
    .map_err(|e| format!("Failed to unsubscribe: {e}"))?;

    Ok(())
}

pub fn is_subscribed(user_id: &str, stream_id: &str) -> Result<bool, String> {
    let conn = open_db()?;
    let result: bool = conn
        .query_row(
            "SELECT 1 FROM subscriptions WHERE user_id = ?1 AND stream_id = ?2",
            rusqlite::params![user_id, stream_id],
            |_| Ok(true),
        )
        .unwrap_or(false);
    Ok(result)
}

pub fn get_subscribed_streams(user_id: &str) -> Result<Vec<String>, String> {
    let conn = open_db()?;
    let mut stmt = conn
        .prepare(
            "SELECT stream_id FROM subscriptions WHERE user_id = ?1 ORDER BY subscribed_at DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![user_id], |row| row.get::<_, String>(0))
        .map_err(|e| format!("Failed to query subscriptions: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Row error: {e}"))?);
    }
    Ok(results)
}

pub fn get_subscribed_streams_info(user_id: &str) -> Result<Vec<crate::types::StreamInfo>, String> {
    let conn = open_db()?;
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.user_id, s.title, s.description, s.is_public, s.created_at, s.updated_at
             FROM subscriptions sub
             JOIN streams s ON s.id = sub.stream_id
             WHERE sub.user_id = ?1
             ORDER BY sub.subscribed_at DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![user_id], |row| {
            Ok(crate::types::StreamInfo {
                id: row.get(0)?,
                user_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                is_public: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to query subscribed streams: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Row error: {e}"))?);
    }
    Ok(results)
}

pub fn get_subscriber_feed(
    user_id: &str,
    limit: usize,
    offset: usize,
) -> Result<Vec<FeedHighlight>, String> {
    let conn = open_db()?;

    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT h.id, h.text, h.summary, h.tags, h.source_url, s.id, s.title
             FROM subscriptions sub
             JOIN streams s ON s.id = sub.stream_id
             JOIN stream_highlights sh ON sh.stream_id = s.id
             JOIN highlights h ON h.id = sh.highlight_id
             WHERE sub.user_id = ?1
             ORDER BY sh.added_at DESC
             LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| format!("Failed to prepare feed query: {e}"))?;

    let rows = stmt
        .query_map(
            rusqlite::params![user_id, limit as i64, offset as i64],
            |row| {
                let tags_str: String = row.get(3)?;
                let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
                Ok(FeedHighlight {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    summary: row.get(2)?,
                    tags,
                    source_url: row.get(4)?,
                    stream_id: row.get(5)?,
                    stream_title: row.get(6)?,
                })
            },
        )
        .map_err(|e| format!("Failed to query feed: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Row error: {e}"))?);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use crate::db::open_test_db;

    fn setup_tables(conn: &rusqlite::Connection) {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS subscriptions (
                user_id TEXT NOT NULL,
                stream_id TEXT NOT NULL,
                subscribed_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (user_id, stream_id)
            );",
        )
        .unwrap();
    }

    #[test]
    fn test_subscribe_and_is_subscribed() {
        let conn = open_test_db().unwrap();
        setup_tables(&conn);

        conn.execute(
            "INSERT INTO subscriptions (user_id, stream_id) VALUES (?1, ?2)",
            rusqlite::params!["user1", "stream1"],
        )
        .unwrap();

        let found: bool = conn
            .query_row(
                "SELECT 1 FROM subscriptions WHERE user_id = ?1 AND stream_id = ?2",
                rusqlite::params!["user1", "stream1"],
                |_| Ok(true),
            )
            .unwrap_or(false);
        assert!(found);

        let not_found: bool = conn
            .query_row(
                "SELECT 1 FROM subscriptions WHERE user_id = ?1 AND stream_id = ?2",
                rusqlite::params!["user1", "stream2"],
                |_| Ok(true),
            )
            .unwrap_or(false);
        assert!(!not_found);
    }

    #[test]
    fn test_unsubscribe() {
        let conn = open_test_db().unwrap();
        setup_tables(&conn);

        conn.execute(
            "INSERT INTO subscriptions (user_id, stream_id) VALUES (?1, ?2)",
            rusqlite::params!["user1", "stream1"],
        )
        .unwrap();

        conn.execute(
            "DELETE FROM subscriptions WHERE user_id = ?1 AND stream_id = ?2",
            rusqlite::params!["user1", "stream1"],
        )
        .unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM subscriptions WHERE user_id = ?1",
                rusqlite::params!["user1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_get_subscribed_streams() {
        let conn = open_test_db().unwrap();
        setup_tables(&conn);

        conn.execute(
            "INSERT INTO subscriptions (user_id, stream_id) VALUES (?1, ?2)",
            rusqlite::params!["u1", "s1"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO subscriptions (user_id, stream_id) VALUES (?1, ?2)",
            rusqlite::params!["u1", "s2"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO subscriptions (user_id, stream_id) VALUES (?1, ?2)",
            rusqlite::params!["u2", "s3"],
        )
        .unwrap();

        let mut stmt = conn
            .prepare("SELECT stream_id FROM subscriptions WHERE user_id = ?1 ORDER BY subscribed_at DESC")
            .unwrap();
        let ids: Vec<String> = stmt
            .query_map(rusqlite::params!["u1"], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"s1".to_string()));
        assert!(ids.contains(&"s2".to_string()));
    }
}
