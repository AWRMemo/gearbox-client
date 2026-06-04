use relay_core::review;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ReviewSessionResponse {
    pub items: Vec<review::ReviewItem>,
    pub total_due: usize,
}

#[tauri::command]
pub fn get_review_session(limit: Option<usize>) -> Result<ReviewSessionResponse, String> {
    let user_id = crate::config::get_device_id()?;
    let trigger = relay_core::db::tiers::check_paywall_trigger(user_id)?;
    if trigger.is_blocked {
        return Err(trigger.reason.unwrap_or_else(|| "paywall_blocked".to_string()));
    }
    let items = review::get_due_reviews(limit)?;
    let total_due = review::count_due_reviews()?;
    Ok(ReviewSessionResponse { items, total_due })
}

#[tauri::command]
pub fn grade_review_item(highlight_id: String, grade: u8) -> Result<(), String> {
    if grade > 5 {
        return Err("Grade must be 0-5".to_string());
    }
    review::record_review(&highlight_id, grade)
}
