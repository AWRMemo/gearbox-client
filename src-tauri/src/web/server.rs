use std::path::Path;
use std::sync::atomic::{AtomicU16, Ordering};

use tiny_http::{Response, Server};

/// Globally accessible port of the local HTTP server.
pub static HTTP_PORT: AtomicU16 = AtomicU16::new(0);

/// Start a loopback-only HTTP server.
/// Binds to `127.0.0.1:0` (OS-assigned port) and stores the bound port in
/// `HTTP_PORT`.
///
/// Routes:
/// - `/stream/{id}` — dynamically queries the local DB and returns an HTML
///   page with the Stream title and highlight summaries (no raw full text).
/// - Everything else — static files from `public_dir` (legacy support).
pub fn start_local_server(public_dir: &Path) -> Result<(), String> {
    let server = Server::http("127.0.0.1:0").map_err(|e| format!("Failed to bind server: {e}"))?;

    let port = server
        .server_addr()
        .to_ip()
        .map(|addr| addr.port())
        .unwrap_or(0);
    HTTP_PORT.store(port, Ordering::SeqCst);
    eprintln!("Local HTTP server listening on 127.0.0.1:{port}");

    for request in server.incoming_requests() {
        if request.method().as_str() != "GET" {
            let _ = request.respond(Response::empty(405));
            continue;
        }

        let url_path = request.url();

        // Dynamic stream page route: /stream/{id}
        if let Some(stream_id) = url_path.strip_prefix("/stream/") {
            let stream_id = stream_id.split('/').next().unwrap_or(stream_id);
            let html = match render_stream_page(stream_id) {
                Ok(page) => page,
                Err(e) => {
                    eprintln!("Stream page render error for {stream_id}: {e}");
                    let _ = request.respond(Response::empty(500));
                    continue;
                }
            };
            let response = Response::from_string(html).with_header(
                tiny_http::Header::from_bytes(
                    &b"Content-Type"[..],
                    &b"text/html; charset=utf-8"[..],
                )
                .unwrap(),
            );
            let _ = request.respond(response);
            continue;
        }

        // Static file fallback
        let url_path = url_path.trim_start_matches('/');
        let safe_path = sanitize_path(url_path);
        let file_path = public_dir.join(&safe_path);

        if !file_path.starts_with(public_dir) {
            let _ = request.respond(Response::empty(403));
            continue;
        }

        if !file_path.exists() || !file_path.is_file() {
            let _ = request.respond(Response::empty(404));
            continue;
        }

        match std::fs::read(&file_path) {
            Ok(data) => {
                let mime = mime_guess(&file_path);
                let response = Response::from_data(data).with_header(
                    tiny_http::Header::from_bytes(&b"Content-Type"[..], mime.as_bytes())
                        .unwrap_or_else(|_| {
                            tiny_http::Header::from_bytes(
                                &b"Content-Type"[..],
                                &b"application/octet-stream"[..],
                            )
                            .unwrap()
                        }),
                );
                let _ = request.respond(response);
            }
            Err(_) => {
                let _ = request.respond(Response::empty(500));
            }
        }
    }

    Ok(())
}

/// Query the local DB and render an HTML page for the given stream.
/// Only summaries and tags are shown — no raw highlight full text.
fn render_stream_page(stream_id: &str) -> Result<String, String> {
    use crate::db::stream;

    let info = stream::get_stream_by_id(stream_id)?;
    let highlights = stream::get_stream_highlights(stream_id)?;

    let highlights_html: String = highlights
        .iter()
        .map(|h| super::generate_highlight_html("", &h.summary, &h.tags, &h.source_url))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(super::generate_stream_page_for(
        &info,
        &highlights_html,
        super::StreamPageTarget::LocalServer,
        "Relay User",
    ))
}

fn sanitize_path(url_path: &str) -> String {
    url_path
        .split('/')
        .filter(|s| !s.is_empty() && *s != "." && *s != "..")
        .collect::<Vec<&str>>()
        .join("/")
}

fn mime_guess(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html".to_string(),
        Some("css") => "text/css".to_string(),
        Some("js") => "application/javascript".to_string(),
        Some("json") => "application/json".to_string(),
        Some("png") => "image/png".to_string(),
        Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
        Some("svg") => "image/svg+xml".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_path() {
        assert_eq!(sanitize_path("stream_abc.html"), "stream_abc.html");
        assert_eq!(sanitize_path("../secret.txt"), "secret.txt");
        assert_eq!(sanitize_path("foo/./bar/../../baz"), "foo/bar/baz");
    }

    #[test]
    fn test_mime_guess() {
        assert_eq!(mime_guess(Path::new("stream.html")), "text/html");
        assert_eq!(mime_guess(Path::new("style.css")), "text/css");
        assert_eq!(
            mime_guess(Path::new("data.bin")),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(super::super::html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(super::super::html_escape("A & B"), "A &amp; B");
        assert_eq!(super::super::html_escape("\"quote\""), "&quot;quote&quot;");
    }
}
