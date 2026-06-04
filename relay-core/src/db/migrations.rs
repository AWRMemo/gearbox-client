/// Migration runner using PRAGMA user_version.
///
/// Version history:
///   0/1 — initial schema (before sync fields)
///   2   — adds last_modified, sync_status, sync_metadata, sync_conflicts, sync_credentials
use rusqlite::Connection;

/// Run migrations. Safe to call multiple times.
pub fn migrate(conn: &Connection) -> Result<(), String> {
    let user_version: i32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| format!("Failed to read user_version: {e}"))?;

    if user_version < 2 {
        apply_v2(conn)?;
        conn.execute("PRAGMA user_version = 2", [])
            .map_err(|e| format!("Failed to bump user_version: {e}"))?;
    }

    Ok(())
}

fn apply_v2(conn: &Connection) -> Result<(), String> {
    let tables_columns: &[(&str, &[&str])] = &[
        (
            "highlights",
            &[
                "last_modified TEXT NOT NULL DEFAULT (datetime('now'))",
                "sync_status TEXT NOT NULL DEFAULT 'local'",
            ],
        ),
        (
            "streams",
            &[
                "last_modified TEXT NOT NULL DEFAULT (datetime('now'))",
                "sync_status TEXT NOT NULL DEFAULT 'local'",
            ],
        ),
        (
            "stream_highlights",
            &[
                "last_modified TEXT NOT NULL DEFAULT (datetime('now'))",
                "sync_status TEXT NOT NULL DEFAULT 'local'",
            ],
        ),
        (
            "subscriptions",
            &[
                "last_modified TEXT NOT NULL DEFAULT (datetime('now'))",
                "sync_status TEXT NOT NULL DEFAULT 'local'",
            ],
        ),
        (
            "user_profile",
            &[
                "last_modified TEXT NOT NULL DEFAULT (datetime('now'))",
                "sync_status TEXT NOT NULL DEFAULT 'local'",
            ],
        ),
    ];

    for (table, columns) in tables_columns {
        let table_exists: bool = conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1 LIMIT 1",
                rusqlite::params![table],
                |_row| Ok(true),
            )
            .unwrap_or(false);
        if !table_exists {
            continue;
        }
        for col_def in *columns {
            let col_name = col_def.split_whitespace().next().unwrap_or(col_def);
            let has_col: bool = conn
                .query_row(
                    "SELECT 1 FROM pragma_table_info(?1) WHERE name = ?2 LIMIT 1",
                    rusqlite::params![table, col_name],
                    |_row| Ok(true),
                )
                .unwrap_or(false);
            if !has_col {
                let sql = format!("ALTER TABLE {table} ADD COLUMN {col_def}");
                conn.execute(&sql, [])
                    .map_err(|e| format!("Failed to add column {col_name} to {table}: {e}"))?;
            }
        }
    }

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sync_metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sync_conflicts (
            id TEXT PRIMARY KEY,
            record_type TEXT NOT NULL,
            record_id TEXT NOT NULL,
            local_version TEXT,
            remote_version TEXT,
            resolved_at TEXT,
            resolution TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS sync_credentials (
            user_email TEXT PRIMARY KEY,
            password_hash TEXT NOT NULL,
            salt_auth TEXT NOT NULL,
            encryption_key_salt TEXT NOT NULL,
            server_url TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .map_err(|e| format!("Failed to create sync tables during migration: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_adds_columns_and_tables() {
        let conn = Connection::open_in_memory().unwrap();
        // Create a minimal v1 schema (without sync fields)
        conn.execute_batch(
            "CREATE TABLE highlights (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL
            );
            CREATE TABLE streams (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL
            );
            CREATE TABLE stream_highlights (
                stream_id TEXT NOT NULL,
                highlight_id TEXT NOT NULL,
                PRIMARY KEY (stream_id, highlight_id)
            );
            CREATE TABLE subscriptions (
                user_id TEXT NOT NULL,
                stream_id TEXT NOT NULL,
                PRIMARY KEY (user_id, stream_id)
            );
            CREATE TABLE user_profile (
                user_id TEXT PRIMARY KEY
            );",
        )
        .unwrap();

        migrate(&conn).unwrap();

        let col_exists = |table: &str, col: &str| -> bool {
            conn.query_row(
                "SELECT 1 FROM pragma_table_info(?1) WHERE name = ?2 LIMIT 1",
                rusqlite::params![table, col],
                |_row| Ok(true),
            )
            .unwrap_or(false)
        };

        assert!(col_exists("highlights", "last_modified"));
        assert!(col_exists("highlights", "sync_status"));
        assert!(col_exists("streams", "last_modified"));
        assert!(col_exists("streams", "sync_status"));
        assert!(col_exists("stream_highlights", "last_modified"));
        assert!(col_exists("stream_highlights", "sync_status"));
        assert!(col_exists("subscriptions", "last_modified"));
        assert!(col_exists("subscriptions", "sync_status"));
        assert!(col_exists("user_profile", "last_modified"));
        assert!(col_exists("user_profile", "sync_status"));

        let table_exists = |name: &str| -> bool {
            conn.query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1 LIMIT 1",
                [name],
                |_row| Ok(true),
            )
            .unwrap_or(false)
        };

        assert!(table_exists("sync_metadata"));
        assert!(table_exists("sync_conflicts"));
        assert!(table_exists("sync_credentials"));

        let version: i32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 2);
    }

    #[test]
    fn test_migration_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE highlights (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                last_modified TEXT NOT NULL DEFAULT (datetime('now')),
                sync_status TEXT NOT NULL DEFAULT 'local'
            );
            CREATE TABLE sync_metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        )
        .unwrap();

        // Should not fail even though columns/tables already exist
        migrate(&conn).unwrap();

        let version: i32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 2);
    }
}
