use std::sync::atomic::AtomicBool;

use crate::db::open_db;

/// Flag set when the AI service quality has degraded below threshold.
/// Once set, the app uses FallbackService permanently until restart.
pub static AI_SERVICE_FAILED: AtomicBool = AtomicBool::new(false);

/// Create the `enrichment_quality_log` table if it does not exist.
fn ensure_table(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS enrichment_quality_log (
            capture_id TEXT PRIMARY KEY,
            parse_success INTEGER NOT NULL,
            tag_count INTEGER NOT NULL,
            summary_length INTEGER NOT NULL,
            timestamp TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_quality_timestamp ON enrichment_quality_log(timestamp);",
    )
    .map_err(|e| format!("Failed to create enrichment_quality_log table: {e}"))?;
    Ok(())
}

/// Record a quality snapshot for a single enrichment (public API using global pool).
pub fn record_quality(
    capture_id: &str,
    parse_success: bool,
    tag_count: usize,
    summary_length: usize,
    model_name: &str,
) -> Result<(), String> {
    let conn = open_db()?;
    record_quality_with_conn(&conn, capture_id, parse_success, tag_count, summary_length, model_name)
}

fn record_quality_with_conn(
    conn: &rusqlite::Connection,
    capture_id: &str,
    parse_success: bool,
    tag_count: usize,
    summary_length: usize,
    model_name: &str,
) -> Result<(), String> {
    ensure_table(conn)?;
    conn.execute(
        "INSERT INTO enrichment_quality_log (capture_id, parse_success, tag_count, summary_length, model_name)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(capture_id) DO UPDATE SET
             parse_success = excluded.parse_success,
             tag_count = excluded.tag_count,
             summary_length = excluded.summary_length,
             model_name = excluded.model_name,
             timestamp = datetime('now')",
        rusqlite::params![
            capture_id,
            if parse_success { 1i32 } else { 0i32 },
            tag_count as i64,
            summary_length as i64,
            model_name,
        ],
    )
    .map_err(|e| format!("Failed to record quality: {e}"))?;
    Ok(())
}

/// Return the percentage of captures that parsed successfully within the last `window` rows.
/// Returns `100.0` if there are no rows.
pub fn rolling_parse_success_rate(window: usize) -> Result<f64, String> {
    let conn = open_db()?;
    rolling_parse_success_rate_with_conn(&conn, window)
}

fn rolling_parse_success_rate_with_conn(
    conn: &rusqlite::Connection,
    window: usize,
) -> Result<f64, String> {
    if window == 0 {
        return Ok(100.0);
    }
    ensure_table(conn)?;
    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM enrichment_quality_log WHERE capture_id IN (
                SELECT capture_id FROM enrichment_quality_log ORDER BY capture_id DESC LIMIT ?1
            )",
            [window as i64],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count quality rows: {e}"))?;

    if total == 0 {
        return Ok(100.0);
    }

    let successes: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM enrichment_quality_log
             WHERE parse_success = 1 AND capture_id IN (
                 SELECT capture_id FROM enrichment_quality_log ORDER BY capture_id DESC LIMIT ?1
             )",
            [window as i64],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count successes: {e}"))?;

    Ok((successes as f64 / total as f64) * 100.0)
}

/// Return `true` if the rolling parse-success rate over `window` is below `threshold` %.
/// `threshold` is 0.0–100.0 (e.g. 70.0).
pub fn should_degrade_to_fallback(window: usize, threshold: f64) -> Result<bool, String> {
    let rate = rolling_parse_success_rate(window)?;
    Ok(rate < threshold)
}

#[allow(dead_code)]
fn should_degrade_to_fallback_with_conn(
    conn: &rusqlite::Connection,
    window: usize,
    threshold: f64,
) -> Result<bool, String> {
    let rate = rolling_parse_success_rate_with_conn(conn, window)?;
    Ok(rate < threshold)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_test_db;

    fn seed_rows(conn: &rusqlite::Connection, offset: usize, successes: usize, failures: usize) {
        let mut i = offset;
        for _ in 0..successes {
            conn.execute(
                "INSERT INTO enrichment_quality_log (capture_id, parse_success, tag_count, summary_length, model_name)
                 VALUES (?1, 1, 3, 100, 'test-model')",
                [format!("cap-{i}")],
            )
            .unwrap();
            i += 1;
        }
        for _ in 0..failures {
            conn.execute(
                "INSERT INTO enrichment_quality_log (capture_id, parse_success, tag_count, summary_length, model_name)
                 VALUES (?1, 0, 0, 0, 'test-model')",
                [format!("cap-{i}")],
            )
            .unwrap();
            i += 1;
        }
    }

    #[test]
    fn test_rolling_window_accuracy() {
        let conn = open_test_db().unwrap();
        ensure_table(&conn).unwrap();
        seed_rows(&conn, 0, 7, 3); // 7 successes, 3 failures out of 10
        let rate = rolling_parse_success_rate_with_conn(&conn, 10).unwrap();
        assert!((rate - 70.0).abs() < 0.01, "expected 70.0, got {rate}");

        // Window of 5 should consider only the last 5 inserted
        // Since rows have identical timestamps, SQLite order is insertion order
        let rate5 = rolling_parse_success_rate_with_conn(&conn, 5).unwrap();
        // last 5 are: cap-5 (success), cap-6 (success), cap-7 (fail), cap-8 (fail), cap-9 (fail)
        // Wait, seed order: 7 successes first (cap-0..cap-6), then 3 failures (cap-7..cap-9)
        // So last 5 are cap-5..cap-9 => 2 successes, 3 failures = 40%
        assert!((rate5 - 40.0).abs() < 0.01, "expected 40.0, got {rate5}");
    }

    #[test]
    fn test_degrade_trigger_at_70_percent() {
        let conn = open_test_db().unwrap();
        ensure_table(&conn).unwrap();
        seed_rows(&conn, 0, 7, 3); // 70% exactly
        let should_degrade = should_degrade_to_fallback_with_conn(&conn, 10, 70.0).unwrap();
        assert!(
            !should_degrade,
            "rate == 70% should NOT trigger degradation"
        );

        // Add 1 more failure -> 7/11 = 63.6% < 70%
        seed_rows(&conn, 10, 0, 1);
        let should_degrade_now = should_degrade_to_fallback_with_conn(&conn, 11, 70.0).unwrap();
        assert!(should_degrade_now, "rate < 70% should trigger degradation");
    }

    #[test]
    fn test_empty_db_returns_100() {
        let conn = open_test_db().unwrap();
        let rate = rolling_parse_success_rate_with_conn(&conn, 10).unwrap();
        assert_eq!(rate, 100.0);
    }

    #[test]
    fn test_window_zero_returns_100() {
        let conn = open_test_db().unwrap();
        ensure_table(&conn).unwrap();
        seed_rows(&conn, 0, 3, 7);
        let rate = rolling_parse_success_rate_with_conn(&conn, 0).unwrap();
        assert_eq!(rate, 100.0);
    }
}
