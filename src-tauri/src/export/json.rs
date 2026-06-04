use super::ExportHighlight;

pub fn render_json(highlights: &[ExportHighlight]) -> Result<String, String> {
    serde_json::to_string_pretty(&highlights)
        .map_err(|e| format!("JSON serialization error: {e}"))
}
