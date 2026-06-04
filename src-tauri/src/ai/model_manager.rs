use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Shared atomic: bytes downloaded so far (for progress reporting).
pub static MODEL_DOWNLOAD_PROGRESS: AtomicU64 = AtomicU64::new(0);
/// Shared atomic: total bytes expected.
pub static MODEL_DOWNLOAD_TOTAL: AtomicU64 = AtomicU64::new(0);
/// Shared atomic: current download state. 0=idle,1=downloading,2=done,3=error
pub static MODEL_DOWNLOAD_STATE: AtomicU64 = AtomicU64::new(0);

use ring::digest::{Context, SHA256};
use serde::Deserialize;
use std::sync::OnceLock;

/// Global manifest — set once at startup by main.rs, read by commands.
static MANIFEST: OnceLock<ModelManifest> = OnceLock::new();

/// Set the global manifest. Called once at startup.
pub fn init_manifest(manifest: ModelManifest) {
    MANIFEST.set(manifest).ok();
}

/// Read the global manifest. Panics if init_manifest was not called first.
fn manifest() -> &'static ModelManifest {
    MANIFEST.get().expect("model_manager::init_manifest not called")
}

/// Compiled-in fallback — updated on release.
const EMBEDDED_MANIFEST: &str = include_str!("../../models/manifest.json");

#[derive(Debug, Clone, Deserialize)]
pub struct ModelManifest {
    #[serde(default)]
    #[allow(dead_code)]
    version: u32,
    default: String,
    models: std::collections::HashMap<String, ModelEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelEntry {
    pub display_name: String,
    #[serde(default)]
    pub family: String,
    pub url: String,
    pub filename: String,
    pub sha256: String,
    #[serde(default)]
    pub size_bytes: u64,
    #[serde(default)]
    pub min_ram_mb: u32,
    #[serde(default)]
    pub architecture: String,
    #[serde(default)]
    pub chat_template: String,
    #[serde(default)]
    pub context_length: u32,
    #[serde(default)]
    pub added: String,
}

/// Resolve the manifest from a runtime file, falling back to embedded.
pub fn load_manifest(resource_dir: Option<&Path>, app_dir: &Path) -> ModelManifest {
    fn try_read(path: &Path) -> Option<ModelManifest> {
        let raw = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&raw).ok()
    }

    if let Some(res_dir) = resource_dir {
        if let Some(m) = try_read(&res_dir.join("models").join("manifest.json")) {
            return m;
        }
    }

    if let Some(m) = try_read(&app_dir.join("models").join("manifest.json")) {
        return m;
    }

    serde_json::from_str(EMBEDDED_MANIFEST).unwrap_or_else(|_| {
        panic!("embedded model manifest is invalid — check src-tauri/models/manifest.json");
    })
}

/// Resolve the default model entry from the manifest.
pub fn default_entry(manifest: &ModelManifest) -> &ModelEntry {
    manifest
        .models
        .get(&manifest.default)
        .unwrap_or_else(|| panic!("default model '{}' not found in manifest", manifest.default))
}

/// Return the human-readable name of the currently active model.
pub fn model_display_name(manifest: &ModelManifest) -> String {
    default_entry(manifest).display_name.clone()
}

/// Global getter — for callers that don't hold a manifest ref (commands).
pub fn model_name() -> String {
    model_display_name(manifest())
}

/// Return the model key (e.g. "qwen-3.5-0.8b") of the active model.
pub fn active_model_key(manifest: &ModelManifest) -> &str {
    &manifest.default
}

/// Return the resolved path for the active model file.
pub fn model_path(app_dir: &Path, manifest: &ModelManifest) -> PathBuf {
    let entry = default_entry(manifest);
    app_dir.join("models").join(&entry.filename)
}

/// Global getter for callers that don't hold a manifest ref.
pub fn model_path_global(app_dir: &Path) -> PathBuf {
    model_path(app_dir, manifest())
}

/// Returns true if the active model file exists and its SHA-256 matches.
pub fn is_model_cached(app_dir: &Path, manifest: &ModelManifest) -> bool {
    let entry = default_entry(manifest);
    let path = app_dir.join("models").join(&entry.filename);
    if !path.exists() {
        return false;
    }
    verify_sha256(&path, &entry.sha256).is_ok()
}

/// Global getter.
pub fn is_model_cached_global(app_dir: &Path) -> bool {
    is_model_cached(app_dir, manifest())
}

