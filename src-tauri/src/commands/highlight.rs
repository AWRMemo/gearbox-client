use crate::db::open_db;
use crate::db::store::list_highlights;
use crate::db::store::ListedHighlight;
use crate::db::vector::delete_vector;

/// List stored highlights in reverse chronological order.
#[tauri::command]
pub fn list_stored_highlights(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<ListedHighlight>, String> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    list_highlights(limit, offset)
}

/// Paginated history endpoint used by the desktop React frontend.
/// Thin wrapper around `list_highlights`.
#[tauri::command]
pub fn get_history_paginated(limit: u32, offset: u32) -> Result<Vec<ListedHighlight>, String> {
    list_highlights(limit as usize, offset as usize)
}

/// Internal delete implementation that accepts an open connection.
/// Used by the command handler and tests.
pub fn delete_highlight_conn(conn: &mut rusqlite::Connection, id: &str) -> Result<(), String> {
    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to start transaction: {e}"))?;

    tx.execute(
        "DELETE FROM highlights WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| format!("Failed to delete highlight: {e}"))?;

    tx.execute(
        "DELETE FROM highlights_fts WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| format!("Failed to delete FTS entry: {e}"))?;

    tx.commit()
        .map_err(|e| format!("Failed to commit delete transaction: {e}"))?;

    // Best-effort LanceDB deletion (do not fail the whole command if vector DB is down).
    if let Err(e) = delete_vector(id) {
        eprintln!("Warning: failed to delete vector for highlight {id}: {e}");
    }

    Ok(())
}

/// Delete a single highlight by ID.
/// Removes from SQLite `highlights`, SQLite FTS `highlights_fts`, and LanceDB vectors.
#[tauri::command]
pub fn delete_highlight(id: String) -> Result<(), String> {
    let mut conn = open_db().map_err(|e| format!("Failed to open DB: {e}"))?;
    delete_highlight_conn(&mut conn, &id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db;

    #[test]
    fn test_delete_highlight_roundtrip() {
        let mut conn = open_test_db().unwrap();

        // Insert a highlight + fts row
        conn.execute(
            "INSERT INTO highlights (id, text, summary, tags) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                "hl-1",
                "The quick brown fox.",
                "A fox summary.",
                "[\"fox\"]"
            ],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO highlights_fts (id, text, summary, tags_text) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["hl-1", "The quick brown fox.", "A fox summary.", "fox"],
        )
        .unwrap();

        // Delete it via the internal helper
        let result = delete_highlight_conn(&mut conn, "hl-1");
        assert!(
            result.is_ok(),
            "delete_highlight_conn should succeed: {:?}",
            result.err()
        );

        // Verify SQLite deletion
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM highlights WHERE id = ?1",
                rusqlite::params!["hl-1"],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();
        assert_eq!(count, 0, "highlight should be removed from SQLite");

        let fts_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM highlights_fts WHERE id = ?1",
                rusqlite::params!["hl-1"],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();
        assert_eq!(fts_count, 0, "highlight should be removed from FTS");
    }

    #[test]
    fn test_delete_nonexistent_highlight_ok() {
        let mut conn = open_test_db().unwrap();
        // Deleting a non-existent highlight should return Ok() — nothing to delete.
        let result = delete_highlight_conn(&mut conn, "no-such-id");
        assert!(
            result.is_ok(),
            "deleting nonexistent highlight should succeed"
        );
    }
}
