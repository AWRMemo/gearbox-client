use crate::db::search::SearchResult;

/// Search stored highlights by query string.
/// If an embedding service is available, performs hybrid semantic + keyword search.
/// Otherwise falls back to keyword-only search.
/// Optional date range and source domain filters narrow results.
#[tauri::command]
pub fn search(
    query: String,
    limit: Option<usize>,
    date_from: Option<String>,
    date_to: Option<String>,
    source_domain: Option<String>,
) -> Result<Vec<SearchResult>, String> {
    let limit = limit.unwrap_or(20);

    if let Some(err) = crate::take_init_error() {
        eprintln!("Vector DB init failed previously: {err}");
    }

    let query_vector = match crate::EMBEDDING_SERVICE_GLOBAL.get() {
        Some(Some(es)) => match es.encode(&query) {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("Query embedding failed: {e}");
                None
            }
        },
        _ => None,
    };

    let vector_slice = query_vector.as_deref();
    search_highlights_with_filters(&query, vector_slice, limit, date_from, date_to, source_domain)
}

fn search_highlights_with_filters(
    query: &str,
    query_vector: Option<&[f32]>,
    limit: usize,
    date_from: Option<String>,
    date_to: Option<String>,
    source_domain: Option<String>,
) -> Result<Vec<SearchResult>, String> {
    let conn = crate::db::open_db()?;

    // Get results from the main hybrid search
    let results = crate::db::search::search_highlights_with_conn(
        &conn,
        query,
        query_vector,
        limit.saturating_mul(3),
    )?;

    // Post-filter with date range and domain using the same connection
    let filtered: Vec<SearchResult> = results
        .into_iter()
        .filter(|r| {
            // Apply date-from filter
            if let Some(ref from) = date_from {
                let ok = conn.query_row(
                    "SELECT 1 FROM highlights WHERE id = ?1 AND created_at >= ?2",
                    rusqlite::params![r.id, from],
                    |_| Ok(()),
                ).is_ok();
                if !ok { return false; }
            }
            // Apply date-to filter
            if let Some(ref to) = date_to {
                let ok = conn.query_row(
                    "SELECT 1 FROM highlights WHERE id = ?1 AND created_at <= ?2",
                    rusqlite::params![r.id, to],
                    |_| Ok(()),
                ).is_ok();
                if !ok { return false; }
            }
            // Apply domain filter
            if let Some(ref domain) = source_domain {
                let ok = conn.query_row(
                    "SELECT 1 FROM highlights WHERE id = ?1 AND source_url LIKE ?2",
                    rusqlite::params![r.id, format!("%{}%", domain)],
                    |_| Ok(()),
                ).is_ok();
                if !ok { return false; }
            }
            true
        })
        .take(limit)
        .collect();

    Ok(filtered)
}
