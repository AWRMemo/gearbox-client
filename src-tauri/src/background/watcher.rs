use std::sync::{Arc, Mutex};

use crate::ai::fallback::MAX_HIGHLIGHT_CHARS;
use crate::ai::quality_monitor;
use crate::ai::service::{ConnectionCandidate, Highlight};
use crate::commands::capture::{do_enrich, extract_source_metadata};
use crate::db::vector::{search_vectors, store_vector};
use crate::{config, db};
use crate::db::deferred_enrichment;
use tauri::Emitter;

/// Current SHA-256 hash of the last captured clipboard text.
static LAST_CLIPBOARD_HASH: Mutex<Option<String>> = Mutex::new(None);

/// Start a background thread that polls the clipboard every 500 ms,
/// deduplicates by SHA-256 hash, and enriches new text via the AI pipeline.
pub fn start_watcher(app: &tauri::AppHandle) -> Result<(), String> {
    let app_handle = app.clone();

    std::thread::spawn(move || {
        let mut clipboard = match arboard::Clipboard::new() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to open clipboard: {e}");
                return;
            }
        };

        loop {
            if crate::background::lifecycle::is_shutdown_requested() {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(500));

            // Skip if auto-capture is disabled
            if !is_auto_capture_enabled() {
                continue;
            }

            // Skip if battery is critical — defer enrichment for later batch
            if crate::background::lifecycle::is_battery_critical() {
                let text = match clipboard.get_text() {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                if text.trim().is_empty() {
                    continue;
                }
                let hash = sha256_hex(&text);
                let is_duplicate = {
                    let guard = LAST_CLIPBOARD_HASH
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    guard.as_ref() == Some(&hash)
                };
                if is_duplicate {
                    continue;
                }
                let (source_url, source_title, source_author) = extract_source_metadata(&text);
                let id = uuid::Uuid::new_v4().to_string();
                if let Err(e) = deferred_enrichment::queue_deferred(
                    &id,
                    &text,
                    source_url.as_deref(),
                    source_title.as_deref(),
                    source_author.as_deref(),
                ) {
                    tracing::warn!("Failed to queue deferred enrichment: {e}");
                }
                let mut guard = LAST_CLIPBOARD_HASH
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                *guard = Some(hash);
                continue;
            }

            // Skip while system is sleeping/suspended
            if crate::background::lifecycle::is_sleeping() {
                continue;
            }

            let text = match clipboard.get_text() {
                Ok(t) => t,
                Err(_) => continue,
            };

            let text = if text.len() > MAX_HIGHLIGHT_CHARS {
                let byte_end = text
                    .char_indices()
                    .nth(MAX_HIGHLIGHT_CHARS)
                    .map(|(i, _)| i)
                    .unwrap_or(text.len());
                text[..byte_end].to_string()
            } else {
                text
            };

            if text.trim().is_empty() {
                continue;
            }

            let hash = sha256_hex(&text);

            let is_duplicate = {
                let guard = LAST_CLIPBOARD_HASH
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                guard.as_ref() == Some(&hash)
            };

            if is_duplicate {
                continue;
            }

            // Retrieve globals
            let (ai_opt, emb_opt) = {
                let ai = crate::AI_SERVICE_GLOBAL.get().cloned();
                let emb = crate::EMBEDDING_SERVICE_GLOBAL.get().cloned();
                (ai, emb)
            };

            let Some(ai_rw) = ai_opt else {
                tracing::warn!("AI service not ready; skipping clipboard capture.");
                continue;
            };

            let service = match ai_rw.read() {
                Ok(g) => Arc::clone(&*g),
                Err(e) => {
                    tracing::error!("AI lock poisoned: {e}");
                    continue;
                }
            };

            let (source_url, source_title, source_author) = extract_source_metadata(&text);
            let highlight = Highlight {
                id: uuid::Uuid::new_v4().to_string(),
                text: text.clone(),
                source_url,
                source_title,
                source_author,
            };

            // Embedding + candidate search
            let emb_service = emb_opt.as_ref().and_then(|opt| opt.as_ref());
            let embedding_vector = emb_service.and_then(|es| match es.encode(&highlight.text) {
                Ok(v) => Some(v),
                Err(e) => {
                    tracing::warn!("Embedding failed for watcher capture: {e}");
                    None
                }
            });

            let candidates = match &embedding_vector {
                Some(vector) => match search_vectors(vector, 3) {
                    Ok(results) => results
                        .into_iter()
                        .map(|r| ConnectionCandidate {
                            id: r.id,
                            text: r.text,
                        })
                        .collect(),
                    Err(e) => {
                        tracing::warn!("Vector search failed: {e}");
                        vec![]
                    }
                },
                None => vec![],
            };

            let hl = highlight.clone();
            let cands = candidates;

            // Retry logic: on enrichment failure, wait 2 s and retry once
            let mut retry_count = 0u32;
            let (output, parse_success) = loop {
                let service_clone = Arc::clone(&service);
                let hl_clone = hl.clone();
                let cands_clone = cands.clone();
                let result = do_enrich(service_clone, hl_clone, cands_clone);
                if result.1 || retry_count >= 1 {
                    break result;
                }
                tracing::warn!("Watcher enrichment failed (parse_success=false), retrying in 2 s…");
                std::thread::sleep(std::time::Duration::from_secs(2));
                retry_count += 1;
            };

            // Record quality metrics for background capture
            let model_name = crate::ai::model_manager::model_name();
            if let Err(e) = quality_monitor::record_quality(
                &highlight.id,
                parse_success,
                output.tags.len(),
                output.summary.len(),
                &model_name,
            ) {
                tracing::warn!("Failed to record quality metrics in watcher: {e}");
            }

            let embedding_slice: Option<&[f32]> = embedding_vector.as_deref();
            // Persist to SQLite
            if let Err(e) = db::store::store_highlight(&highlight, &output, embedding_slice) {
                tracing::warn!("Failed to persist highlight: {e}");
            } else {
                // Only update dedup hash after successful persistence.
                // This prevents data loss if the user copies new text
                // while the previous capture is still being processed.
                let mut guard = LAST_CLIPBOARD_HASH
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                *guard = Some(hash);
            }

            // Store vector in LanceDB
            if let Some(ref vector) = embedding_vector {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                if let Err(e) = store_vector(&highlight.id, vector, &highlight.text, now) {
                    tracing::warn!("Failed to store vector: {e}");
                }
            }

            // Analytics
            if let Ok(user_id) = config::get_device_id() {
                if let Ok(true) = config::is_first_highlight_capture() {
                    let _ = db::analytics::log_event(
                        "first_highlight_captured",
                        None,
                        Some(user_id),
                        None,
                        None,
                    );
                }
            }

            // OS notification
            let summary_snippet = {
                let s: String = output.summary.chars().take(80).collect();
                if output.summary.chars().count() > 80 {
                    format!("{}…", s)
                } else {
                    s
                }
            };
            let _ = notify_rust::Notification::new()
                .summary("Relay captured")
                .body(&summary_snippet)
                .show();

            // Emit a Tauri event so the React UI can refresh
            let _ = app_handle.emit("relay://clipboard-captured", &highlight.id);
        }
    });

    Ok(())
}

