use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static DEVICE_ID: OnceLock<String> = OnceLock::new();
static APP_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn init(data_dir: &Path) -> Result<bool, String> {
    APP_DIR
        .set(data_dir.to_path_buf())
        .map_err(|_| "APP_DIR already initialized".to_string())?;

    let id_path = data_dir.join("device_id.txt");
    let is_first_run = !id_path.exists();

    let id = if is_first_run {
        let id = uuid::Uuid::new_v4().to_string();
        std::fs::write(&id_path, &id).map_err(|e| format!("Failed to write device ID: {e}"))?;
        id
    } else {
        std::fs::read_to_string(&id_path)
            .map_err(|e| format!("Failed to read device ID: {e}"))?
            .trim()
            .to_string()
    };
    DEVICE_ID
        .set(id)
        .map_err(|_| "DEVICE_ID already initialized".to_string())?;
    Ok(is_first_run)
}

pub fn get_device_id() -> Result<&'static str, String> {
    DEVICE_ID
        .get()
        .map(|s| s.as_str())
        .ok_or_else(|| "DEVICE_ID not initialized. Call config::init() first.".to_string())
}

pub fn get_app_dir() -> Result<&'static PathBuf, String> {
    APP_DIR
        .get()
        .ok_or_else(|| "APP_DIR not initialized".to_string())
}

/// Return true if this is the first highlight capture for this device.
/// Writes a sentinel file on first call.
pub fn is_first_highlight_capture() -> Result<bool, String> {
    let app_dir = get_app_dir()?;
    let sentinel = app_dir.join("first_highlight_captured.flag");
    if sentinel.exists() {
        return Ok(false);
    }
    std::fs::write(&sentinel, b"1").map_err(|e| format!("Failed to write sentinel: {e}"))?;
    Ok(true)
}
