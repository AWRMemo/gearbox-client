use relay_core::telemetry;

#[tauri::command]
pub fn get_telemetry_opt_out() -> Result<bool, String> {
    Ok(telemetry::is_opted_out())
}

#[tauri::command]
pub fn set_telemetry_opt_out(opt_out: bool) -> Result<(), String> {
    telemetry::set_opt_out(opt_out)
}

/// Toggle telemetry ON or OFF dynamically, without requiring an app restart.
#[tauri::command]
pub fn toggle_telemetry(enabled: bool) -> Result<(), String> {
    telemetry::set_opt_out(!enabled)?;
    crate::telemetry::reinit(None);
    Ok(())
}
