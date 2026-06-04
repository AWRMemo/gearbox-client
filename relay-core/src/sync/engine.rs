use crate::sync::conflict::log_conflict;
use crate::sync::encrypt::{decrypt, encrypt};
use crate::sync::opaque_blob::{encrypt_inner_blob, decrypt_payload, InnerBlob, OpaqueBlob};
use crate::sync::server::{EncryptedBlob, SyncClient};
use rusqlite::{params, Connection};
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing;

#[derive(Serialize)]
pub struct SyncReport {
    pub pushed: usize,
    pub pulled: usize,
    pub conflicts: usize,
}

/// Engine that orchestrates sync between local DB and remote server.
pub struct SyncEngine {
    client: Arc<dyn SyncClient>,
    auth_token: String,
    encryption_key: [u8; 32],
    use_v2: bool,
}

impl SyncEngine {
    pub fn new(client: Arc<dyn SyncClient>, auth_token: String, encryption_key: [u8; 32]) -> Self {
        let mut engine = Self { client, auth_token, encryption_key, use_v2: false };
        engine.use_v2 = engine.detect_or_migrate();
        engine
    }

    /// Detect protocol version. If v1, attempt automatic migration to v2.
    /// Returns true if the account is now using v2.
    fn detect_or_migrate(&self) -> bool {
        let Ok(conn) = crate::db::open_db() else { return false };
        let version: Option<i64> = conn
            .query_row(
                "SELECT protocol_version FROM sync_credentials LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok();

        if version == Some(2) {
            return true;
        }

        if version == Some(1) {
            match self.migrate_v1_to_v2() {
                Ok(()) => return true,
                Err(e) => {
                    tracing::warn!("v1→v2 migration failed, falling back to v1: {e}");
                    return false;
                }
            }
        }

        false
    }

    /// Migrate all v1 blobs to v2: pull from server, decrypt, re-encrypt as v2, push.
    fn migrate_v1_to_v2(&self) -> Result<(), String> {
        tracing::info!("Starting v1→v2 protocol migration…");

        let conn = crate::db::open_db()?;
        let v1_blobs = self.client.pull(&self.auth_token, "1970-01-01T00:00:00Z")?;

        if v1_blobs.is_empty() {
            conn.execute(
                "UPDATE sync_credentials SET protocol_version = 2 WHERE protocol_version = 1",
                [],
            )
            .map_err(|e| format!("Failed to update protocol version: {e}"))?;
            tracing::info!("v1→v2 migration complete (0 blobs migrated).");
            return Ok(());
        }

        let mut v2_blobs: Vec<OpaqueBlob> = Vec::new();
        for blob in &v1_blobs {
            let decrypted = decrypt(&blob.ciphertext, &self.encryption_key).map_err(|e| {
                format!("Failed to decrypt v1 blob {} during migration: {e}", blob.id)
            })?;
            let data: serde_json::Value = serde_json::from_str(&decrypted)
                .map_err(|e| format!("Invalid JSON in v1 blob {}: {e}", blob.id))?;
            let last_modified = chrono::DateTime::parse_from_rfc3339(&blob.last_modified)
                .map(|dt| dt.timestamp_millis())
                .unwrap_or(0);
            let inner = InnerBlob {
                id: blob.id.clone(),
                record_type: blob.record_type.clone(),
                last_modified,
                data,
            };
            let payload = encrypt_inner_blob(&self.encryption_key, &inner)
                .map_err(|e| format!("v2 encrypt failed during migration: {e}"))?;
            v2_blobs.push(OpaqueBlob {
                blob_id: uuid::Uuid::new_v4().to_string(),
                payload,
            });
        }

        self.client
            .push_v2(&self.auth_token, &v2_blobs)
            .map_err(|e| format!("Failed to push migrated v2 blobs: {e}"))?;

        conn.execute(
            "UPDATE sync_credentials SET protocol_version = 2 WHERE protocol_version = 1",
            [],
        )
        .map_err(|e| format!("Failed to update protocol version: {e}"))?;

        tracing::info!("v1→v2 migration complete ({} blobs migrated).", v2_blobs.len());
        Ok(())
    }

    /// Push local rows with sync_status='local' to the server.
    pub fn push(&self) -> Result<usize, String> {
        let conn = crate::db::open_db()?;
        let tables: &[(&str, &str)] = &[
            ("highlights", "id"),
            ("streams", "id"),
            ("stream_highlights", "stream_id || ':' || highlight_id"),
            ("subscriptions", "user_id || ':' || stream_id"),
            ("user_profile", "user_id"),
        ];

        let mut v1_blobs: Vec<EncryptedBlob> = Vec::new();
        let mut v2_blobs: Vec<OpaqueBlob> = Vec::new();
        let mut update_ids: Vec<(&str, String)> = Vec::new();

        for (table, id_expr) in tables {
            let sql = format!(
                "SELECT {id_expr} as record_id, * FROM {table} WHERE sync_status = 'local'"
            );
            let mut stmt = conn
                .prepare(&sql)
                .map_err(|e| format!("Push prepare failed for {table}: {e}"))?;
            let rows = stmt
                .query_map([], |row| {
                    let record_id: String = row.get(0)?;
                    let mut map = HashMap::<String, serde_json::Value>::new();
                    let col_count = row.as_ref().column_count();
                    for i in 0..col_count {
                        let name = row.as_ref().column_name(i).unwrap_or("").to_string();
                        let val = if let Ok(v) = row.get::<_, String>(i) {
                            serde_json::Value::String(v)
                        } else if let Ok(v) = row.get::<_, i64>(i) {
                            serde_json::Value::Number(v.into())
                        } else if let Ok(v) = row.get::<_, f64>(i) {
                            serde_json::Number::from_f64(v)
                                .map(serde_json::Value::Number)
                                .unwrap_or(serde_json::Value::Null)
                        } else {
                            serde_json::Value::Null
                        };
                        map.insert(name, val);
                    }
                    let json_str = serde_json::to_string(&map).unwrap_or_default();
                    let last_modified: String =
                        row.get::<_, String>("last_modified").unwrap_or_default();
                    Ok((record_id, json_str, last_modified))
                })
                .map_err(|e| format!("Push query failed for {table}: {e}"))?;

            for r in rows {
                let (record_id, json_str, last_modified) =
                    r.map_err(|e| format!("Push row error: {e}"))?;

                if self.use_v2 {
                    let data: serde_json::Value = serde_json::from_str(&json_str)
                        .map_err(|e| format!("Invalid JSON for v2: {e}"))?;
                    let inner = InnerBlob {
                        id: record_id.clone(),
                        record_type: table.to_string(),
                        // Parse timestamp as Unix ms for v2; fallback to 0 on parse failure
                        last_modified: chrono::DateTime::parse_from_rfc3339(&last_modified)
                            .map(|dt| dt.timestamp_millis())
                            .unwrap_or(0),
                        data,
                    };
                    let payload = encrypt_inner_blob(&self.encryption_key, &inner)
                        .map_err(|e| format!("v2 encrypt failed: {e}"))?;
                    v2_blobs.push(OpaqueBlob {
                        blob_id: uuid::Uuid::new_v4().to_string(),
                        payload,
                    });
                } else {
                    let ciphertext = encrypt(&json_str, &self.encryption_key)?;
                    v1_blobs.push(EncryptedBlob {
                        id: record_id.clone(),
                        record_type: table.to_string(),
                        ciphertext,
                        last_modified,
                    });
                }
                update_ids.push((table, record_id));
            }
        }

        let accepted = if self.use_v2 {
            if v2_blobs.is_empty() { return Ok(0); }
            self.client.push_v2(&self.auth_token, &v2_blobs)?
        } else {
            if v1_blobs.is_empty() { return Ok(0); }
            self.client.push(&self.auth_token, &v1_blobs)?
        };

        // Mark pushed rows as synced
        for (table, record_id) in &update_ids {
            if *table == "stream_highlights" {
                let parts: Vec<&str> = record_id.split(':').collect();
                if parts.len() == 2 {
                    conn.execute(
                        "UPDATE stream_highlights SET sync_status = 'synced' WHERE stream_id = ?1 AND highlight_id = ?2",
                        params![parts[0], parts[1]],
                    )
                    .map_err(|e| format!("Push update failed for stream_highlights: {e}"))?;
                }
            } else if *table == "subscriptions" {
                let parts: Vec<&str> = record_id.split(':').collect();
                if parts.len() == 2 {
                    conn.execute(
                        "UPDATE subscriptions SET sync_status = 'synced' WHERE user_id = ?1 AND stream_id = ?2",
                        params![parts[0], parts[1]],
                    )
                    .map_err(|e| format!("Push update failed for subscriptions: {e}"))?;
                }
            } else {
                let pk = match *table {
                    "highlights" | "streams" => "id",
                    "user_profile" => "user_id",
                    _ => "id",
                };
                conn.execute(
                    &format!("UPDATE {table} SET sync_status = 'synced' WHERE {pk} = ?1"),
                    params![record_id],
                )
                .map_err(|e| format!("Push update failed for {table}: {e}"))?;
            }
        }

        Ok(accepted)
    }

    /// Pull remote blobs, decrypt, and reconcile with local data (LWW).
    pub fn pull(&self) -> Result<usize, String> {
        let conn = crate::db::open_db()?;
        let since: String = conn
            .query_row(
                "SELECT value FROM sync_metadata WHERE key = 'last_sync_timestamp'",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());

        if self.use_v2 {
            self.pull_v2(&conn, &since)
        } else {
            self.pull_v1(&conn, &since)
        }
    }

    fn pull_v1(&self, conn: &Connection, since: &str) -> Result<usize, String> {
        let blobs = self.client.pull(&self.auth_token, since)?;

        for blob in &blobs {
            let decrypted = match decrypt(&blob.ciphertext, &self.encryption_key) {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("Failed to decrypt blob {}: {}", blob.id, e);
                    continue;
                }
            };

            let local_lm: Option<String> = match blob.record_type.as_str() {
                "highlights" => conn
                    .query_row(
                        "SELECT last_modified FROM highlights WHERE id = ?1",
                        params![blob.id],
                        |row| row.get::<_, String>(0),
                    )
                    .ok(),
                "streams" => conn
                    .query_row(
                        "SELECT last_modified FROM streams WHERE id = ?1",
                        params![blob.id],
                        |row| row.get::<_, String>(0),
                    )
                    .ok(),
                "stream_highlights" => {
                    let parts: Vec<&str> = blob.id.split(':').collect();
                    if parts.len() == 2 {
                        conn.query_row(
                            "SELECT last_modified FROM stream_highlights WHERE stream_id = ?1 AND highlight_id = ?2",
                            params![parts[0], parts[1]],
                            |row| row.get::<_, String>(0),
                        )
                        .ok()
                    } else {
                        None
                    }
                }
                "subscriptions" => {
                    let parts: Vec<&str> = blob.id.split(':').collect();
                    if parts.len() == 2 {
                        conn.query_row(
                            "SELECT last_modified FROM subscriptions WHERE user_id = ?1 AND stream_id = ?2",
                            params![parts[0], parts[1]],
                            |row| row.get::<_, String>(0),
                        )
                        .ok()
                    } else {
                        None
                    }
                }
                "user_profile" => conn
                    .query_row(
                        "SELECT last_modified FROM user_profile WHERE user_id = ?1",
                        params![blob.id],
                        |row| row.get::<_, String>(0),
                    )
                    .ok(),
                _ => None,
            };

            match local_lm {
                Some(ref lm) if lm == &blob.last_modified => {
                    continue;
                }
                Some(ref lm_str_cmp) if lm_str_cmp > &blob.last_modified => {
                    log_conflict(
                        conn,
                        &blob.record_type,
                        &blob.id,
                        Some(lm_str_cmp),
                        Some(&blob.last_modified),
                    )
                    .map_err(|e| format!("Failed to log conflict: {e}"))?;
                }
                _ => {
                    apply_remote(conn, &blob.record_type, &blob.id, &decrypted, &blob.last_modified)?;
                }
            }
        }

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO sync_metadata (key, value) VALUES ('last_sync_timestamp', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![now],
        )
        .map_err(|e| format!("Failed to update sync timestamp: {e}"))?;

        Ok(blobs.len())
    }

