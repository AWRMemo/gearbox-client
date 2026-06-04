use crate::config;
use crate::db::account;
use crate::db::analytics;
use crate::db::stream::{self, StreamHighlight, StreamInfo};
use crate::web;

use std::sync::{Mutex, OnceLock};

/// Return the device-based user ID for ownership checks.
#[tauri::command]
pub fn get_device_id() -> Result<String, String> {
    config::get_device_id().map(|s| s.to_string())
}

/// Delete a stream by ID.
#[tauri::command]
pub fn delete_stream(stream_id: String) -> Result<(), String> {
    stream::delete_stream(&stream_id)
}

/// Create a new Stream with the given title and description.
/// The user_id is derived from the device-based ID.
/// Free tier is limited to 1 Stream; Pro users have unlimited.
#[tauri::command]
pub fn create_stream(title: String, description: String) -> Result<StreamInfo, String> {
    let user_id = config::get_device_id()?;

    let is_pro = account::is_pro(user_id).unwrap_or(false);
    if !is_pro {
        let existing = stream::list_user_streams(user_id).unwrap_or_default();
        if existing.len() >= 1 {
            return Err(
                "Free tier limited to 3 Streams. Upgrade to Pro for unlimited Streams.".to_string(),
            );
        }
    }

    let info = stream::create_stream(user_id, &title, &description)?;

    analytics::log_event(
        "stream_published",
        Some(&info.id),
        Some(user_id),
        None,
        None,
    )?;

    Ok(info)
}

/// Add a highlight to a Stream.
#[tauri::command]
pub fn add_to_stream(stream_id: String, highlight_id: String) -> Result<(), String> {
    stream::add_highlight_to_stream(&stream_id, &highlight_id)
}

/// Remove a highlight from a Stream.
#[tauri::command]
pub fn remove_from_stream(stream_id: String, highlight_id: String) -> Result<(), String> {
    stream::remove_highlight_from_stream(&stream_id, &highlight_id)
}

/// Get a single stream by ID.
#[tauri::command]
pub fn get_stream(stream_id: String) -> Result<StreamInfo, String> {
    stream::get_stream_by_id(&stream_id)
}

/// List all Streams belonging to the current user.
#[tauri::command]
pub fn list_my_streams() -> Result<Vec<StreamInfo>, String> {
    let user_id = config::get_device_id()?;
    stream::list_user_streams(user_id)
}

/// Get highlights for a given Stream.
#[tauri::command]
pub fn get_stream_highlights(stream_id: String) -> Result<Vec<StreamHighlight>, String> {
    stream::get_stream_highlights(&stream_id)
}

/// Generate the share link for a Stream (relay://stream/{id}).
/// Logs the share_link_generated analytics event.
#[tauri::command]
pub fn share_stream(stream_id: String, channel: String) -> Result<String, String> {
    let user_id = config::get_device_id()?;
    analytics::log_event(
        "stream_share_link_generated",
        Some(&stream_id),
        Some(user_id),
        None,
        Some(&channel),
    )?;

    Ok(format!("relay://stream/{}", stream_id))
}

/// Generate the public HTML page for a Stream and return its path.
#[tauri::command]
pub fn generate_stream_html(stream_id: String) -> Result<String, String> {
    let (file_path, _page) = generate_stream_html_inner(&stream_id, "public")?;
    Ok(file_path.to_string_lossy().to_string())
}

/// Export a self-contained stream HTML file to the user's Documents folder
/// and optionally open it in the default browser.
#[tauri::command]
pub fn export_stream_html(stream_id: String, open_in_browser: bool) -> Result<String, String> {
    let (file_path, _page) = generate_stream_html_inner(&stream_id, "export")?;

    if open_in_browser {
        let path_str = file_path.to_string_lossy().to_string();
        let _ = open::that(&path_str);
    }

    Ok(file_path.to_string_lossy().to_string())
}