/// Ensure the active model is downloaded and verified.
pub fn ensure_model(app_dir: &Path, manifest: &ModelManifest) -> Result<PathBuf, String> {
    let entry = default_entry(manifest).clone();
    let models_dir = app_dir.join("models");
    std::fs::create_dir_all(&models_dir)
        .map_err(|e| format!("Failed to create models directory: {e}"))?;

    clean_stale_downloads(&models_dir);

    let path = models_dir.join(&entry.filename);

    if path.exists() {
        verify_sha256(&path, &entry.sha256)?;
        return Ok(path);
    }

    MODEL_DOWNLOAD_STATE.store(1, Ordering::SeqCst);
    let result = download_model(&models_dir, &entry);
    MODEL_DOWNLOAD_STATE.store(2, Ordering::SeqCst);

    if let Err(ref e) = result {
        MODEL_DOWNLOAD_STATE.store(3, Ordering::SeqCst);
        let _ = std::fs::remove_file(&path);
        eprintln!("Model download failed: {e}");
        relay_core::telemetry::fire(relay_core::telemetry::TelemetryEvent::ModelDownloadFailed {
            reason: e.clone(),
        });
    }

    result.map(|_| path.clone())?;
    verify_sha256(&path, &entry.sha256)?;

    relay_core::telemetry::fire(relay_core::telemetry::TelemetryEvent::ModelDownloadSuccess);

    if path.exists() {
        Ok(path)
    } else {
        Err("Model download completed but file not found.".to_string())
    }
}

/// Cancel an in-flight download by removing the temp file.
pub fn cancel_download(app_dir: &Path, manifest: &ModelManifest) {
    let entry = default_entry(manifest);
    let models_dir = app_dir.join("models");
    let temp_path = models_dir.join(format!("{}.downloading", entry.filename));
    if temp_path.exists() {
        let _ = std::fs::remove_file(&temp_path);
    }
    MODEL_DOWNLOAD_STATE.store(0, Ordering::SeqCst);
    MODEL_DOWNLOAD_PROGRESS.store(0, Ordering::SeqCst);
}

fn partial_size(temp_path: &Path) -> u64 {
    std::fs::metadata(temp_path)
        .map(|m| m.len())
        .unwrap_or(0)
}

fn verify_sha256(path: &Path, expected: &str) -> Result<(), String> {
    let mut file = std::fs::File::open(path)
        .map_err(|e| format!("Failed to open model for SHA256 check: {e}"))?;
    let mut context = Context::new(&SHA256);
    let mut buf = [0u8; 8192];
    loop {
        let n = std::io::Read::read(&mut file, &mut buf)
            .map_err(|e| format!("Failed to read model for SHA256 check: {e}"))?;
        if n == 0 {
            break;
        }
        context.update(&buf[..n]);
    }
    let digest = context.finish();
    let actual = digest
        .as_ref()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(format!(
            "SHA256 mismatch for {}: expected {expected}, got {actual}. \
             The model file may be corrupted or tampered with.",
            path.display()
        ))
    }
}

