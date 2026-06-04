use crate::db::open_db;
pub use crate::types::{EnrichmentOutput, Highlight, ListedHighlight};

pub fn list_highlights(limit: usize, offset: usize) -> Result<Vec<ListedHighlight>, String> {
    let conn = open_db()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, text, source_url, source_title, source_author, summary, tags, connection_suggestion, created_at
             FROM highlights
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| format!("Failed to prepare list query: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![limit as i64, offset as i64], |row| {
            let tags_str: String = row.get(6)?;
            let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
            Ok(ListedHighlight {
                id: row.get(0)?,
                text: row.get(1)?,
                source_url: row.get(2)?,
                source_title: row.get(3)?,
                source_author: row.get(4)?,
                summary: row.get(5)?,
                tags,
                connection_suggestion: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| format!("List query failed: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Row error: {e}"))?);
    }
    Ok(results)
}

pub fn get_highlight_by_id(id: &str) -> Result<Option<ListedHighlight>, String> {
    let conn = open_db()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, text, source_url, source_title, source_author, summary, tags, connection_suggestion, created_at
             FROM highlights
             WHERE id = ?1",
        )
        .map_err(|e| format!("Failed to prepare get query: {e}"))?;

    let mut rows = stmt
        .query_map(rusqlite::params![id], |row| {
            let tags_str: String = row.get(6)?;
            let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
            Ok(ListedHighlight {
                id: row.get(0)?,
                text: row.get(1)?,
                source_url: row.get(2)?,
                source_title: row.get(3)?,
                source_author: row.get(4)?,
                summary: row.get(5)?,
                tags,
                connection_suggestion: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| format!("Get query failed: {e}"))?;

    rows.next().transpose().map_err(|e| e.to_string())
}

/// Persist a highlight and its enrichment to SQLite + FTS.
///
/// If `embedding` is `Some`, the real vector is written to the LanceDB vector
/// store. If `None`, a zero-vector placeholder is inserted so the record
/// remains discoverable until an embedder is available (desktop will backfill
/// via its own LanceDB index; relay-core's table is the shared fallback).
pub fn store_highlight(
    highlight: &Highlight,
    enrichment: &EnrichmentOutput,
    embedding: Option<&[f32]>,
) -> Result<(), String> {
    let mut conn = open_db()?;

    let tags_json = serde_json::to_string(&enrichment.tags)
        .map_err(|e| format!("Failed to serialize tags: {e}"))?;

    let connection_json = enrichment
        .connection_suggestion
        .as_ref()
        .map(|cs| serde_json::to_string(cs).unwrap_or_default());

    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to start transaction: {e}"))?;

    tx.execute(
        "INSERT OR REPLACE INTO highlights (id, text, source_url, source_title, source_author, summary, tags, connection_suggestion)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            highlight.id,
            highlight.text,
            highlight.source_url,
            highlight.source_title,
            highlight.source_author,
            enrichment.summary,
            tags_json,
            connection_json,
        ],
    )
    .map_err(|e| format!("Failed to insert highlight: {e}"))?;

    let tags_text = enrichment.tags.join(" ");

    tx.execute(
        "INSERT OR REPLACE INTO highlights_fts (id, text, summary, tags_text)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![highlight.id, highlight.text, enrichment.summary, tags_text],
    )
    .map_err(|e| format!("Failed to insert FTS entry: {e}"))?;

    tx.commit()
        .map_err(|e| format!("Failed to commit transaction: {e}"))?;

    if let Some(vector) = embedding {
        crate::db::vector::upsert_embedding(&highlight.id, vector)
            .map_err(|e| format!("Failed to upsert embedding: {e}"))?;
    } else {
        // TODO: zero-vector fallback for missing embedder — desktop backfills via LanceDB
        let fallback: Vec<f32> = vec![0.0_f32; crate::db::vector::VECTOR_DIM as usize];
        crate::db::vector::upsert_embedding(&highlight.id, &fallback)
            .map_err(|e| format!("Failed to upsert embedding: {e}"))?;
    }

    Ok(())
}
