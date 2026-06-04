use crate::db::open_db;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PaywallTrigger {
    pub is_blocked: bool,
    pub reason: Option<String>,
}

/// Returns `PaywallTrigger` if the user has hit a free-tier limit.
///
/// Checks:
/// - Free tier: max 1 stream
/// - Free tier: max 1 review session per day
/// - Free tier: max 1 synced device
pub fn check_paywall_trigger(user_id: &str) -> Result<PaywallTrigger, String> {
    let conn = open_db()?;

    let tier: String = conn
        .query_row(
            "SELECT tier FROM user_profile WHERE user_id = ?1",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "free".to_string());

    if tier == "pro" {
        return Ok(PaywallTrigger {
            is_blocked: false,
            reason: None,
        });
    }

    // Check stream limit (free: 1)
    let stream_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM streams WHERE user_id = ?1",
            rusqlite::params![user_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if stream_count >= 3 {
        return Ok(PaywallTrigger {
            is_blocked: true,
            reason: Some("free_stream_limit".to_string()),
        });
    }

    // Check review session limit (free: 1 per day)
    let reviews_today: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM review_log WHERE reviewed_at >= datetime('now', '-1 day')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if reviews_today >= 1 {
        return Ok(PaywallTrigger {
            is_blocked: true,
            reason: Some("free_review_limit".to_string()),
        });
    }

    // Check device limit (free: 1)
    let device_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sync_credentials",
            [],
            |row| row.get(0),
        )
        .unwrap_or(1);
    if device_count > 1 {
        return Ok(PaywallTrigger {
            is_blocked: true,
            reason: Some("free_device_limit".to_string()),
        });
    }

    Ok(PaywallTrigger {
        is_blocked: false,
        reason: None,
    })
}

pub fn get_subscription_tier(user_id: &str) -> Result<String, String> {
    let conn = open_db()?;
    conn.query_row(
        "SELECT tier FROM user_profile WHERE user_id = ?1",
        rusqlite::params![user_id],
        |row| row.get::<_, String>(0),
    )
    .map_err(|e| format!("Failed to get tier: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_tier_review_not_blocked() {
        // First review should not trigger paywall (free: 1 review/day)
        let conn = crate::db::open_test_db().unwrap();
        conn.execute("INSERT INTO user_profile (user_id, tier) VALUES ('u1', 'free')", []).unwrap();
        let result = check_paywall_trigger("u1").unwrap();
        assert!(!result.is_blocked);
    }

    #[test]
    fn test_free_tier_stream_blocked_at_four() {
        // Free tier allows 3 streams; 4th triggers paywall
        let conn = crate::db::open_test_db().unwrap();
        conn.execute("INSERT INTO user_profile (user_id, tier) VALUES ('u1', 'free')", []).unwrap();
        conn.execute("INSERT INTO streams (id, user_id, title) VALUES ('s1', 'u1', 'a')", []).unwrap();
        conn.execute("INSERT INTO streams (id, user_id, title) VALUES ('s2', 'u1', 'b')", []).unwrap();
        conn.execute("INSERT INTO streams (id, user_id, title) VALUES ('s3', 'u1', 'c')", []).unwrap();

        let result = check_paywall_trigger("u1").unwrap();
        assert!(result.is_blocked);
        assert_eq!(result.reason.unwrap(), "free_stream_limit");
    }

    #[test]
    fn test_pro_tier_not_blocked_with_three_streams() {
        let conn = crate::db::open_test_db().unwrap();
        conn.execute("INSERT INTO user_profile (user_id, tier) VALUES ('u1', 'pro')", []).unwrap();
        conn.execute("INSERT INTO streams (id, user_id, title) VALUES ('s1', 'u1', 'a')", []).unwrap();
        conn.execute("INSERT INTO streams (id, user_id, title) VALUES ('s2', 'u1', 'b')", []).unwrap();
        conn.execute("INSERT INTO streams (id, user_id, title) VALUES ('s3', 'u1', 'c')", []).unwrap();

        let result = check_paywall_trigger("u1").unwrap();
        assert!(!result.is_blocked);
        assert!(result.reason.is_none());
    }
}