fn clean_stale_downloads(models_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(models_dir) else { return };
    let now = std::time::SystemTime::now();
    let stale_age = std::time::Duration::from_secs(7 * 24 * 60 * 60);

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("downloading") {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            if let Ok(modified) = meta.modified() {
                if now.duration_since(modified).unwrap_or_default() > stale_age {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }
}

fn download_model(models_dir: &Path, entry: &ModelEntry) -> Result<(), String> {
    let path = models_dir.join(&entry.filename);
    let temp_path = models_dir.join(format!("{}.downloading", entry.filename));

    eprintln!("Downloading {} from {}…", entry.display_name, entry.url);
    eprintln!("This is a one-time download of ~{} MB.", entry.size_bytes / 1_000_000);
    eprintln!("Saving to: {}", path.display());

    relay_core::telemetry::fire(relay_core::telemetry::TelemetryEvent::ModelDownloadStarted);

    let result = download_to_temp(&temp_path, entry);

    match result {
        Ok(()) => {
            std::fs::rename(&temp_path, &path)
                .map_err(|e| format!("Failed to rename temp file: {e}"))?;
            eprintln!("Download complete: {}", path.display());
            Ok(())
        }
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            Err(e)
        }
    }
}

fn download_to_temp(temp_path: &Path, entry: &ModelEntry) -> Result<(), String> {
    use reqwest::header::{HeaderMap, HeaderValue, RANGE};
    use std::io::Write;

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let mut headers = HeaderMap::new();
    let already = partial_size(temp_path);
    if already > 0 {
        let range_val = format!("bytes={}-", already);
        eprintln!("Resuming partial download from byte {}…", already);
        headers.insert(RANGE, HeaderValue::from_str(&range_val).unwrap());
    }

    let response = client
        .get(&entry.url)
        .headers(headers)
        .send()
        .map_err(|e| format!("Failed to start download: {e}"))?;

    if !response.status().is_success() && response.status().as_u16() != 206 {
        return Err(format!(
            "Download failed with HTTP {}: expected a GGUF model file but got an error response",
            response.status()
        ));
    }

    let total = if already > 0 {
        response
            .headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.rsplit('/').next())
            .and_then(|size| size.parse().ok())
            .unwrap_or_else(|| {
                response.content_length().unwrap_or(entry.size_bytes) + already
            })
    } else {
        response.content_length().unwrap_or(entry.size_bytes)
    };
    MODEL_DOWNLOAD_TOTAL.store(total, Ordering::SeqCst);

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(temp_path)
        .map_err(|e| format!("Failed to open temp file for writing: {e}"))?;

    let mut downloaded: u64 = already;
    let mut last_percent = 0u32;
    let mut last_telemetry = already;

    let mut reader = std::io::BufReader::new(response);
    let mut buf = vec![0u8; 32768];

    loop {
        let n = std::io::Read::read(&mut reader, &mut buf)
            .map_err(|e| format!("Read error: {e}"))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])
            .map_err(|e| format!("Write error: {e}"))?;
        downloaded += n as u64;
        MODEL_DOWNLOAD_PROGRESS.store(downloaded, Ordering::SeqCst);

        let percent = ((downloaded as f64 / total as f64) * 100.0) as u32;
        #[allow(clippy::manual_is_multiple_of)]
        if percent != last_percent && percent % 10 == 0 {
            eprintln!(
                "Download progress: {}% ({} MB / {} MB)",
                percent,
                downloaded / 1_000_000,
                total / 1_000_000
            );
            if (percent as u64 * total / 100) >= last_telemetry + (total / 10) {
                last_telemetry = downloaded;
                relay_core::telemetry::fire(
                    relay_core::telemetry::TelemetryEvent::ModelDownloadProgress {
                        percent,
                        bytes_downloaded: downloaded,
                        total_bytes: total,
                    },
                );
            }
            last_percent = percent;
        }

        if !temp_path.exists() {
            return Err("Download cancelled by user.".to_string());
        }
    }

    file.flush().map_err(|e| format!("Flush error: {e}"))?;
    drop(file);

    let content =
        std::fs::read(temp_path).map_err(|e| format!("Failed to read downloaded file: {e}"))?;
    if content.len() < 4 || &content[0..4] != b"GGUF" {
        let preview = if content.is_empty() {
            "empty".to_string()
        } else {
            String::from_utf8_lossy(&content[..content.len().min(64)]).to_string()
        };
        let _ = std::fs::remove_file(temp_path);
        return Err(format!(
            "Downloaded file is not a valid GGUF model. First bytes: {preview:?}. \
             Expected 'GGUF'. The model URL may be returning an error page instead of the model."
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_manifest() -> ModelManifest {
        serde_json::from_str(EMBEDDED_MANIFEST).expect("embedded manifest must be valid JSON")
    }

    #[test]
    fn test_manifest_embedded_is_valid() {
        let m = test_manifest();
        assert_eq!(m.version, 1);
        assert_eq!(m.default, "qwen-3.5-0.8b");
        assert!(m.models.contains_key("qwen-3.5-0.8b"));
    }

    #[test]
    fn test_default_entry_has_required_fields() {
        let m = test_manifest();
        let e = default_entry(&m);
        assert!(!e.display_name.is_empty());
        assert!(!e.url.is_empty());
        assert!(!e.filename.is_empty());
        assert_eq!(e.sha256.len(), 64);
        assert!(e.size_bytes > 0);
        assert!(e.architecture == "llama");
    }

    #[test]
    fn test_manifest_sha256_is_lowercase() {
        let m = test_manifest();
        let e = default_entry(&m);
        assert!(
            e.sha256
                .chars()
                .all(|c| !c.is_ascii_alphabetic() || c.is_ascii_lowercase()),
            "sha256 must be lowercase"
        );
    }

    #[test]
    fn test_verify_sha256_mismatch() {
        let m = test_manifest();
        let e = default_entry(&m);
        let tmp = std::env::temp_dir().join("relay-model-test-mismatch.gguf");
        std::fs::write(&tmp, b"GGUF-not-real-model").unwrap();
        let result = verify_sha256(&tmp, &e.sha256);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("SHA256 mismatch"));
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_model_path() {
        let m = test_manifest();
        let p = model_path(Path::new("/tmp/relay"), &m);
        let e = default_entry(&m);
        assert_eq!(
            p,
            PathBuf::from(format!("/tmp/relay/models/{}", e.filename))
        );
    }

    #[test]
    fn test_is_model_cached_no_file() {
        let m = test_manifest();
        let dir = std::env::temp_dir().join("relay-test-no-cache-12345");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(!is_model_cached(&dir, &m));
    }
}