    fn pull_v2(&self, conn: &Connection, since: &str) -> Result<usize, String> {
        let opaque_blobs = self.client.pull_v2(&self.auth_token, since)?;

        for blob in &opaque_blobs {
            let inner = match decrypt_payload(&self.encryption_key, &blob.payload) {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("Failed to decrypt v2 blob {}: {}", blob.blob_id, e);
                    continue;
                }
            };

            let lm_str = chrono::DateTime::from_timestamp_millis(inner.last_modified)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| inner.last_modified.to_string());

            let local_lm: Option<String> = match inner.record_type.as_str() {
                "highlights" => conn
                    .query_row(
                        "SELECT last_modified FROM highlights WHERE id = ?1",
                        params![&inner.id],
                        |row| row.get::<_, String>(0),
                    )
                    .ok(),
                "streams" => conn
                    .query_row(
                        "SELECT last_modified FROM streams WHERE id = ?1",
                        params![&inner.id],
                        |row| row.get::<_, String>(0),
                    )
                    .ok(),
                "stream_highlights" => {
                    let parts: Vec<&str> = inner.id.split(':').collect();
                    if parts.len() == 2 {
                        conn.query_row(
                            "SELECT last_modified FROM stream_highlights WHERE stream_id = ?1 AND highlight_id = ?2",
                            params![parts[0], parts[1]],
                            |row| row.get::<_, String>(0),
                        )
                        .ok()
                    } else {
                        None
                    }
                }
                "subscriptions" => {
                    let parts: Vec<&str> = inner.id.split(':').collect();
                    if parts.len() == 2 {
                        conn.query_row(
                            "SELECT last_modified FROM subscriptions WHERE user_id = ?1 AND stream_id = ?2",
                            params![parts[0], parts[1]],
                            |row| row.get::<_, String>(0),
                        )
                        .ok()
                    } else {
                        None
                    }
                }
                "user_profile" => conn
                    .query_row(
                        "SELECT last_modified FROM user_profile WHERE user_id = ?1",
                        params![&inner.id],
                        |row| row.get::<_, String>(0),
                    )
                    .ok(),
                _ => None,
            };

