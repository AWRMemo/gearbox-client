use crate::db::open_db;
pub use crate::types::{StreamHighlight, StreamInfo};
use uuid::Uuid;

fn ensure_tables(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS streams (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT DEFAULT '',
            is_public INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS stream_highlights (
            stream_id TEXT NOT NULL REFERENCES streams(id) ON DELETE CASCADE,
            highlight_id TEXT NOT NULL REFERENCES highlights(id) ON DELETE CASCADE,
            added_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (stream_id, highlight_id)
        );",
    )
    .map_err(|e| format!("Failed to create stream tables: {e}"))?;
    Ok(())
}

pub fn create_stream(user_id: &str, title: &str, description: &str) -> Result<StreamInfo, String> {
    let conn = open_db()?;
    ensure_tables(&conn)?;

    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO streams (id, user_id, title, description) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, user_id, title, description],
    )
    .map_err(|e| format!("Failed to create stream: {e}"))?;

    get_stream_by_id_internal(&conn, &id)
}

pub fn add_highlight_to_stream(stream_id: &str, highlight_id: &str) -> Result<(), String> {
    let conn = open_db()?;
    ensure_tables(&conn)?;
    add_highlight_to_stream_internal(&conn, stream_id, highlight_id)
}

fn add_highlight_to_stream_internal(
    conn: &rusqlite::Connection,
    stream_id: &str,
    highlight_id: &str,
) -> Result<(), String> {
    let exists: bool = conn
        .query_row(
            "SELECT 1 FROM highlights WHERE id = ?1",
            rusqlite::params![highlight_id],
            |_| Ok(true),
        )
        .unwrap_or(false);
    if !exists {
        return Err(format!("Highlight {} not found", highlight_id));
    }

    let already_in_stream: bool = conn
        .query_row(
            "SELECT 1 FROM stream_highlights WHERE stream_id = ?1 AND highlight_id = ?2",
            rusqlite::params![stream_id, highlight_id],
            |_| Ok(true),
        )
        .unwrap_or(false);

    if !already_in_stream {
        conn.execute(
            "INSERT INTO stream_highlights (stream_id, highlight_id) VALUES (?1, ?2)",
            rusqlite::params![stream_id, highlight_id],
        )
        .map_err(|e| format!("Failed to add highlight to stream: {e}"))?;
    }

    Ok(())
}

pub fn remove_highlight_from_stream(stream_id: &str, highlight_id: &str) -> Result<(), String> {
    let conn = open_db()?;
    remove_highlight_from_stream_internal(&conn, stream_id, highlight_id)
}

fn remove_highlight_from_stream_internal(
    conn: &rusqlite::Connection,
    stream_id: &str,
    highlight_id: &str,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM stream_highlights WHERE stream_id = ?1 AND highlight_id = ?2",
        rusqlite::params![stream_id, highlight_id],
    )
    .map_err(|e| format!("Failed to remove highlight from stream: {e}"))?;

    Ok(())
}

pub fn delete_stream(stream_id: &str) -> Result<(), String> {
    let conn = open_db()?;
    ensure_tables(&conn)?;
    conn.execute(
        "DELETE FROM streams WHERE id = ?1",
        rusqlite::params![stream_id],
    )
    .map_err(|e| format!("Failed to delete stream: {e}"))?;
    Ok(())
}

pub fn get_stream_highlights(stream_id: &str) -> Result<Vec<StreamHighlight>, String> {
    let conn = open_db()?;
    ensure_tables(&conn)?;
    get_stream_highlights_internal(&conn, stream_id)
}

fn get_stream_highlights_internal(
    conn: &rusqlite::Connection,
    stream_id: &str,
) -> Result<Vec<StreamHighlight>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT h.id, h.text, h.summary, h.tags, h.source_url
             FROM stream_highlights sh
             JOIN highlights h ON h.id = sh.highlight_id
             WHERE sh.stream_id = ?1
             ORDER BY sh.added_at DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![stream_id], |row| {
            let tags_str: String = row.get(3)?;
            let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
            Ok(StreamHighlight {
                id: row.get(0)?,
                text: row.get(1)?,
                summary: row.get(2)?,
                tags,
                source_url: row.get(4)?,
            })
        })
        .map_err(|e| format!("Failed to query stream highlights: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Row error: {e}"))?);
    }
    Ok(results)
}

pub fn list_user_streams(user_id: &str) -> Result<Vec<StreamInfo>, String> {
    let conn = open_db()?;
    ensure_tables(&conn)?;
    list_user_streams_internal(&conn, user_id)
}

