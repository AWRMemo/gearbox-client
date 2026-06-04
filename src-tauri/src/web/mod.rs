pub mod server;

use crate::db::stream::StreamInfo;

pub use server::{start_local_server, HTTP_PORT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamPageTarget {
    /// Local HTTP server — deep-link to subscribe: relay://subscribe/{id}
    LocalServer,
    /// Exported HTML file — link to stream view: relay://stream/{id}
    Export,
}

pub fn generate_stream_page(stream: &StreamInfo, highlights_html: &str) -> String {
    generate_stream_page_for(stream, highlights_html, StreamPageTarget::Export, "Relay User")
}

pub fn generate_stream_page_for(
    stream: &StreamInfo,
    highlights_html: &str,
    target: StreamPageTarget,
    curator: &str,
) -> String {
    let (subscribe_href, subscribe_text) = match target {
        StreamPageTarget::LocalServer => (
            format!("relay://subscribe/{}", html_escape(&stream.id)),
            "Subscribe in Relay",
        ),
        StreamPageTarget::Export => (
            format!("relay://stream/{}", html_escape(&stream.id)),
            "Subscribe to this Stream",
        ),
    };
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<meta http-equiv="Content-Security-Policy" content="default-src 'self'; style-src 'unsafe-inline';">
<title>{title} — Relay Stream</title>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: #1a1a2e; color: #e0e0e0; padding: 2rem;
  }}
  .container {{ max-width: 700px; margin: 0 auto; }}
  h1 {{ font-size: 1.5rem; color: #f0f0f0; margin-bottom: 0.25rem; }}
  .description {{ color: #888; font-size: 0.9rem; margin-bottom: 1.5rem; }}
  .highlight {{ background: #16213e; border-radius: 8px; padding: 1rem; margin-bottom: 1rem; }}
  .highlight .summary {{ font-size: 1rem; line-height: 1.5; margin-bottom: 0.5rem; }}
  .tags {{ display: flex; flex-wrap: wrap; gap: 0.4rem; margin-top: 0.5rem; }}
  .tag {{ background: #0f3460; color: #a0d8ef; padding: 0.25rem 0.6rem; border-radius: 4px; font-size: 0.8rem; }}
  .subscribe {{
    display: inline-block; background: #3a3a5c; color: #e0e0e0;
    border: none; padding: 0.6rem 1.2rem; border-radius: 6px;
    font-size: 0.9rem; cursor: pointer; text-decoration: none;
    margin-bottom: 1.5rem;
  }}
  .subscribe:hover {{ background: #4a4a7c; }}
  .source {{ font-size: 0.8rem; color: #666; margin-top: 0.3rem; }}
  .source a {{ color: #a0d8ef; }}
  .curator {{ color: #888; font-size: 0.85rem; margin-bottom: 1rem; }}
</style>
</head>
<body>
<div class="container">
  <h1>{title}</h1>
  <p class="description">{description}</p>
  <p class="curator">Curated by {curator}</p>
  <a class="subscribe" href="{subscribe_href}">{subscribe_text}</a>
  <div class="highlights">
    {highlights}
  </div>
</div>
</body>
</html>"##,
        title = html_escape(&stream.title),
        description = html_escape(&stream.description),
        curator = html_escape(curator),
        subscribe_href = subscribe_href,
        subscribe_text = subscribe_text,
        highlights = highlights_html,
    )
}

pub fn generate_highlight_html(
    _: &str,
    summary: &str,
    tags: &[String],
    source_url: &Option<String>,
) -> String {
    let tags_html = tags
        .iter()
        .map(|t| format!("<span class=\"tag\">{}</span>", html_escape(t)))
        .collect::<Vec<_>>()
        .join("\n    ");

    let source_html = match source_url {
        Some(url) if url.starts_with("http://") || url.starts_with("https://") => format!(
            "<p class=\"source\">Source: <a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\">{}</a></p>",
            html_escape(url),
            html_escape(url)
        ),
        Some(url) => format!(
            "<p class=\"source\">Source: {}</p>",
            html_escape(url)
        ),
        None => String::new(),
    };

    format!(
        r##"<div class="highlight">
  <p class="summary">{summary}</p>
  <div class="tags">{tags}</div>
  {source}
</div>"##,
        summary = html_escape(summary),
        tags = tags_html,
        source = source_html,
    )
}

pub(crate) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
