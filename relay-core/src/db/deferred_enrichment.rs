use crate::db::open_db;

#[derive(Debug, Clone)]
pub struct DeferredItem {
    pub highlight_id: String,
    pub text: String,
    pub source_url: Option<String>,
    pub source_title: Option<String>,
    pub source_author: Option<String>,
}

/// Ensure the deferred_enrichment table exists.
pub fn ensure_table(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS deferred_enrichment (
            highlight_id TEXT PRIMARY KEY,
            text TEXT NOT NULL,
            source_url TEXT,
            source_title TEXT,
            source_author TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (highlight_id) REFERENCES highlights(id) ON DELETE CASCADE
        );",
    )
    .map_err(|e| format!("deferred_enrichment table: {e}"))
}

/// Queue a highlight for deferred enrichment (stored raw, enriched later).
pub fn queue_deferred(
    highlight_id: &str,
    text: &str,
    source_url: Option<&str>,
    source_title: Option<&str>,
    source_author: Option<&str>,
) -> Result<(), String> {
    let conn = open_db()?;
    ensure_table(&conn)?;
    conn.execute(
        "INSERT OR IGNORE INTO deferred_enrichment (highlight_id, text, source_url, source_title, source_author)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![highlight_id, text, source_url, source_title, source_author],
    )
    .map_err(|e| format!("Failed to queue deferred enrichment: {e}"))?;
    Ok(())
}

/// Return all queued deferred enrichments.
pub fn list_deferred() -> Result<Vec<DeferredItem>, String> {
    let conn = open_db()?;
    ensure_table(&conn)?;
    let mut stmt = conn
        .prepare("SELECT highlight_id, text, source_url, source_title, source_author FROM deferred_enrichment ORDER BY created_at ASC")
        .map_err(|e| format!("Failed to prepare deferred list: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(DeferredItem {
                highlight_id: row.get(0)?,
                text: row.get(1)?,
                source_url: row.get(2)?,
                source_title: row.get(3)?,
                source_author: row.get(4)?,
            })
        })
        .map_err(|e| format!("Failed to query deferred list: {e}"))?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| format!("Row read error: {e}"))?);
    }
    Ok(items)
}

/// Remove a deferred enrichment entry after processing.
pub fn remove_deferred(highlight_id: &str) -> Result<(), String> {
    let conn = open_db()?;
    conn.execute(
        "DELETE FROM deferred_enrichment WHERE highlight_id = ?1",
        rusqlite::params![highlight_id],
    )
    .map_err(|e| format!("Failed to remove deferred enrichment: {e}"))?;
    Ok(())
}

/// Count pending deferred enrichments.
pub fn count_deferred() -> Result<usize, String> {
    let conn = open_db()?;
    ensure_table(&conn)?;
    conn.query_row(
        "SELECT COUNT(*) FROM deferred_enrichment",
        [],
        |row| row.get::<_, usize>(0),
    )
    .map_err(|e| format!("Failed to count deferred: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db;

    #[test]
    fn test_queue_and_list_deferred() {
        let conn = open_test_db().unwrap();
        ensure_table(&conn).unwrap();

        queue_deferred("hl1", "text 1", Some("https://example.com"), None, None).unwrap();
        queue_deferred("hl2", "text 2", None, Some("Title"), Some("Author")).unwrap();

        let items = list_deferred().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].highlight_id, "hl1");
        assert_eq!(items[1].text, "text 2");

        remove_deferred("hl1").unwrap();
        assert_eq!(count_deferred().unwrap(), 1);
    }
}