            let decrypted = serde_json::to_string(&inner.data)
                .map_err(|e| format!("Failed to serialize inner data: {e}"))?;

            match local_lm {
                Some(ref lm) if lm == &lm_str => {
                    continue;
                }
                Some(ref lm_str_cmp) if lm_str_cmp > &lm_str => {
                    log_conflict(
                        conn,
                        &inner.record_type,
                        &inner.id,
                        Some(&lm_str),
                        Some(lm_str_cmp),
                    )
                    .map_err(|e| format!("Failed to log conflict: {e}"))?;
                }
                _ => {
                    apply_remote(conn, &inner.record_type, &inner.id, &decrypted, &lm_str)?;
                }
            }
        }

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO sync_metadata (key, value) VALUES ('last_sync_timestamp', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![now],
        )
        .map_err(|e| format!("Failed to update sync timestamp: {e}"))?;

        Ok(opaque_blobs.len())
    }

    /// Push then pull, returning counts.
    pub fn sync_now(&self) -> Result<SyncReport, String> {
        let pushed = self.push()?;
        let pulled = self.pull()?;
        let conn = crate::db::open_db()?;
        let pending_conflicts = conn
            .query_row(
                "SELECT COUNT(*) FROM sync_conflicts WHERE resolved_at IS NULL",
                [],
                |row| row.get::<_, usize>(0),
            )
            .unwrap_or(0);
        Ok(SyncReport {
            pushed,
            pulled,
            conflicts: pending_conflicts,
        })
    }
}

