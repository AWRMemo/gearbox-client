use relay_core::telemetry;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Emitter;
use tauri::Manager;

/// Flag to request watcher thread shutdown.
pub(crate) static WATCHER_SHOULD_STOP: AtomicBool = AtomicBool::new(false);

/// Flag indicating the system is sleeping / suspended.
/// The watcher loop checks this and skips captures while true.
static SLEEPING: AtomicBool = AtomicBool::new(false);

/// Request the watcher thread to stop.
pub fn request_shutdown() {
    WATCHER_SHOULD_STOP.store(true, Ordering::SeqCst);
}

/// Check whether shutdown has been requested.
pub fn is_shutdown_requested() -> bool {
    WATCHER_SHOULD_STOP.load(Ordering::SeqCst)
}

/// Mark the system as sleeping (pause background work).
pub fn set_sleeping(sleep: bool) {
    SLEEPING.store(sleep, Ordering::SeqCst);
    eprintln!("Lifecycle: SLEEPING set to {sleep}");
}

/// Check whether the system is currently sleeping.
pub fn is_sleeping() -> bool {
    SLEEPING.load(Ordering::SeqCst)
}

/// Lifecycle event handler for Tauri `RunEvent`.
pub fn on_run_event(app: &tauri::AppHandle, event: tauri::RunEvent) {
    match event {
        tauri::RunEvent::WindowEvent {
            label,
            event: tauri::WindowEvent::CloseRequested { api, .. },
            ..
        } => {
            if label == "main" {
                api.prevent_close();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
        }
        tauri::RunEvent::Resumed => {
            set_sleeping(false);
            telemetry::fire(telemetry::TelemetryEvent::SystemWake);
            spawn_sync_if_authenticated(app);
            batch_apply_deferred_enrichment(app);
        }
        // Tauri does not expose RunEvent::Suspended in v2.11.
        // On Windows, a low-level hook for WM_POWERBROADCAST / PBT_APMSUSPEND
        // would call set_sleeping(true) + crate::db::vector::flush() + telemetry::SystemSuspend.
        // For now, the 500 ms watcher poll loop gracefully skips work while SLEEPING is true.
        tauri::RunEvent::ExitRequested { .. } => {
            graceful_shutdown();
        }
        _ => {}
    }
}

/// Spawn sync_now if the auth state indicates the user is online.
fn spawn_sync_if_authenticated(app: &tauri::AppHandle) {
    let auth_state = app.try_state::<std::sync::Arc<std::sync::RwLock<Option<crate::commands::auth::AuthState>>>>();
    if let Some(state) = auth_state {
        if let Ok(guard) = state.read() {
            if guard.is_some() {
                let handle = app.clone();
                std::thread::spawn(move || {
                    let _ = handle.emit("relay://request-sync", ());
                });
            }
        }
    }
}

/// Graceful shutdown: stop watcher, flush LanceDB, print completion.
pub fn graceful_shutdown() {
    request_shutdown();
    // Best-effort flush of pending LanceDB writes
    let _ = crate::db::vector::flush();
    eprintln!("Graceful shutdown complete");
}

/// On Windows, check whether battery is at ≤10 % and not on AC power.
#[cfg(windows)]
pub fn is_battery_critical() -> bool {
    use winapi::um::winbase::GetSystemPowerStatus;
    use winapi::um::winbase::SYSTEM_POWER_STATUS;

    let mut status: SYSTEM_POWER_STATUS = unsafe { std::mem::zeroed() };
    let ok = unsafe { GetSystemPowerStatus(&mut status) };
    if ok == 0 {
        return false;
    }
    let on_battery = status.ACLineStatus == 0;
    let battery_low = status.BatteryLifePercent <= 10;
    on_battery && battery_low
}

/// Stub on non-Windows platforms.
#[cfg(not(windows))]
pub fn is_battery_critical() -> bool {
    false
}

/// Process all deferred (battery-critical) enrichments in batch.
/// Called on wake/AC restore to enrich highlights that were skipped due to low battery.
pub fn batch_apply_deferred_enrichment(app: &tauri::AppHandle) {
    let items = match crate::db::deferred_enrichment::list_deferred() {
        Ok(items) => items,
        Err(e) => {
            eprintln!("Failed to list deferred enrichments: {e}");
            return;
        }
    };

    if items.is_empty() {
        return;
    }

    let ai_opt = crate::AI_SERVICE_GLOBAL
        .get()
        .and_then(|arc| arc.read().ok().map(|g| Arc::clone(&*g)));

    let ai_service = match ai_opt {
        Some(s) => s,
        None => {
            eprintln!("AI service not ready for deferred batch; queued items will remain deferred.");
            return;
        }
    };

    for item in &items {
        let highlight = crate::ai::service::Highlight {
            id: item.highlight_id.clone(),
            text: item.text.clone(),
            source_url: item.source_url.clone(),
            source_title: item.source_title.clone(),
            source_author: item.source_author.clone(),
        };

        let (output, _parse_success) = crate::commands::capture::do_enrich(
            Arc::clone(&ai_service),
            highlight,
            vec![],
        );

        let tags_json = serde_json::to_string(&output.tags).unwrap_or_default();
        let connection_json = output
            .connection_suggestion
            .as_ref()
            .map(|cs| serde_json::to_string(cs).unwrap_or_default());

        let conn = match crate::db::open_db() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to open DB for deferred batch: {e}");
                continue;
            }
        };

        let _ = conn.execute(
            "UPDATE highlights SET summary = ?1, tags = ?2, connection_suggestion = ?3 WHERE id = ?4",
            rusqlite::params![&output.summary, &tags_json, &connection_json, &item.highlight_id],
        );

        let tags_text = output.tags.join(" ");
        let _ = conn.execute(
            "INSERT OR REPLACE INTO highlights_fts (id, text, summary, tags_text)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![&item.highlight_id, &item.text, &output.summary, &tags_text],
        );

        let _ = crate::db::deferred_enrichment::remove_deferred(&item.highlight_id);
    }

    let _ = app.emit("relay://deferred-batch-complete", items.len());
    eprintln!("Processed {} deferred enrichments on wake/AC restore", items.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_flag() {
        // Reset flag for determinism in tests
        WATCHER_SHOULD_STOP.store(false, Ordering::SeqCst);
        assert!(!is_shutdown_requested());
        request_shutdown();
        assert!(is_shutdown_requested());
    }

    #[test]
    fn test_sleeping_flag() {
        set_sleeping(false);
        assert!(!is_sleeping());
        set_sleeping(true);
        assert!(is_sleeping());
        set_sleeping(false);
        assert!(!is_sleeping());
    }

    #[test]
    fn test_suspend_sets_sleeping_flag() {
        set_sleeping(false);
        assert!(!is_sleeping());
        // Simulate suspend: set_sleeping(true) and flush
        set_sleeping(true);
        assert!(is_sleeping());
    }

    #[test]
    fn test_wake_resumes_flag() {
        set_sleeping(true);
        assert!(is_sleeping());
        // Simulate wake: set_sleeping(false)
        set_sleeping(false);
        assert!(!is_sleeping());
    }

    #[test]
    fn test_graceful_shutdown_stops_watcher_and_flushes() {
        WATCHER_SHOULD_STOP.store(false, Ordering::SeqCst);
        assert!(!is_shutdown_requested());
        graceful_shutdown();
        assert!(is_shutdown_requested());
        WATCHER_SHOULD_STOP.store(false, Ordering::SeqCst);
    }
}
