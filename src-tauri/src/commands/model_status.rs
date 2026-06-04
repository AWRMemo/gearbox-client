use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::Manager;

/// Whether the on-device SLM is ready or the app is running on fallback.
#[derive(Debug, Clone, Serialize)]
pub struct ModelStatus {
    pub loaded: bool,
    pub model_name: Option<String>,
    pub embedding_available: bool,
    /// Download progress: 0–100 if a download is in flight, null otherwise.
    pub download_progress: Option<u32>,
    /// Current download state: "idle", "downloading", "done", "error".
    pub download_state: String,
}

/// Shared atomic flag updated by the background model-loading thread.
pub static MODEL_LOADED: AtomicBool = AtomicBool::new(false);

/// Shared atomic flag set when the embedding service fails to initialize.
pub static EMBEDDING_UNAVAILABLE: AtomicBool = AtomicBool::new(false);

/// Return the current model status to the frontend.
#[tauri::command]
pub fn get_model_status() -> Result<ModelStatus, String> {
    let loaded = MODEL_LOADED.load(Ordering::SeqCst);
    let embedding_available = !EMBEDDING_UNAVAILABLE.load(Ordering::SeqCst);

    // Map download state atomics into human-readable status
    let (progress, state) = {
        use crate::ai::model_manager::{
            MODEL_DOWNLOAD_STATE, MODEL_DOWNLOAD_TOTAL, MODEL_DOWNLOAD_PROGRESS,
        };
        let state_raw = MODEL_DOWNLOAD_STATE.load(Ordering::SeqCst);
        let total = MODEL_DOWNLOAD_TOTAL.load(Ordering::SeqCst);
        let downloaded = MODEL_DOWNLOAD_PROGRESS.load(Ordering::SeqCst);
        match state_raw {
            1 => {
                let pct = if total > 0 {
                    ((downloaded as f64 / total as f64) * 100.0) as u32
                } else {
                    0
                };
                (Some(pct), "downloading".to_string())
            }
            2 => (None, "done".to_string()),
            3 => (None, "error".to_string()),
            _ => (None, "idle".to_string()),
        }
    };

    Ok(ModelStatus {
        loaded,
        model_name: if loaded {
            Some(crate::ai::model_manager::model_name())
        } else {
            None
        },
        embedding_available,
        download_progress: progress,
        download_state: state,
    })
}

/// Trigger a re-download of the embedding model files.
/// This removes any existing cached ONNX + tokenizer and re-runs
/// `ensure_embedding_model`, returning the path to the model on success.
#[tauri::command]
pub fn re_download_embedding_model(app: tauri::AppHandle) -> Result<String, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {e}"))?;
    let models_dir = app_dir.join("models");

    // Remove existing cached files to force re-download
    let onnx_path = models_dir.join("all-MiniLM-L6-v2.onnx");
    let tokenizer_path = models_dir.join("tokenizer.json");
    if onnx_path.exists() {
        let _ = std::fs::remove_file(&onnx_path);
    }
    if tokenizer_path.exists() {
        let _ = std::fs::remove_file(&tokenizer_path);
    }

    let (model_path, _tokenizer_path) =
        crate::ai::embedding_model_manager::ensure_embedding_model(&app_dir)?;
    Ok(model_path.to_string_lossy().to_string())
}