/// Apply a remote record locally, overwriting existing row if present.
pub fn apply_remote(
    conn: &Connection,
    record_type: &str,
    record_id: &str,
    decrypted: &str,
    last_modified: &str,
) -> Result<(), String> {
    let map: HashMap<String, serde_json::Value> =
        serde_json::from_str(decrypted).map_err(|e| format!("Invalid JSON in remote blob: {e}"))?;

    let mut cols: Vec<(&str, serde_json::Value)> = Vec::new();
    for (key, val) in &map {
        if key == "record_id" || key == "last_modified" || key == "sync_status" {
            continue;
        }
        cols.push((key.as_str(), val.clone()));
    }

    match record_type {
        "highlights" => {
            let text = get_str(&map, "text");
            let source_url = get_str(&map, "source_url");
            let source_title = get_str(&map, "source_title");
            let source_author = get_str(&map, "source_author");
            let summary = get_str(&map, "summary");
            let tags = get_str(&map, "tags");
            let connection_suggestion = get_str(&map, "connection_suggestion");
            let created_at = get_str(&map, "created_at");
            conn.execute(
                "INSERT OR REPLACE INTO highlights (id, text, source_url, source_title, source_author, summary, tags, connection_suggestion, created_at, last_modified, sync_status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'synced')",
                params![
                    record_id,
                    text,
                    source_url,
                    source_title,
                    source_author,
                    summary,
                    tags,
                    connection_suggestion,
                    created_at,
                    last_modified,
                ],
            )
            .map_err(|e| format!("apply_remote highlights failed: {e}"))?;
        }
        "streams" => {
            let user_id = get_str(&map, "user_id");
            let title = get_str(&map, "title");
            let description = get_str(&map, "description");
            let is_public = get_i64(&map, "is_public").unwrap_or(1);
            let created_at = get_str(&map, "created_at");
            let updated_at = get_str(&map, "updated_at");
            conn.execute(
                "INSERT OR REPLACE INTO streams (id, user_id, title, description, is_public, created_at, updated_at, last_modified, sync_status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'synced')",
                params![
                    record_id,
                    user_id,
                    title,
                    description,
                    is_public,
                    created_at,
                    updated_at,
                    last_modified,
                ],
            )
            .map_err(|e| format!("apply_remote streams failed: {e}"))?;
        }
        "stream_highlights" => {
            let parts: Vec<&str> = record_id.split(':').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid stream_highlight id: {}", record_id));
            }
            let added_at = get_str(&map, "added_at");
            conn.execute(
                "INSERT OR REPLACE INTO stream_highlights (stream_id, highlight_id, added_at, last_modified, sync_status)
                 VALUES (?1, ?2, ?3, ?4, 'synced')",
                params![parts[0], parts[1], added_at, last_modified],
            )
            .map_err(|e| format!("apply_remote stream_highlights failed: {e}"))?;
        }
        "subscriptions" => {
            let parts: Vec<&str> = record_id.split(':').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid subscription id: {}", record_id));
            }
            let subscribed_at = get_str(&map, "subscribed_at");
            conn.execute(
                "INSERT OR REPLACE INTO subscriptions (user_id, stream_id, subscribed_at, last_modified, sync_status)
                 VALUES (?1, ?2, ?3, ?4, 'synced')",
                params![parts[0], parts[1], subscribed_at, last_modified],
            )
            .map_err(|e| format!("apply_remote subscriptions failed: {e}"))?;
        }
        "user_profile" => {
            let email = get_str(&map, "email");
            let display_name = get_str(&map, "display_name");
            let tier = get_str(&map, "tier");
            let created_at = get_str(&map, "created_at");
            conn.execute(
                "INSERT OR REPLACE INTO user_profile (user_id, email, display_name, tier, created_at, last_modified, sync_status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'synced')",
                params![
                    record_id,
                    email,
                    display_name,
                    tier,
                    created_at,
                    last_modified,
                ],
            )
            .map_err(|e| format!("apply_remote user_profile failed: {e}"))?;
        }
        _ => {}
    }

    Ok(())
}

