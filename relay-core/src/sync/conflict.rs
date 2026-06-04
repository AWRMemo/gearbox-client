use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Conflict {
    pub id: String,
    pub record_type: String,
    pub record_id: String,
    pub local_version: Option<String>,
    pub remote_version: Option<String>,
    pub resolved_at: Option<String>,
    pub resolution: Option<String>,
    pub created_at: String,
}

/// Log an unresolved conflict.
pub fn log_conflict(
    conn: &Connection,
    record_type: &str,
    record_id: &str,
    local_version: Option<&str>,
    remote_version: Option<&str>,
) -> Result<(), String> {
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO sync_conflicts (id, record_type, record_id, local_version, remote_version)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, record_type, record_id, local_version, remote_version,],
    )
    .map_err(|e| format!("Failed to log conflict: {e}"))?;
    Ok(())
}

/// List unresolved conflicts (resolved_at IS NULL), ordered newest first.
pub fn list_conflicts(conn: &Connection) -> Result<Vec<Conflict>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, record_type, record_id, local_version, remote_version, resolved_at, resolution, created_at
             FROM sync_conflicts
             WHERE resolved_at IS NULL
             ORDER BY created_at DESC",
        )
        .map_err(|e| format!("Failed to prepare conflict query: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Conflict {
                id: row.get(0)?,
                record_type: row.get(1)?,
                record_id: row.get(2)?,
                local_version: row.get(3)?,
                remote_version: row.get(4)?,
                resolved_at: row.get(5)?,
                resolution: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to query conflicts: {e}"))?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| format!("Conflict row error: {e}"))?);
    }
    Ok(out)
}

/// Resolve a conflict by applying the chosen resolution.
pub fn resolve_conflict_with_action(
    conn: &Connection,
    conflict_id: &str,
    resolution: &str,
) -> Result<(), String> {
    let conflict: Conflict = conn
        .query_row(
            "SELECT id, record_type, record_id, local_version, remote_version, resolved_at, resolution, created_at FROM sync_conflicts WHERE id = ?1",
            [conflict_id],
            |row| {
                Ok(Conflict {
                    id: row.get(0)?,
                    record_type: row.get(1)?,
                    record_id: row.get(2)?,
                    local_version: row.get(3)?,
                    remote_version: row.get(4)?,
                    resolved_at: row.get(5)?,
                    resolution: row.get(6)?,
                    created_at: row.get(7)?,
                })
            },
        )
        .map_err(|e| format!("Failed to fetch conflict: {e}"))?;

    if resolution == "accept_remote" {
        if let Some(ref remote_json) = conflict.remote_version {
            let now = chrono::Utc::now().to_rfc3339();
            crate::sync::engine::apply_remote(
                conn,
                &conflict.record_type,
                &conflict.record_id,
                remote_json,
                &now,
            )
            .map_err(|e| format!("Failed to apply remote version: {e}"))?;
        }
    }
    // "keep_local" does nothing to the live record

    conn.execute(
        "UPDATE sync_conflicts SET resolved_at = datetime('now'), resolution = ?1 WHERE id = ?2",
        rusqlite::params![resolution, conflict_id],
    )
    .map_err(|e| format!("Failed to mark conflict resolved: {e}"))?;

    Ok(())
}