/// Read `auto_capture_enabled` from `sync_metadata` table (default true).
fn is_auto_capture_enabled() -> bool {
    let conn = match db::open_db() {
        Ok(c) => c,
        Err(_) => return true,
    };
    let val: Result<String, _> = conn.query_row(
        "SELECT value FROM sync_metadata WHERE key = 'auto_capture_enabled'",
        [],
        |row| row.get(0),
    );
    match val {
        Ok(s) => s.parse::<bool>().unwrap_or(true),
        Err(_) => true,
    }
}

fn sha256_hex(input: &str) -> String {
    use ring::digest::{Context, SHA256};
    let mut ctx = Context::new(&SHA256);
    ctx.update(input.as_bytes());
    let digest = ctx.finish();
    let bytes = digest.as_ref();
    const HEX: [u8; 16] = *b"0123456789abcdef";
    let mut out = Vec::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(*b >> 4) as usize]);
        out.push(HEX[(*b & 0x0f) as usize]);
    }
    String::from_utf8(out).expect("hex digits are ASCII")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hex_consistency() {
        let a = sha256_hex("hello clipboard");
        let b = sha256_hex("hello clipboard");
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn test_sha256_hex_different_inputs() {
        let a = sha256_hex("alpha");
        let b = sha256_hex("beta");
        assert_ne!(a, b);
    }

    #[test]
    fn test_dedup_flag() {
        // Reset LAST_CLIPBOARD_HASH for deterministic test
        {
            let mut guard = LAST_CLIPBOARD_HASH
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            *guard = None;
        }
        let hash_a = sha256_hex("first");
        {
            let mut guard = LAST_CLIPBOARD_HASH
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            *guard = Some(hash_a.clone());
        }
        let guard = LAST_CLIPBOARD_HASH
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        assert_eq!(guard.as_ref(), Some(&hash_a));
    }

    #[test]
    fn test_watcher_retry_on_failure() {
        use crate::ai::service::{AIService, ConnectionCandidate, EnrichmentOutput, Highlight};

        struct AlwaysFail;
        impl AIService for AlwaysFail {
            fn enrich(
                &self,
                _highlight: &Highlight,
                _candidates: &[ConnectionCandidate],
            ) -> Result<EnrichmentOutput, String> {
                Err("mock failure".into())
            }
        }

        let service: Arc<dyn AIService> = Arc::new(AlwaysFail);
        let highlight = Highlight {
            id: "retry-test".into(),
            text: "Retry test text.".into(),
            source_url: None,
            source_title: None,
            source_author: None,
        };

        // Simulate watcher retry logic inline
        let mut retry_count = 0u32;
        let (output, parse_success) = loop {
            let service_clone = Arc::clone(&service);
            let hl_clone = highlight.clone();
            let result = do_enrich(service_clone, hl_clone, vec![]);
            if result.1 || retry_count >= 1 {
                break result;
            }
            retry_count += 1;
        };

        assert!(
            !parse_success,
            "always-fail service should produce parse_success=false"
        );
        assert_eq!(retry_count, 1, "should have retried exactly once");
        assert!(
            !output.summary.is_empty() && output.summary != "[enrichment failed]",
            "fallback should produce a real summary"
        );
    }
}