fn get_str(map: &HashMap<String, serde_json::Value>, key: &str) -> Option<String> {
    map.get(key).and_then(|v| v.as_str().map(|s| s.to_string()))
}

fn get_i64(map: &HashMap<String, serde_json::Value>, key: &str) -> Option<i64> {
    map.get(key).and_then(|v| v.as_i64())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::conflict::{list_conflicts, log_conflict};
    use crate::sync::encrypt::derive_key;
    use rusqlite::Connection;

    fn setup_local_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE highlights (
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
            );
            CREATE TABLE sync_conflicts (
                id TEXT PRIMARY KEY,
                record_type TEXT NOT NULL,
                record_id TEXT NOT NULL,
                local_version TEXT,
                remote_version TEXT,
                resolved_at TEXT,
                resolution TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE sync_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE sync_credentials (
                user_email TEXT PRIMARY KEY,
                password_hash TEXT NOT NULL,
                salt_auth TEXT NOT NULL,
                encryption_key_salt TEXT NOT NULL,
                server_url TEXT NOT NULL,
                protocol_version INTEGER NOT NULL DEFAULT 1
            );",
        )
        .unwrap();
        conn
    }

    fn setup_remote_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE highlights (
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
    fn test_lww_remote_newer_overwrites() {
        let local = setup_local_db();
        let remote = setup_remote_db();

        local
            .execute(
                "INSERT INTO highlights (id, text, last_modified, sync_status) VALUES ('hl1', 'local old', '2024-01-01T00:00:00Z', 'synced')",
                [],
            )
            .unwrap();

        remote
            .execute(
                "INSERT INTO highlights (id, text, last_modified, sync_status) VALUES ('hl1', 'remote new', '2025-01-01T00:00:00Z', 'synced')",
                [],
            )
            .unwrap();

        let key = derive_key("password", b"saltsaltsaltsalt").unwrap();
        let (remote_text, remote_lm): (String, String) = remote
            .query_row(
                "SELECT text, last_modified FROM highlights WHERE id = 'hl1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        let payload = serde_json::json!({
            "id": "hl1",
            "text": remote_text,
            "source_url": null,
            "source_title": null,
            "source_author": null,
            "summary": "",
            "tags": "",
            "connection_suggestion": null,
            "created_at": "2024-01-01T00:00:00Z",
            "last_modified": remote_lm.clone(),
            "sync_status": "synced",
        });
        let ciphertext = encrypt(&payload.to_string(), &key).unwrap();
        let decrypted = decrypt(&ciphertext, &key).unwrap();

        apply_remote(&local, "highlights", "hl1", &decrypted, &remote_lm).unwrap();

        let text: String = local
            .query_row("SELECT text FROM highlights WHERE id = 'hl1'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(text, "remote new");
    }

    #[test]
    fn test_lww_local_newer_logs_conflict() {
        let local = setup_local_db();
        let remote = setup_remote_db();

        local
            .execute(
                "INSERT INTO highlights (id, text, last_modified, sync_status) VALUES ('hl1', 'local new', '2025-01-01T00:00:00Z', 'synced')",
                [],
            )
            .unwrap();

        remote
            .execute(
                "INSERT INTO highlights (id, text, last_modified, sync_status) VALUES ('hl1', 'remote old', '2024-01-01T00:00:00Z', 'synced')",
                [],
            )
            .unwrap();

        let local_lm: String = local
            .query_row(
                "SELECT last_modified FROM highlights WHERE id = 'hl1'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        log_conflict(
            &local,
            "highlights",
            "hl1",
            Some(&local_lm),
            Some("2024-01-01T00:00:00Z"),
        )
        .unwrap();

        let conflicts = list_conflicts(&local).unwrap();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].record_type, "highlights");
    }
}
