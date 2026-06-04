use crate::config;
use crate::db::analytics;
use crate::db::subscriptions::{self, FeedHighlight};
use relay_core::types::StreamInfo;

#[tauri::command]
pub fn get_subscriptions() -> Result<Vec<StreamInfo>, String> {
    let user_id = config::get_device_id()?;
    subscriptions::get_subscribed_streams_info(user_id)
}

#[tauri::command]
pub fn subscribe_to_stream(stream_id: String) -> Result<(), String> {
    let user_id = config::get_device_id()?;
    subscriptions::subscribe(user_id, &stream_id)?;

    analytics::log_event(
        "stream_subscribe_click",
        Some(&stream_id),
        None,
        Some(user_id),
        None,
    )?;

    Ok(())
}

#[tauri::command]
pub fn unsubscribe_from_stream(stream_id: String) -> Result<(), String> {
    let user_id = config::get_device_id()?;
    subscriptions::unsubscribe(user_id, &stream_id)
}

#[tauri::command]
pub fn is_subscribed_to_stream(stream_id: String) -> Result<bool, String> {
    let user_id = config::get_device_id()?;
    subscriptions::is_subscribed(user_id, &stream_id)
}

#[tauri::command]
pub fn get_subscriber_feed(
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<FeedHighlight>, String> {
    let user_id = config::get_device_id()?;
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);
    subscriptions::get_subscriber_feed(user_id, limit, offset)
}
