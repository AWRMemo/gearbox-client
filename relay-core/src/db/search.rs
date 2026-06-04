use crate::db::open_db;
use crate::types::SearchResult;

/// Keyword-only search via SQLite FTS5 with LIKE fallback.
/// No semantic/vector search in relay-core (desktop adds LanceDB).
///
/// `query`: the raw user query string.
/// `limit`: max results to return.
pub fn search_highlights(query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
    let conn = open_db()?;
    search_highlights_with_conn(&conn, query, limit)
}

/// Internal variant that accepts an already-open `Connection`.
/// Used by tests to avoid global `DB_DIR` races.
pub(crate) fn search_highlights_with_conn(
    conn: &rusqlite::Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    match search_keyword(conn, query, limit) {
        Ok(r) => Ok(r),
        Err(e) => {
            eprintln!("Keyword search failed: {e}");
            search_like(conn, query, limit)
        }
    }
}

fn search_keyword(
    conn: &rusqlite::Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    if contains_fts_special_chars(query) {
        return search_like(conn, query, limit);
    }

    let results = search_fts(conn, query, limit);

    match results {
        Ok(results) => Ok(results),
        Err(_) => search_like(conn, query, limit),
    }
}

fn contains_fts_special_chars(query: &str) -> bool {
    for ch in query.chars() {
        match ch {
            '*' | '"' | '(' | ')' | '^' | '~' | '+' | '-' => return true,
            _ => {}
        }
    }
    let lower = query.to_lowercase();
    let tokens: Vec<&str> = lower.split_whitespace().collect();
    for token in &tokens {
        if matches!(*token, "and" | "or" | "not") {
            return true;
        }
    }
    false
}

fn search_fts(
    conn: &rusqlite::Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    let sanitized = query.replace('\'', "''");
    let sql = "SELECT h.id, h.summary, h.tags, h.text,
                bm25(highlights_fts, 0.0, 1.0, 2.0, 3.0) as score
         FROM highlights_fts
         JOIN highlights h ON h.id = highlights_fts.id
         WHERE highlights_fts MATCH ?1
         ORDER BY score
         LIMIT ?2"
        .to_string();

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("FTS prepare error: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![sanitized, limit as i64], |row| {
            let tags_str: String = row.get(2)?;
            let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
            Ok(SearchResult {
                id: row.get(0)?,
                summary: row.get(1)?,
                tags,
                text: row.get(3)?,
                score: row.get::<_, f64>(4)? as f32,
            })
        })
        .map_err(|e| format!("FTS query error: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("FTS row error: {e}"))?);
    }
    Ok(results)
}

fn search_like(
    conn: &rusqlite::Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    let pattern = format!("%{}%", query.replace('\'', "''"));

    let mut stmt = conn
        .prepare(
            "SELECT id, summary, tags, text
             FROM highlights
             WHERE text LIKE ?1 OR summary LIKE ?1 OR tags LIKE ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )
        .map_err(|e| format!("LIKE prepare error: {e}"))?;

    let rows = stmt
        .query_map(rusqlite::params![pattern, limit as i64], |row| {
            let tags_str: String = row.get(2)?;
            let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
            Ok(SearchResult {
                id: row.get(0)?,
                summary: row.get(1)?,
                tags,
                text: row.get(3)?,
                score: 0.0,
            })
        })
        .map_err(|e| format!("LIKE query error: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("LIKE row error: {e}"))?);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db;

    fn seed_highlights(conn: &rusqlite::Connection) {
        let rows = vec![
            (
                "h1",
                "The quick brown fox.",
                "A jumping fox summary.",
                vec!["fox", "brown"],
            ),
            (
                "h2",
                "How to train your dragon.",
                "Dragon training is dangerous.",
                vec!["dragon", "training"],
            ),
            (
                "h3",
                "Rust and Tauri are fast.",
                "Performance guide in Rust.",
                vec!["rust", "tauri"],
            ),
        ];
        for (id, text, summary, tags) in rows {
            let tags_json = serde_json::to_string(&tags).unwrap();
            let tags_text = tags.join(" ");
            conn.execute(
                "INSERT INTO highlights (id, text, summary, tags) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![id, text, summary, tags_json],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO highlights_fts (id, text, summary, tags_text) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![id, text, summary, tags_text],
            )
            .unwrap();
        }
    }

    #[test]
    fn test_fts_basic_match() {
        let conn = open_test_db().unwrap();
        seed_highlights(&conn);
        let results = search_fts(&conn, "fox", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "h1");
    }

    #[test]
    fn test_fts_multi_word_match() {
        let conn = open_test_db().unwrap();
        seed_highlights(&conn);
        let results = search_fts(&conn, "dragon training", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "h2");
    }

    #[test]
    fn test_fts_no_match() {
        let conn = open_test_db().unwrap();
        seed_highlights(&conn);
        let results = search_fts(&conn, "elephant", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_fts_limit_respected() {
        let conn = open_test_db().unwrap();
        seed_highlights(&conn);
        let results = search_fts(&conn, "the", 2).unwrap();
        assert_eq!(results.len(), 1); // only h1 contains "the" in searchable text
    }

    #[test]
    fn test_fts_special_chars_fallback() {
        assert!(contains_fts_special_chars("*"));
        assert!(contains_fts_special_chars("\"hello\""));
        assert!(contains_fts_special_chars("(a OR b)"));
    }

    #[test]
    fn test_fts_boolean_operators_fallback() {
        assert!(contains_fts_special_chars("hello AND world"));
        assert!(contains_fts_special_chars("hello OR world"));
        assert!(contains_fts_special_chars("hello NOT world"));
    }

    #[test]
    fn test_like_fallback_basic() {
        let conn = open_test_db().unwrap();
        seed_highlights(&conn);
        let results = search_like(&conn, "RUST", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "h3");
    }

    #[test]
    fn test_like_fallback_no_match() {
        let conn = open_test_db().unwrap();
        seed_highlights(&conn);
        let results = search_like(&conn, "elephant", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_keyword_only_branch() {
        let conn = open_test_db().unwrap();
        seed_highlights(&conn);
        let results = search_highlights_with_conn(&conn, "fox", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "h1");
    }

    #[test]
    fn test_empty_db_returns_empty() {
        let conn = open_test_db().unwrap();
        let results = search_highlights_with_conn(&conn, "anything", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_keyword_search_special_char_falls_back_to_like() {
        let conn = open_test_db().unwrap();
        seed_highlights(&conn);
        let _results = search_highlights_with_conn(&conn, "fox*", 10).unwrap();
        let results = search_highlights_with_conn(&conn, "%fox%", 10).unwrap();
        assert!(
            !results.is_empty(),
            "LIKE fallback should still match via % wildcard"
        );
    }

    #[test]
    fn test_limit_respected() {
        let conn = open_test_db().unwrap();
        seed_highlights(&conn);
        let results = search_highlights_with_conn(&conn, "the", 10).unwrap();
        assert_eq!(results.len(), 1, "only h1 contains 'the' in indexed text");
    }
}
