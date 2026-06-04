use std::time::SystemTime;

use native_dialog::MessageDialog;
use native_dialog::MessageType;

use crate::config;
use crate::export;

/// Export highlights as a ZIP containing highlights.json + highlights.md.
/// Optional date_from and date_to filter (ISO 8601 strings, e.g. "2026-01-01").
/// Returns the full path to the created ZIP.
#[tauri::command]
pub fn export_local_data(
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<String, String> {
    let user_id = config::get_device_id()?;
    let trigger = relay_core::db::tiers::check_paywall_trigger(user_id)?;
    if trigger.is_blocked {
        return Err(trigger.reason.unwrap_or_else(|| "paywall_blocked".to_string()));
    }

    let app_dir = config::get_app_dir()?;
    let backup_dir = app_dir.join("exports");
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create export directory: {e}"))?;

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| format!("System time error: {e}"))?
        .as_secs();

    let filter = if date_from.is_some() || date_to.is_some() {
        Some(export::ExportFilter { date_from, date_to })
    } else {
        None
    };

    let highlights = export::query_highlights(filter.as_ref())?;
    let zip_path = backup_dir.join(format!("relay_export_{}.zip", timestamp));

    export::generate_export_zip(&zip_path, &highlights)?;

    Ok(zip_path.to_string_lossy().to_string())
}

/// Clear all local data after user confirmation.
/// Preserves device_id.txt. Deletes relay.db, vectors.lance, and .flag files.
#[tauri::command]
pub fn clear_local_data() -> Result<(), String> {
    let confirmed = MessageDialog::new()
        .set_type(MessageType::Warning)
        .set_title("Clear All Local Data")
        .set_text("This will permanently delete all your highlights, streams, and analytics.\n\nAre you sure?")
        .show_confirm()
        .map_err(|e| format!("Dialog error: {e}"))?;

    if !confirmed {
        return Err("User cancelled deletion.".to_string());
    }

    let app_dir = config::get_app_dir()?;

    let db_path = app_dir.join("relay.db");
    if db_path.exists() {
        std::fs::remove_file(&db_path).map_err(|e| format!("Failed to remove relay.db: {e}"))?;
    }

    let vectors_dir = app_dir.join("vectors.lance");
    if vectors_dir.exists() {
        std::fs::remove_dir_all(&vectors_dir)
            .map_err(|e| format!("Failed to remove vectors.lance: {e}"))?;
    }

    for entry in std::fs::read_dir(app_dir.clone()).map_err(|e| format!("Failed to read app dir: {e}"))? {
        let entry = entry.map_err(|e| format!("Dir entry error: {e}"))?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with(".flag") {
            let _ = std::fs::remove_file(entry.path());
        }
    }

    Ok(())
}
