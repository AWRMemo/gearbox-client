#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use rusqlite::Connection;

    use crate::db::{open_db, set_data_dir};
    use crate::sync::conflict::list_conflicts;
    use crate::sync::encrypt::{decrypt, encrypt};
    use crate::sync::engine::SyncEngine;
    use crate::sync::opaque_blob::{decrypt_payload, encrypt_inner_blob, InnerBlob, OpaqueBlob};
    use crate::sync::server::{EncryptedBlob, MockSyncServerClient, SyncClient};

    use std::sync::atomic::{AtomicU64, Ordering};

    /// Serialize integration tests because they mutate the process-global
    /// `DB_DIR` / `DB_POOL` singletons.
    static SYNC_TEST_MUTEX: Mutex<()> = Mutex::new(());
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn lock_test_mutex() -> std::sync::MutexGuard<'static, ()> {
        SYNC_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn init_test_db() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "relay_core_sync_integ_{}_{}",
            std::process::id(),
            TEST_COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        set_data_dir(dir.clone());
        dir
    }

    /// Helper: insert a highlight directly into the current global DB.
    fn insert_highlight(
        conn: &Connection,
        id: &str,
        text: &str,
        last_modified: &str,
        sync_status: &str,
    ) {
        conn.execute(
            "INSERT INTO highlights (id, text, source_url, source_title, source_author, summary, tags, connection_suggestion, created_at, last_modified, sync_status)
             VALUES (?1, ?2, NULL, NULL, NULL, '', '', NULL, ?3, ?4, ?5)",
            rusqlite::params![id, text, last_modified, last_modified, sync_status],
        )
        .unwrap();
    }

    /// Helper: build an encrypted blob for the mock server.
    fn make_blob(
        id: &str,
        record_type: &str,
        text: &str,
        last_modified: &str,
        key: &[u8; 32],
    ) -> EncryptedBlob {
        let payload = serde_json::json!({
            "id": id,
            "text": text,
            "source_url": null,
            "source_title": null,
            "source_author": null,
            "summary": "",
            "tags": "",
            "connection_suggestion": null,
            "created_at": last_modified,
            "last_modified": last_modified,
            "sync_status": "synced",
        });
        let ciphertext = encrypt(&payload.to_string(), key).unwrap();
        EncryptedBlob {
            id: id.to_string(),
            record_type: record_type.to_string(),
            ciphertext,
            last_modified: last_modified.to_string(),
        }
    }

    fn insert_credentials_v1(conn: &Connection) {
        conn.execute(
            "INSERT INTO sync_credentials (user_email, password_hash, salt_auth, encryption_key_salt, server_url, protocol_version)
             VALUES ('test@example.com', 'ph', 'sa', 'se', 'https://example.com', 1)
             ON CONFLICT(user_email) DO UPDATE SET protocol_version = excluded.protocol_version",
            [],
        )
        .unwrap();
    }

    fn insert_credentials_v2(conn: &Connection) {
        conn.execute(
            "INSERT INTO sync_credentials (user_email, password_hash, salt_auth, encryption_key_salt, server_url, protocol_version)
             VALUES ('test@example.com', 'ph', 'sa', 'se', 'https://example.com', 2)
             ON CONFLICT(user_email) DO UPDATE SET protocol_version = excluded.protocol_version",
            [],
        )
        .unwrap();
    }

    #[allow(dead_code)]
    fn make_v2_opaque_blob(
        id: &str,
        record_type: &str,
        text: &str,
        last_modified: i64,
        key: &[u8; 32],
    ) -> OpaqueBlob {
        let data = serde_json::json!({
            "text": text,
            "source_url": null,
            "source_title": null,
            "source_author": null,
            "summary": "",
            "tags": "",
            "connection_suggestion": null,
            "created_at": "2026-05-20T00:00:00Z",
            "sync_status": "synced",
        });
        let inner = InnerBlob {
            id: id.to_string(),
            record_type: record_type.to_string(),
            last_modified,
            data,
        };
        let payload = encrypt_inner_blob(key, &inner).unwrap();
        OpaqueBlob {
            blob_id: uuid::Uuid::new_v4().to_string(),
            payload,
        }
    }

    #[test]
    fn test_offline_capture_then_sync() {
        let _guard = lock_test_mutex();
        let _temp = init_test_db();

        let key = [42u8; 32];
        let mock = MockSyncServerClient::new();
        let engine = SyncEngine::new(Arc::new(mock.clone()), "token".to_string(), key);

        // 1. Simulate offline capture
        {
            let conn = open_db().unwrap();
            insert_highlight(
                &conn,
                "hl1",
                "offline highlight",
                "2026-05-20T00:00:00Z",
                "local",
            );
        }

        // 2. Push to mock server
        let pushed = engine.push().unwrap();
        assert_eq!(pushed, 1);

        // 3. Verify local row is now 'synced'
        {
            let conn = open_db().unwrap();
            let status: String = conn
                .query_row(
                    "SELECT sync_status FROM highlights WHERE id = 'hl1'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(status, "synced");
        }

        // 4. Verify server holds the encrypted blob
        let blobs = mock.pull("token", "1970-01-01T00:00:00Z").unwrap();
        assert_eq!(blobs.len(), 1);
        let decrypted = decrypt(&blobs[0].ciphertext, &key).unwrap();
        assert!(decrypted.contains("offline highlight"));
    }

    #[test]
    fn test_lww_conflict_local_newer() {
        let _guard = lock_test_mutex();
        let _temp = init_test_db();

        let key = [42u8; 32];
        let mock = MockSyncServerClient::new();
        let engine = SyncEngine::new(Arc::new(mock.clone()), "token".to_string(), key);

        // Device A: local row is newer
        {
            let conn = open_db().unwrap();
            insert_highlight(
                &conn,
                "hl1",
                "device A text",
                "2026-05-20T00:00:00Z",
                "synced",
            );
        }

        // Device B: push an *older* revision to the mock server
        let older_blob = make_blob(
            "hl1",
            "highlights",
            "device B text",
            "2026-05-19T00:00:00Z",
            &key,
        );
        mock.push("token", &[older_blob]).unwrap();

        // Local pulls
        let pulled = engine.pull().unwrap();
        assert_eq!(pulled, 1);

        // Local text must remain unchanged (local is newer)
        {
            let conn = open_db().unwrap();
            let text: String = conn
                .query_row("SELECT text FROM highlights WHERE id = 'hl1'", [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(text, "device A text");

            // Conflict must be logged
            let conflicts = list_conflicts(&conn).unwrap();
            assert_eq!(conflicts.len(), 1);
            assert_eq!(conflicts[0].record_type, "highlights");
            assert_eq!(conflicts[0].record_id, "hl1");
        }
    }

    #[test]
    fn test_lww_conflict_remote_newer() {
        let _guard = lock_test_mutex();
        let _temp = init_test_db();

        let key = [42u8; 32];
        let mock = MockSyncServerClient::new();
        let engine = SyncEngine::new(Arc::new(mock.clone()), "token".to_string(), key);

        // Device A: local row
        {
            let conn = open_db().unwrap();
            insert_highlight(
                &conn,
                "hl1",
                "device A text",
                "2026-05-20T00:00:00Z",
                "synced",
            );
        }

        // Device B: push a *newer* revision to the mock server
        let newer_blob = make_blob(
            "hl1",
            "highlights",
            "device B text",
            "2026-05-21T00:00:00Z",
            &key,
        );
        mock.push("token", &[newer_blob]).unwrap();

        // Local pulls
        let pulled = engine.pull().unwrap();
        assert_eq!(pulled, 1);

        // Remote is newer -> local must be overwritten
        {
            let conn = open_db().unwrap();
            let text: String = conn
                .query_row("SELECT text FROM highlights WHERE id = 'hl1'", [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(text, "device B text");

            // No conflict should be logged
            let conflicts = list_conflicts(&conn).unwrap();
            assert_eq!(conflicts.len(), 0);
        }
    }

    #[test]
    fn test_v2_push_pull_roundtrip() {
        let _guard = lock_test_mutex();
        let _temp = init_test_db();

        let key = [42u8; 32];
        let mock = MockSyncServerClient::new();
        insert_credentials_v2(&open_db().unwrap());
        let engine = SyncEngine::new(Arc::new(mock.clone()), "token".to_string(), key);

        // Insert 3 local highlights
        {
            let conn = open_db().unwrap();
            insert_highlight(&conn, "hl1", "first", "2026-05-20T10:00:00Z", "local");
            insert_highlight(&conn, "hl2", "second", "2026-05-20T11:00:00Z", "local");
            insert_highlight(&conn, "hl3", "third", "2026-05-20T12:00:00Z", "local");
        }

        // Push v2
        let pushed = engine.push().unwrap();
        assert_eq!(pushed, 3, "all 3 local rows pushed");

        // Verify local rows synced
        {
            let conn = open_db().unwrap();
            for id in ["hl1", "hl2", "hl3"] {
                let status: String = conn
                    .query_row(
                        &format!("SELECT sync_status FROM highlights WHERE id = '{id}'"),
                        [],
                        |row| row.get(0),
                    )
                    .unwrap();
                assert_eq!(status, "synced", "{id} should be synced");
            }
        }

        // Clear local DB and re-pull to simulate fresh device
        {
            let conn = open_db().unwrap();
            conn.execute("DELETE FROM highlights", []).unwrap();
        }

        let pulled = engine.pull().unwrap();
        assert_eq!(pulled, 3, "all 3 blobs pulled");

        // Verify content round-tripped
        {
            let conn = open_db().unwrap();
            let text1: String = conn
                .query_row("SELECT text FROM highlights WHERE id = 'hl1'", [], |row| {
                    row.get(0)
                })
                .unwrap();
            let text2: String = conn
                .query_row("SELECT text FROM highlights WHERE id = 'hl2'", [], |row| {
                    row.get(0)
                })
                .unwrap();
            let text3: String = conn
                .query_row("SELECT text FROM highlights WHERE id = 'hl3'", [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(text1, "first");
            assert_eq!(text2, "second");
            assert_eq!(text3, "third");
        }

        // No plaintext metadata in transit
        let v2_blobs = mock.pull_v2("token", "1970-01-01T00:00:00Z").unwrap();
        assert_eq!(v2_blobs.len(), 3);
        for blob in &v2_blobs {
            assert!(blob.blob_id != "hl1" && blob.blob_id != "hl2" && blob.blob_id != "hl3");
            // Payload must be opaque (cannot parse as JSON)
            assert!(
                serde_json::from_str::<serde_json::Value>(&blob.payload).is_err(),
                "payload must be opaque, not plaintext JSON"
            );
        }
    }

    #[test]
    fn test_v1_to_v2_migration() {
        let _guard = lock_test_mutex();
        let _temp = init_test_db();

        let key = [42u8; 32];
        let mock = MockSyncServerClient::new();
        insert_credentials_v1(&open_db().unwrap());
        let engine_v1 = SyncEngine::new(Arc::new(mock.clone()), "token".to_string(), key);

        // Insert a local v1 record and push it
        {
            let conn = open_db().unwrap();
            insert_highlight(&conn, "hl1", "v1 text", "2026-05-20T00:00:00Z", "local");
        }
        let pushed_v1 = engine_v1.push().unwrap();
        assert_eq!(pushed_v1, 1, "v1 push should succeed");

        // Verify v1 blob exists on server with plaintext metadata (legacy)
        let v1_blobs = mock.pull("token", "1970-01-01T00:00:00Z").unwrap();
        assert_eq!(v1_blobs.len(), 1, "v1 blob on server");
        assert_eq!(v1_blobs[0].id, "hl1", "v1 blob has plaintext id");
        assert_eq!(v1_blobs[0].record_type, "highlights", "v1 blob has plaintext record_type");

        // Switch to v2
        insert_credentials_v2(&open_db().unwrap());
        let engine_v2 = SyncEngine::new(Arc::new(mock.clone()), "token".to_string(), key);

        // Insert a NEW local record after v2 switch and push as v2
        {
            let conn = open_db().unwrap();
            insert_highlight(&conn, "hl2", "v2 text", "2026-05-21T00:00:00Z", "local");
        }
        let pushed_v2 = engine_v2.push().unwrap();
        assert_eq!(pushed_v2, 1, "v2 push should succeed");

        // Verify v2 blobs have no plaintext metadata
        let opaque_blobs = mock.pull_v2("token", "1970-01-01T00:00:00Z").unwrap();
        assert_eq!(opaque_blobs.len(), 1, "v2 blob on server");
        let outer = serde_json::json!({
            "blob_id": &opaque_blobs[0].blob_id,
            "payload": &opaque_blobs[0].payload,
        });
        let outer_str = outer.to_string();
        assert!(!outer_str.contains("\"id\":\"hl2"), "v2 outer must not contain record id");
        assert!(
            !outer_str.contains("\"record_type\":"),
            "v2 outer must not contain record_type"
        );
        assert!(
            !outer_str.contains("\"last_modified\":"),
            "v2 outer must not contain last_modified"
        );

        // Inner blob must contain metadata after decryption
        let inner = decrypt_payload(&key, &opaque_blobs[0].payload).unwrap();
        assert_eq!(inner.id, "hl2", "inner must have id");
        assert_eq!(inner.record_type, "highlights", "inner must have record_type");
        assert!(inner.last_modified > 0, "inner must have last_modified");

        // v2 pull must restore the v2 record correctly
        {
            let conn = open_db().unwrap();
            conn.execute("DELETE FROM highlights WHERE id = 'hl2'", []).unwrap();
        }
        let pulled = engine_v2.pull().unwrap();
        assert_eq!(pulled, 1, "v2 pull should restore the v2 blob");

        let text: String = open_db().unwrap()
            .query_row("SELECT text FROM highlights WHERE id = 'hl2'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(text, "v2 text");
    }

    #[test]
    fn test_dual_protocol_lww() {
        let _guard = lock_test_mutex();
        let _temp = init_test_db();

        let key = [42u8; 32];
        let mock = MockSyncServerClient::new();

        // Device A (v1) pushes a blob to the v1 store
        {
            let v1_data = serde_json::json!({
                "id": "hl1",
                "text": "device A v1",
                "source_url": null,
                "source_title": null,
                "source_author": null,
                "summary": "",
                "tags": "",
                "connection_suggestion": null,
                "created_at": "2026-05-22T00:00:00Z",
                "last_modified": "2026-05-22T00:00:00Z",
                "sync_status": "synced",
            });
            let ciphertext = encrypt(&v1_data.to_string(), &key).unwrap();
            let v1_blob = EncryptedBlob {
                id: "hl1".to_string(),
                record_type: "highlights".to_string(),
                ciphertext,
                last_modified: "2026-05-22T00:00:00Z".to_string(),
            };
            mock.push("token", &[v1_blob]).unwrap();
        }

        // Device B (v2) has older local revision
        {
            let conn = open_db().unwrap();
            insert_highlight(&conn, "hl1", "device B", "2026-05-20T00:00:00Z", "synced");
            insert_credentials_v2(&conn);
        }
        let engine_b = SyncEngine::new(Arc::new(mock.clone()), "token".to_string(), key);

        // v2 pull reads from v2 store, which is empty (v1 blob is in v1 store)
        // Server cannot convert v1→v2 because it lacks the decryption key
        let pulled = engine_b.pull().unwrap();
        assert_eq!(pulled, 0, "v2 client cannot see v1 store blobs");

        // Device B pushes its local data as v2
        {
            let conn = open_db().unwrap();
            conn.execute(
                "UPDATE highlights SET sync_status = 'local' WHERE id = 'hl1'",
                [],
            )
            .unwrap();
        }
        let pushed_v2 = engine_b.push().unwrap();
        assert_eq!(pushed_v2, 1, "v2 push succeeds");

        // v2 pull now sees the v2 blob
        let pulled_v2 = engine_b.pull().unwrap();
        assert_eq!(pulled_v2, 1, "v2 client sees its own v2 blob");

        // Local data should be the v2 content (same as what was pushed)
        {
            let conn = open_db().unwrap();
            let text: String = conn
                .query_row("SELECT text FROM highlights WHERE id = 'hl1'", [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(text, "device B");
        }
    }

    #[test]
    fn test_no_plaintext_metadata_in_transit() {
        let _guard = lock_test_mutex();
        let _temp = init_test_db();

        let key = [42u8; 32];
        let mock = MockSyncServerClient::new();
        insert_credentials_v2(&open_db().unwrap());
        let engine = SyncEngine::new(Arc::new(mock.clone()), "token".to_string(), key);

        // Insert local rows
        {
            let conn = open_db().unwrap();
            insert_highlight(&conn, "hl1", "alpha", "2026-05-20T00:00:00Z", "local");
            insert_highlight(&conn, "hl2", "beta", "2026-05-21T00:00:00Z", "local");
        }

        engine.push().unwrap();

        let opaque_blobs = mock.pull_v2("token", "1970-01-01T00:00:00Z").unwrap();
        assert_eq!(opaque_blobs.len(), 2);

        for blob in &opaque_blobs {
            // outer envelope must NOT contain id / record_type / last_modified
            let json = serde_json::json!({
                "blob_id": &blob.blob_id,
                "payload": &blob.payload,
            });
            let s = json.to_string();
            assert!(
                !s.contains("\"id\":\"hl"),
                "outer envelope must not contain record id"
            );
            assert!(
                !s.contains("\"record_type\":\"highlights"),
                "outer envelope must not contain record_type"
            );
            assert!(
                !s.contains("\"last_modified\":"),
                "outer envelope must not contain last_modified"
            );

            // Inner blob decrypt must contain metadata
            let inner = decrypt_payload(&key, &blob.payload).unwrap();
            assert!(
                inner.id.starts_with("hl"),
                "inner must contain id after decryption"
            );
            assert_eq!(inner.record_type, "highlights");
            assert!(inner.last_modified > 0);
        }
    }
}
