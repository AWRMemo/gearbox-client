use crate::db::open_db;
use uuid::Uuid;

pub fn ensure_analytics_table(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS analytics_events (
            id TEXT PRIMARY KEY,
            event_type TEXT NOT NULL,
            stream_id TEXT,
            curator_user_id TEXT,
            visitor_anonymous_id TEXT,
            channel TEXT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_analytics_event_type ON analytics_events(event_type);
        CREATE INDEX IF NOT EXISTS idx_analytics_timestamp ON analytics_events(timestamp);",
    )
    .map_err(|e| format!("Failed to create analytics table: {e}"))?;
    Ok(())
}

pub fn log_event(
    event_type: &str,
    stream_id: Option<&str>,
    curator_user_id: Option<&str>,
    visitor_anonymous_id: Option<&str>,
    channel: Option<&str>,
) -> Result<(), String> {
    let conn = open_db()?;
    ensure_analytics_table(&conn)?;

    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO analytics_events (id, event_type, stream_id, curator_user_id, visitor_anonymous_id, channel)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, event_type, stream_id, curator_user_id, visitor_anonymous_id, channel],
    )
    .map_err(|e| format!("Failed to log analytics event: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db;

    #[test]
    fn test_log_and_retrieve_event() {
        let conn = open_test_db().unwrap();
        ensure_analytics_table(&conn).unwrap();

        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO analytics_events (id, event_type, stream_id, channel)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id, "stream_published", "stream-1", "clipboard"],
        )
        .unwrap();

        let (event_type, stream_id, channel): (String, Option<String>, Option<String>) = conn
            .query_row(
                "SELECT event_type, stream_id, channel FROM analytics_events WHERE id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert_eq!(event_type, "stream_published");
        assert_eq!(stream_id, Some("stream-1".to_string()));
        assert_eq!(channel, Some("clipboard".to_string()));
    }

    #[test]
    fn test_log_multiple_event_types() {
        let conn = open_test_db().unwrap();
        ensure_analytics_table(&conn).unwrap();

        let events = [
            ("stream_published", Some("s1")),
            ("stream_share_link_generated", Some("s1")),
            ("stream_page_view", Some("s1")),
            ("stream_subscribe_click", Some("s1")),
        ];

        for (idx, (event_type, stream_id)) in events.iter().enumerate() {
            let id = format!("e-{}", idx);
            conn.execute(
                "INSERT INTO analytics_events (id, event_type, stream_id) VALUES (?1, ?2, ?3)",
                rusqlite::params![id, event_type, stream_id],
            )
            .unwrap();
        }

        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM analytics_events")
            .unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn test_kfactor_events_all_logged() {
        let conn = open_test_db().unwrap();
        ensure_analytics_table(&conn).unwrap();

        let events = [
            "relay_install_complete",
            "first_highlight_captured",
            "stream_published",
            "stream_page_view",
            "stream_subscribe_click",
        ];

        for (idx, event_type) in events.iter().enumerate() {
            let id = format!("kf-{}", idx);
            conn.execute(
                "INSERT INTO analytics_events (id, event_type) VALUES (?1, ?2)",
                rusqlite::params![id, event_type],
            )
            .unwrap();
        }

        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM analytics_events")
            .unwrap();
        let count: i64 = stmt
            .query_row([], |row: &rusqlite::Row| row.get(0))
            .unwrap();
        assert_eq!(count, 5);

        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM analytics_events WHERE event_type = ?1")
            .unwrap();
        for event_type in events {
            let event_count: i64 = stmt
                .query_row([event_type], |row: &rusqlite::Row| row.get(0))
                .unwrap();
            assert_eq!(event_count, 1, "{event_type} should be logged exactly once");
        }
    }
}
