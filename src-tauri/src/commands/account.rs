use crate::config;
use crate::db::account::{self, UserProfile};
use crate::db::tiers::{self, PaywallTrigger};

#[tauri::command]
pub fn get_user_profile() -> Result<UserProfile, String> {
    let user_id = config::get_device_id()?;
    account::get_profile(user_id)
}

#[tauri::command]
pub fn get_user_profile_by_id(user_id: String) -> Result<UserProfile, String> {
    account::get_profile(&user_id)
}

#[tauri::command]
pub fn set_user_email(email: String) -> Result<(), String> {
    let user_id = config::get_device_id()?;
    account::set_email(user_id, &email)
}

#[tauri::command]
pub fn get_user_tier() -> Result<String, String> {
    let user_id = config::get_device_id()?;
    let profile = account::get_profile(user_id)?;
    Ok(profile.tier)
}

#[tauri::command]
pub fn set_user_tier(tier: String) -> Result<(), String> {
    if tier != "free" && tier != "pro" {
        return Err(format!("Invalid tier: {}. Must be 'free' or 'pro'", tier));
    }
    let user_id = config::get_device_id()?;
    account::set_tier(user_id, &tier)
}

#[tauri::command]
pub fn check_paywall_trigger() -> Result<PaywallTrigger, String> {
    let user_id = config::get_device_id()?;
    tiers::check_paywall_trigger(user_id)
}