fn list_user_streams_internal(
    conn: &rusqlite::Connection,
    user_id: &str,
) -> Result<Vec<StreamInfo>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, user_id, title, description, is_public, created_at, updated_at
             FROM streams
             WHERE user_id = ?1
             ORDER BY updated_at DESC",
        )
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![user_id], |row| {
            Ok(StreamInfo {
                id: row.get(0)?,
                user_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                is_public: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to query streams: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Row error: {e}"))?);
    }
    Ok(results)
}

pub fn get_stream_by_id(stream_id: &str) -> Result<StreamInfo, String> {
    let conn = open_db()?;
    ensure_tables(&conn)?;
    get_stream_by_id_internal(&conn, stream_id)
}

fn get_stream_by_id_internal(
    conn: &rusqlite::Connection,
    stream_id: &str,
) -> Result<StreamInfo, String> {
    conn.query_row(
        "SELECT id, user_id, title, description, is_public, created_at, updated_at
         FROM streams WHERE id = ?1",
        rusqlite::params![stream_id],
        |row| {
            Ok(StreamInfo {
                id: row.get(0)?,
                user_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                is_public: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    )
    .map_err(|e| format!("Stream not found: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db;

    fn setup_full_test_db() -> rusqlite::Connection {
        let conn = open_test_db().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS streams (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT DEFAULT '',
                is_public INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS stream_highlights (
                stream_id TEXT NOT NULL REFERENCES streams(id) ON DELETE CASCADE,
                highlight_id TEXT NOT NULL REFERENCES highlights(id) ON DELETE CASCADE,
                added_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (stream_id, highlight_id)
            );",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_create_and_get_stream() {
        let conn = setup_full_test_db();
        let user_id = "test-user";
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO streams (id, user_id, title, description) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id, user_id, "Test Stream", "A test stream"],
        )
        .unwrap();

        let stream = get_stream_by_id_internal(&conn, &id).unwrap();
        assert_eq!(stream.title, "Test Stream");
        assert_eq!(stream.description, "A test stream");
        assert_eq!(stream.user_id, user_id);
        assert!(stream.is_public);
    }

    #[test]
    fn test_add_and_get_highlights_in_stream() {
        let conn = setup_full_test_db();
        conn.execute(
            "INSERT INTO highlights (id, text, summary, tags) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["h-1", "Some text", "A summary", "[\"tag1\"]"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO highlights_fts (id, text, summary, tags_text) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["h-1", "Some text", "A summary", "tag1"],
        )
        .unwrap();

        let stream_id = "s-1";
        conn.execute(
            "INSERT INTO streams (id, user_id, title) VALUES (?1, ?2, ?3)",
            rusqlite::params![stream_id, "user", "Test"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO stream_highlights (stream_id, highlight_id) VALUES (?1, ?2)",
            rusqlite::params![stream_id, "h-1"],
        )
        .unwrap();

        let highlights = get_stream_highlights_internal(&conn, stream_id).unwrap();
        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].id, "h-1");
        assert_eq!(highlights[0].summary, "A summary");
    }

    #[test]
    fn test_remove_highlight_from_stream() {
        let conn = setup_full_test_db();
        conn.execute(
            "INSERT INTO highlights (id, text, summary, tags) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["h-2", "Text", "Summary", "[]"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO highlights_fts (id, text, summary, tags_text) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["h-2", "Text", "Summary", ""],
        )
        .unwrap();

        let stream_id = "s-2";
        conn.execute(
            "INSERT INTO streams (id, user_id, title) VALUES (?1, ?2, ?3)",
            rusqlite::params![stream_id, "user", "Test"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO stream_highlights (stream_id, highlight_id) VALUES (?1, ?2)",
            rusqlite::params![stream_id, "h-2"],
        )
        .unwrap();

        remove_highlight_from_stream_internal(&conn, stream_id, "h-2").unwrap();

        let highlights = get_stream_highlights_internal(&conn, stream_id).unwrap();
        assert_eq!(highlights.len(), 0);
    }

    #[test]
    fn test_list_user_streams() {
        let conn = setup_full_test_db();
        conn.execute(
            "INSERT INTO streams (id, user_id, title) VALUES (?1, ?2, ?3)",
            rusqlite::params!["s-a", "u1", "Stream A"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO streams (id, user_id, title) VALUES (?1, ?2, ?3)",
            rusqlite::params!["s-b", "u1", "Stream B"],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO streams (id, user_id, title) VALUES (?1, ?2, ?3)",
            rusqlite::params!["s-c", "u2", "Stream C"],
        )
        .unwrap();

        let streams = list_user_streams_internal(&conn, "u1").unwrap();
        assert_eq!(streams.len(), 2);
        assert!(streams.iter().any(|s| s.title == "Stream A"));
        assert!(streams.iter().any(|s| s.title == "Stream B"));
    }
}