/// Mark a conflict as resolved.
pub fn resolve_conflict(
    conn: &Connection,
    conflict_id: &str,
    resolution: &str,
) -> Result<(), String> {
    conn.execute(
        "UPDATE sync_conflicts
         SET resolved_at = datetime('now'), resolution = ?1
         WHERE id = ?2",
        rusqlite::params![resolution, conflict_id],
    )
    .map_err(|e| format!("Failed to resolve conflict: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE sync_conflicts (
                id TEXT PRIMARY KEY,
                record_type TEXT NOT NULL,
                record_id TEXT NOT NULL,
                local_version TEXT,
                remote_version TEXT,
                resolved_at TEXT,
                resolution TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE highlights (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                source_url TEXT,
                source_title TEXT,
                source_author TEXT,
                summary TEXT NOT NULL DEFAULT '',
                tags TEXT NOT NULL DEFAULT '',
                connection_suggestion TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_modified TEXT NOT NULL DEFAULT (datetime('now')),
                sync_status TEXT NOT NULL DEFAULT 'local'
            );",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_log_and_list_conflicts() {
        let conn = setup_db();
        log_conflict(&conn, "highlight", "hl1", Some("local-a"), Some("remote-b")).unwrap();
        log_conflict(&conn, "stream", "st1", None, Some("remote-x")).unwrap();

        let conflicts = list_conflicts(&conn).unwrap();
        assert_eq!(conflicts.len(), 2);
        let types: std::collections::HashSet<String> =
            conflicts.iter().map(|c| c.record_type.clone()).collect();
        assert!(types.contains("highlight"));
        assert!(types.contains("stream"));
        let hl = conflicts
            .iter()
            .find(|c| c.record_type == "highlight")
            .unwrap();
        assert_eq!(hl.local_version.as_deref(), Some("local-a"));
    }

    #[test]
    fn test_resolve_conflict() {
        let conn = setup_db();
        log_conflict(&conn, "highlight", "hl1", Some("local-a"), Some("remote-b")).unwrap();

        let conflicts = list_conflicts(&conn).unwrap();
        assert_eq!(conflicts.len(), 1);

        resolve_conflict(&conn, &conflicts[0].id, "merged").unwrap();

        let after = list_conflicts(&conn).unwrap();
        assert_eq!(after.len(), 0);
    }

    #[test]
    fn test_resolve_conflict_with_action_accept_remote() {
        let conn = setup_db();
        // Insert a local highlight
        conn.execute(
            "INSERT INTO highlights (id, text, last_modified, sync_status) VALUES ('hl1', 'local text', '2024-01-01T00:00:00Z', 'synced')",
            [],
        )
        .unwrap();

        // Insert a conflict with a remote version
        let remote_json = serde_json::json!({
            "id": "hl1",
            "text": "remote text",
            "source_url": null,
            "source_title": null,
            "source_author": null,
            "summary": "",
            "tags": "",
            "connection_suggestion": null,
            "created_at": "2024-01-01T00:00:00Z",
            "last_modified": "2025-01-01T00:00:00Z",
            "sync_status": "synced",
        })
        .to_string();

        log_conflict(
            &conn,
            "highlights",
            "hl1",
            Some("2024-01-01T00:00:00Z"),
            Some(&remote_json),
        )
        .unwrap();

        let conflicts = list_conflicts(&conn).unwrap();
        assert_eq!(conflicts.len(), 1);

        resolve_conflict_with_action(&conn, &conflicts[0].id, "accept_remote").unwrap();

        let after = list_conflicts(&conn).unwrap();
        assert_eq!(after.len(), 0);

        let text: String = conn
            .query_row("SELECT text FROM highlights WHERE id = 'hl1'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(text, "remote text");
    }

    #[test]
    fn test_resolve_conflict_with_action_keep_local() {
        let conn = setup_db();
        conn.execute(
            "INSERT INTO highlights (id, text, last_modified, sync_status) VALUES ('hl1', 'local text', '2024-01-01T00:00:00Z', 'synced')",
            [],
        )
        .unwrap();

        let remote_json = serde_json::json!({
            "id": "hl1",
            "text": "remote text",
            "source_url": null,
            "source_title": null,
            "source_author": null,
            "summary": "",
            "tags": "",
            "connection_suggestion": null,
            "created_at": "2024-01-01T00:00:00Z",
            "last_modified": "2025-01-01T00:00:00Z",
            "sync_status": "synced",
        })
        .to_string();

        log_conflict(
            &conn,
            "highlights",
            "hl1",
            Some("2024-01-01T00:00:00Z"),
            Some(&remote_json),
        )
        .unwrap();

        let conflicts = list_conflicts(&conn).unwrap();
        resolve_conflict_with_action(&conn, &conflicts[0].id, "keep_local").unwrap();

        let text: String = conn
            .query_row("SELECT text FROM highlights WHERE id = 'hl1'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(text, "local text");
    }
}
