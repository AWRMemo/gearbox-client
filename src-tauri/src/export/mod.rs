pub mod json;
pub mod markdown;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_highlights() -> Vec<ExportHighlight> {
        vec![
            ExportHighlight {
                id: "h1".into(),
                text: "The quick brown fox.".into(),
                summary: "A quick fox".into(),
                tags: vec!["fox".into(), "animal".into()],
                source_url: Some("https://example.com/fox".into()),
                source_title: Some("Fox Article".into()),
                source_author: None,
                created_at: "2026-05-20T12:00:00Z".into(),
                connection_suggestion: None,
            },
            ExportHighlight {
                id: "h2".into(),
                text: "Climate data.".into(),
                summary: "Climate change data".into(),
                tags: vec!["climate".into(), "science".into()],
                source_url: None,
                source_title: None,
                source_author: None,
                created_at: "2026-05-21T12:00:00Z".into(),
                connection_suggestion: Some(r#"{"bridging":"Related"}"#.into()),
            },
        ]
    }

    #[test]
    fn test_json_render_returns_valid_json() {
        let highlights = sample_highlights();
        let result = json::render_json(&highlights).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["summary"], "A quick fox");
    }

    #[test]
    fn test_markdown_render_contains_summaries() {
        let highlights = sample_highlights();
        let result = markdown::render_markdown(&highlights);
        assert!(result.contains("A quick fox"));
        assert!(result.contains("Climate change data"));
    }

    #[test]
    fn test_markdown_contains_tag_badges() {
        let highlights = sample_highlights();
        let result = markdown::render_markdown(&highlights);
        assert!(result.contains("`fox`"));
        assert!(result.contains("`climate`"));
    }


}

use std::path::Path;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportHighlight {
    pub id: String,
    pub text: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub source_url: Option<String>,
    pub source_title: Option<String>,
    pub source_author: Option<String>,
    pub created_at: String,
    pub connection_suggestion: Option<String>,
}

pub struct ExportFilter {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

pub fn query_highlights(filter: Option<&ExportFilter>) -> Result<Vec<ExportHighlight>, String> {
    let conn = crate::db::open_db().map_err(|e| format!("DB error: {e}"))?;

    let mut sql = String::from(
        "SELECT id, text, summary, tags, source_url, source_title, source_author, created_at, connection_suggestion
         FROM highlights WHERE 1=1",
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(f) = filter {
        if let Some(ref date_from) = f.date_from {
            sql.push_str(" AND created_at >= ?");
            params.push(Box::new(date_from.clone()));
        }
        if let Some(ref date_to) = f.date_to {
            sql.push_str(" AND created_at <= ?");
            params.push(Box::new(date_to.clone()));
        }
    }
    sql.push_str(" ORDER BY created_at DESC");

    let mut stmt = conn.prepare(&sql).map_err(|e| format!("Prepare error: {e}"))?;

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt
        .query_map(&param_refs[..], |row| {
            let tags_str: String = row.get(3)?;
            let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
            Ok(ExportHighlight {
                id: row.get(0)?,
                text: row.get(1)?,
                summary: row.get(2)?,
                tags,
                source_url: row.get(4)?,
                source_title: row.get(5)?,
                source_author: row.get(6)?,
                created_at: row.get(7)?,
                connection_suggestion: row.get(8)?,
            })
        })
        .map_err(|e| format!("Query error: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Row error: {e}"))?);
    }
    Ok(results)
}

/// Generate the export ZIP at a given path.
pub fn generate_export_zip(
    dest: &Path,
    highlights: &[ExportHighlight],
) -> Result<(), String> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    let file = std::fs::File::create(dest).map_err(|e| format!("Failed to create ZIP: {e}"))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default();

    let json_content = json::render_json(highlights)?;
    zip.start_file("highlights.json", options)
        .map_err(|e| format!("ZIP error: {e}"))?;
    zip.write_all(json_content.as_bytes())
        .map_err(|e| format!("ZIP write error: {e}"))?;

    let md_content = markdown::render_markdown(highlights);
    zip.start_file("highlights.md", options)
        .map_err(|e| format!("ZIP error: {e}"))?;
    zip.write_all(md_content.as_bytes())
        .map_err(|e| format!("ZIP write error: {e}"))?;

    zip.finish().map_err(|e| format!("ZIP finish error: {e}"))?;
    Ok(())
}
