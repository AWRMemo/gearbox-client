use crate::commands::auth::AuthState;
use crate::db::open_db;
use crate::sync::conflict::{list_conflicts, resolve_conflict_with_action, Conflict};
use crate::sync::engine::{SyncEngine, SyncReport};
use crate::sync::queue::OfflineSyncQueue;
use crate::sync::server::SyncServerClient;
use serde::Serialize;
use std::sync::{Arc, RwLock};

#[derive(Serialize)]
pub struct SyncStatus {
    pub last_sync: Option<String>,
    pub status: String,
    pub pending_conflicts: usize,
}

#[derive(Serialize)]
pub struct SyncPaywallCheck {
    pub is_blocked: bool,
    pub reason: Option<String>,
}

#[tauri::command]
pub fn check_sync_paywall() -> Result<SyncPaywallCheck, String> {
    let user_id = crate::config::get_device_id()?;
    let conn = crate::db::open_db()?;
    let device_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sync_credentials",
            [],
            |row| row.get(0),
        )
        .unwrap_or(1);
    if device_count > 1 {
        let trigger = relay_core::db::tiers::check_paywall_trigger(user_id)?;
        if trigger.is_blocked {
            return Ok(SyncPaywallCheck {
                is_blocked: true,
                reason: trigger.reason,
            });
        }
    }
    Ok(SyncPaywallCheck {
        is_blocked: false,
        reason: None,
    })
}

fn is_network_error(err: &str) -> bool {
    let lower = err.to_lowercase();
    lower.contains("http")
        || lower.contains("network")
        || lower.contains("connection")
        || lower.contains("request")
        || lower.contains("timeout")
        || lower.contains("dns")
}

#[tauri::command]
pub fn sync_now(
    state: tauri::State<Arc<RwLock<Option<AuthState>>>>,
    queue: tauri::State<Arc<OfflineSyncQueue>>,
) -> Result<SyncReport, String> {
    let auth = {
        let guard = state.read().map_err(|e| format!("Lock poisoned: {e}"))?;
        guard.as_ref().ok_or("Not authenticated")?.clone()
    };
    let client = Arc::new(SyncServerClient::new(auth.server_url.clone()));
    let engine = SyncEngine::new(client, auth.jwt, auth.encryption_key);
    match engine.sync_now() {
        Ok(report) => Ok(report),
        Err(e) => {
            if is_network_error(&e) {
                queue.enqueue();
                Err("Sync queued for retry".to_string())
            } else {
                Err(e)
            }
        }
    }
}

#[tauri::command]
pub fn get_sync_status() -> Result<SyncStatus, String> {
    let conn = open_db()?;
    let last_sync: Option<String> = conn
        .query_row(
            "SELECT value FROM sync_metadata WHERE key = 'last_sync_timestamp'",
            [],
            |row| row.get(0),
        )
        .ok();

    let pending_conflicts: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM sync_conflicts WHERE resolved_at IS NULL",
            [],
            |row| row.get::<_, usize>(0),
        )
        .unwrap_or(0);

    let status = if last_sync.is_some() {
        "active".to_string()
    } else {
        "never".to_string()
    };

    Ok(SyncStatus {
        last_sync,
        status,
        pending_conflicts,
    })
}

#[tauri::command]
pub fn get_conflicts() -> Result<Vec<Conflict>, String> {
    let conn = open_db()?;
    list_conflicts(&conn)
}

#[tauri::command]
pub fn resolve_conflict(id: String, resolution: String) -> Result<(), String> {
    let conn = open_db()?;
    resolve_conflict_with_action(&conn, &id, &resolution)
}

#[tauri::command]
pub fn get_telemetry_enabled() -> Result<bool, String> {
    let conn = open_db()?;
    let val: Option<String> = conn
        .query_row(
            "SELECT value FROM sync_metadata WHERE key = 'telemetry_enabled'",
            [],
            |row| row.get(0),
        )
        .ok();
    Ok(val.as_deref() == Some("true"))
}

#[tauri::command]
pub fn set_telemetry_enabled(enabled: bool) -> Result<(), String> {
    let conn = open_db()?;
    let val = if enabled { "true" } else { "false" };
    conn.execute(
        "INSERT INTO sync_metadata (key, value) VALUES ('telemetry_enabled', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [val],
    )
    .map_err(|e| format!("Failed to set telemetry: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn get_auto_capture_enabled() -> Result<bool, String> {
    let conn = open_db()?;
    let val: Option<String> = conn
        .query_row(
            "SELECT value FROM sync_metadata WHERE key = 'auto_capture_enabled'",
            [],
            |row| row.get(0),
        )
        .ok();
    Ok(val.as_deref() != Some("false"))
}

#[tauri::command]
pub fn set_auto_capture_enabled(enabled: bool) -> Result<(), String> {
    let conn = open_db()?;
    let val = if enabled { "true" } else { "false" };
    conn.execute(
        "INSERT INTO sync_metadata (key, value) VALUES ('auto_capture_enabled', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [val],
    )
    .map_err(|e| format!("Failed to set auto_capture: {e}"))?;
    Ok(())
}