fn generate_stream_html_inner(
    stream_id: &str,
    mode: &str,
) -> Result<(std::path::PathBuf, String), String> {
    let info = stream::get_stream_by_id(stream_id)?;
    let highlights = stream::get_stream_highlights(stream_id)?;

    let highlights_html: String = highlights
        .iter()
        .map(|h| web::generate_highlight_html(&h.text, &h.summary, &h.tags, &h.source_url))
        .collect::<Vec<_>>()
        .join("\n");

    let page = web::generate_stream_page(&info, &highlights_html);

    let file_path = match mode {
        "export" => {
            let docs = dirs_next()
                .ok_or("Could not find user Documents directory")?;
            let relay_dir = docs.join("Relay Streams");
            std::fs::create_dir_all(&relay_dir)
                .map_err(|e| format!("Failed to create export dir: {e}"))?;
            let sanitized = info.title.replace(
                |c: char| !c.is_alphanumeric() && c != '-' && c != '_' && c != ' ',
                "_",
            );
            relay_dir.join(format!("{sanitized}.html"))
        }
        _ => {
            let app_dir = config::get_app_dir()?;
            let public_dir = app_dir.join("public");
            std::fs::create_dir_all(&public_dir)
                .map_err(|e| format!("Failed to create public dir: {e}"))?;
            public_dir.join(format!("stream_{stream_id}.html"))
        }
    };

    std::fs::write(&file_path, &page)
        .map_err(|e| format!("Failed to write stream HTML: {e}"))?;

    Ok((file_path, page))
}

fn dirs_next() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE").ok().map(std::path::PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        dirs::document_dir()
    }
}

// ── Lazy local HTTP server for stream sharing ────────────────────────────────

/// Global server handle: `None` until first call to `get_stream_public_url`.
/// We only store the bound port because `tiny_http::Server` is not `Send + Sync`.
static SERVER_PORT: OnceLock<Mutex<u16>> = OnceLock::new();

/// Return the local HTTP URL for a Stream page (127.0.0.1:0 binding).
///
/// The server is created lazily on the first call and reused for all
/// subsequent streams (same port, different `/stream/{id}` path).
#[tauri::command]
pub fn get_stream_public_url(stream_id: String) -> Result<String, String> {
    let port = *SERVER_PORT
        .get_or_init(|| {
            std::thread::spawn(|| {
                let public_dir = config::get_app_dir()
                    .map(|p| p.join("public"))
                    .unwrap_or_else(|_| std::path::PathBuf::from("public"));
                if let Err(e) = web::server::start_local_server(&public_dir) {
                    eprintln!("Local HTTP server thread exited: {e}");
                }
            });
            // Give the thread a moment to bind and set HTTP_PORT.
            std::thread::sleep(std::time::Duration::from_millis(50));
            let port = web::server::HTTP_PORT.load(std::sync::atomic::Ordering::SeqCst);
            Mutex::new(port)
        })
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    build_stream_url(port, &stream_id)
}

/// Pure helper: format the public URL given a bound port and stream id.
fn build_stream_url(port: u16, stream_id: &str) -> Result<String, String> {
    if port == 0 {
        return Err("Local HTTP server failed to start.".to_string());
    }
    Ok(format!("http://127.0.0.1:{port}/stream/{stream_id}"))
}

/// Log a page_view event. Called when a non-owner opens a deep link.
/// Skips logging if the viewer is the stream curator (is_owner = true).
#[tauri::command]
pub fn log_stream_page_view(
    stream_id: String,
    visitor_id: Option<String>,
    is_owner: bool,
) -> Result<(), String> {
    if is_owner {
        return Ok(());
    }
    analytics::log_event(
        "stream_page_view",
        Some(&stream_id),
        None,
        visitor_id.as_deref(),
        None,
    )
}

/// Log a subscribe_click event.
#[tauri::command]
pub fn log_stream_subscribe_click(
    stream_id: String,
    visitor_id: Option<String>,
) -> Result<(), String> {
    analytics::log_event(
        "stream_subscribe_click",
        Some(&stream_id),
        None,
        visitor_id.as_deref(),
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_stream_public_url_format() {
        let url = build_stream_url(9876, "abc123").unwrap();
        assert_eq!(url, "http://127.0.0.1:9876/stream/abc123");
    }

    #[test]
    fn test_get_stream_public_url_zero_port_is_error() {
        let result = build_stream_url(0, "abc123");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Local HTTP server failed to start.");
    }
}
