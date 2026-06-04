use super::ExportHighlight;

pub fn render_markdown(highlights: &[ExportHighlight]) -> String {
    let mut out = String::from("# Gearbox Relay — Highlights Export\n\n");
    out.push_str(&format!("_Exported {} highlights_\n\n", highlights.len()));

    for (i, h) in highlights.iter().enumerate() {
        out.push_str(&format!("## {}. {}\n\n", i + 1, escape_md(&h.summary)));
        out.push_str(&format!("**ID:** `{}`\n\n", h.id));
        out.push_str(&format!("**Created:** {}\n\n", h.created_at));

        if !h.tags.is_empty() {
            let badges: Vec<String> = h.tags.iter().map(|t| format!("`{}`", escape_md(t))).collect();
            out.push_str(&format!("**Tags:** {}\n\n", badges.join(" ")));
        }

        if let Some(ref url) = h.source_url {
            out.push_str(&format!("**Source:** [{}]({})\n\n", url, url));
        }
        if let Some(ref title) = h.source_title {
            out.push_str(&format!("**Title:** {}\n\n", escape_md(title)));
        }

        out.push_str("### Text\n\n");
        out.push_str(&format!("> {}\n\n", escape_md(&h.text)));

        if let Some(ref conn) = h.connection_suggestion {
            out.push_str("### Connection Suggestion\n\n");
            out.push_str(&format!("> {}\n\n", escape_md(conn)));
        }

        out.push_str("---\n\n");
    }

    out
}

fn escape_md(s: &str) -> String {
    s.replace('_', "\\_")
        .replace('*', "\\*")
        .replace('`', "\\`")
        .replace('[', "\\[")
}
