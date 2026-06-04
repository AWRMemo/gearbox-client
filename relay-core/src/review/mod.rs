use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::db::open_db;

const INITIAL_EASE: f64 = 2.5;
const MIN_EASE: f64 = 1.3;
const EASE_BONUS: [f64; 6] = [0.0, -0.8, -0.5, 0.0, 0.2, 0.3];
const INITIAL_INTERVAL_DAYS: i64 = 1;
const MIN_INTERVAL_DAYS: i64 = 1;

pub(crate) fn ensure_review_log_table(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS review_log (
            highlight_id TEXT PRIMARY KEY,
            ease_factor REAL NOT NULL DEFAULT 2.5,
            interval_days INTEGER NOT NULL DEFAULT 1,
            next_review_at TEXT NOT NULL,
            review_count INTEGER NOT NULL DEFAULT 0,
            last_grade INTEGER,
            reviewed_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (highlight_id) REFERENCES highlights(id) ON DELETE CASCADE
        );"
    ).map_err(|e| format!("review_log table: {e}"))
}

fn ensure_review_log(conn: &rusqlite::Connection) -> Result<(), String> {
    let _ = ensure_review_log_table(conn);
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItem {
    pub highlight_id: String,
    pub text: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub ease_factor: f64,
    pub interval_days: i64,
    pub next_review_at: String,
    pub review_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSession {
    pub items: Vec<ReviewItem>,
    pub total_due: usize,
}

pub fn grade_item(ease: f64, interval_days: i64, grade: u8) -> (f64, i64, String) {
    let g = grade.min(5);
    let bonus = EASE_BONUS[g as usize];
    let new_ease = (ease + bonus).max(MIN_EASE);

    let new_interval = if grade < 2 {
        INITIAL_INTERVAL_DAYS
    } else {
        let days = (interval_days as f64 * new_ease).round() as i64;
        days.max(MIN_INTERVAL_DAYS)
    };

    let next_date = Utc::now() + chrono::Duration::days(new_interval);
    let next_review_at = next_date.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    (new_ease, new_interval, next_review_at)
}

pub fn record_review(highlight_id: &str, grade: u8) -> Result<(), String> {
    let conn = open_db()?;
    ensure_review_log(&conn)?;
    record_review_conn(&conn, highlight_id, grade)
}

fn record_review_conn(conn: &rusqlite::Connection, highlight_id: &str, grade: u8) -> Result<(), String> {
    let current = conn.query_row(
        "SELECT ease_factor, interval_days, review_count FROM review_log WHERE highlight_id = ?1",
        rusqlite::params![highlight_id],
        |row| Ok((row.get::<_, f64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?)),
    );

    let (ease, interval, count) = match current {
        Ok(t) => t,
        Err(rusqlite::Error::QueryReturnedNoRows) => (INITIAL_EASE, INITIAL_INTERVAL_DAYS, 0),
        Err(e) => return Err(format!("Review query error: {e}")),
    };

    let (new_ease, new_interval, next_review_at) = grade_item(ease, interval, grade);
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    conn.execute(
        "INSERT INTO review_log (highlight_id, ease_factor, interval_days, next_review_at, review_count, last_grade, reviewed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(highlight_id) DO UPDATE SET
             ease_factor = excluded.ease_factor,
             interval_days = excluded.interval_days,
             next_review_at = excluded.next_review_at,
             review_count = excluded.review_count,
             last_grade = excluded.last_grade,
             reviewed_at = excluded.reviewed_at",
        rusqlite::params![highlight_id, new_ease, new_interval, next_review_at, count + 1, grade, now],
    ).map_err(|e| format!("review_log upsert: {e}"))?;

    Ok(())
}

pub fn get_due_reviews(limit: Option<usize>) -> Result<Vec<ReviewItem>, String> {
    let conn = open_db()?;
    ensure_review_log(&conn)?;
    get_due_reviews_conn(&conn, limit)
}

fn get_due_reviews_conn(conn: &rusqlite::Connection, limit: Option<usize>) -> Result<Vec<ReviewItem>, String> {
    let limit_val = limit.unwrap_or(20) as i64;
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let mut stmt = conn.prepare(
        "SELECT h.id, h.text, h.summary, h.tags,
                COALESCE(rl.ease_factor, 2.5),
                COALESCE(rl.interval_days, 1),
                COALESCE(rl.next_review_at, h.created_at),
                COALESCE(rl.review_count, 0)
         FROM highlights h
         LEFT JOIN review_log rl ON h.id = rl.highlight_id
         WHERE COALESCE(rl.next_review_at, h.created_at) <= ?1
         ORDER BY COALESCE(rl.next_review_at, h.created_at) ASC
         LIMIT ?2"
    ).map_err(|e| format!("review query error: {e}"))?;

    let rows = stmt.query_map(rusqlite::params![now, limit_val], |row| {
        let tags_str: String = row.get(3)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        Ok(ReviewItem {
            highlight_id: row.get(0)?,
            text: row.get(1)?,
            summary: row.get(2)?,
            tags,
            ease_factor: row.get(4)?,
            interval_days: row.get(5)?,
            next_review_at: row.get(6)?,
            review_count: row.get(7)?,
        })
    }).map_err(|e| format!("review query map: {e}"))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Row error: {e}"))?);
    }
    Ok(results)
}

pub fn count_due_reviews() -> Result<usize, String> {
    let conn = open_db()?;
    ensure_review_log(&conn)?;
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let count: i64 = conn.query_row(
        "SELECT COUNT(*)
         FROM highlights h
         LEFT JOIN review_log rl ON h.id = rl.highlight_id
         WHERE COALESCE(rl.next_review_at, h.created_at) <= ?1",
        rusqlite::params![now],
        |row| row.get(0),
    ).unwrap_or(0);
    Ok(count as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db;

    fn seed(conn: &rusqlite::Connection, id: &str, created_at: &str) {
        let _ = ensure_review_log_table(conn);
        conn.execute(
            "INSERT OR REPLACE INTO highlights (id, text, summary, tags, created_at, last_modified)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            rusqlite::params![id, "text", "summary", "[]", created_at],
        ).unwrap();
    }

    #[test]
    fn test_sm2_grade_again_resets_interval() {
        let (_ease, interval, _date) = grade_item(2.5, 5, 0);
        assert_eq!(interval, 1, "Again resets interval to 1 day");
    }

    #[test]
    fn test_sm2_grade_easy_boosts_ease() {
        let (ease, interval, _date) = grade_item(2.5, 3, 5);
        assert!(ease > 2.5, "Easy should boost ease factor, got {ease}");
        assert!(interval >= 3, "Interval should be at least 3 days, got {interval}");
    }

    #[test]
    fn test_sm2_ease_never_below_min() {
        let (ease, _, _) = grade_item(1.3, 1, 0);
        assert!(ease >= 1.3, "Ease should not go below MIN_EASE, got {ease}");
    }

    #[test]
    fn test_record_and_retrieve_review() {
        let conn = open_test_db().unwrap();
        seed(&conn, "r1", "2026-01-01T00:00:00Z");
        record_review_conn(&conn, "r1", 4).unwrap();

        // After grading good (4), the item should NOT be due anymore
        let items = get_due_reviews_conn(&conn, Some(10)).unwrap();
        let found = items.iter().any(|i| i.highlight_id == "r1");
        assert!(!found, "Reviewed item should not appear in due list immediately");

        // But the review_log should record the count
        let count: i64 = conn.query_row(
            "SELECT review_count FROM review_log WHERE highlight_id = ?1",
            rusqlite::params!["r1"],
            |row| row.get(0),
        ).unwrap_or(0);
        assert_eq!(count, 1, "Review count should be 1");
    }

    #[test]
    fn test_get_due_reviews_includes_new_highlights() {
        let conn = open_test_db().unwrap();
        seed(&conn, "r2", "2026-01-01T00:00:00Z");
        let due = get_due_reviews_conn(&conn, Some(10)).unwrap();
        assert!(!due.is_empty(), "New highlights should be due immediately");
    }
}
