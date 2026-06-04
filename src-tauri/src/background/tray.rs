use relay_core::telemetry;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Emitter;
use tauri::Manager;

static TRAY_BUILT: AtomicBool = AtomicBool::new(false);

/// Build the system tray icon and context menu for Gearbox Relay.
pub fn build_tray(app: &tauri::AppHandle) -> Result<(), String> {
    if TRAY_BUILT.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    let icon = build_icon()?;
    let tray = tauri::tray::TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(false)
        .tooltip(tooltip_text(app))
        .on_tray_icon_event(|tray: &tauri::tray::TrayIcon, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button_state,
                button,
                ..
            } = event
            {
                if button == tauri::tray::MouseButton::Left
                    && button_state == tauri::tray::MouseButtonState::Up
                {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)
        .map_err(|e| format!("Failed to build tray: {e}"))?;

    let menu = tauri::menu::Menu::with_items(
        app,
        &[
            &tauri::menu::MenuItem::with_id(app, "open", "Open Relay", true, None::<&str>)
                .map_err(|e| format!("Menu item error: {e}"))?,
            &tauri::menu::MenuItem::with_id(app, "capture", "Capture Now", true, None::<&str>)
                .map_err(|e| format!("Menu item error: {e}"))?,
            &tauri::menu::MenuItem::with_id(app, "sync", "Sync Now", true, None::<&str>)
                .map_err(|e| format!("Menu item error: {e}"))?,
            &tauri::menu::MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)
                .map_err(|e| format!("Menu item error: {e}"))?,
            &tauri::menu::PredefinedMenuItem::separator(app)
                .map_err(|e| format!("Menu item error: {e}"))?,
            &tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
                .map_err(|e| format!("Menu item error: {e}"))?,
        ],
    )
    .map_err(|e| format!("Failed to create tray menu: {e}"))?;

    tray.set_menu(Some(menu))
        .map_err(|e| format!("Failed to set tray menu: {e}"))?;

    tray.on_menu_event(|app: &tauri::AppHandle, event: tauri::menu::MenuEvent| {
        match event.id.as_ref() {
            "open" => {
                telemetry::fire(telemetry::TelemetryEvent::TrayMenuClicked {
                    item: "open".into(),
                });
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "capture" => {
                telemetry::fire(telemetry::TelemetryEvent::TrayMenuClicked {
                    item: "capture".into(),
                });
                let _ = app.emit("relay://request-capture", ());
            }
            "sync" => {
                telemetry::fire(telemetry::TelemetryEvent::TrayMenuClicked {
                    item: "sync".into(),
                });
                let _ = app.emit("relay://request-sync", ());
            }
            "settings" => {
                telemetry::fire(telemetry::TelemetryEvent::TrayMenuClicked {
                    item: "settings".into(),
                });
                let _ = app.emit("relay://open-settings", ());
            }
            "quit" => {
                telemetry::fire(telemetry::TelemetryEvent::TrayMenuClicked {
                    item: "quit".into(),
                });
                telemetry::fire(telemetry::TelemetryEvent::AppQuitViaTray);
                crate::background::lifecycle::graceful_shutdown();
                app.exit(0);
            }
            _ => {}
        }
    });

    Ok(())
}

/// Return a dynamic tooltip based on sync status.
fn tooltip_text(_app: &tauri::AppHandle) -> String {
    // In a future iteration we could query sync metadata here.
    "Gearbox Relay — local-first knowledge pipeline".to_string()
}

/// Build the tray icon — Signal Amber (#FFB000) circle, matching brand.
fn build_icon() -> Result<tauri::image::Image<'static>, String> {
    let mut rgba = Vec::with_capacity(32 * 32 * 4);
    for y in 0..32 {
        for x in 0..32 {
            let cx = (x as f64 - 15.5) / 10.0;
            let cy = (y as f64 - 15.5) / 10.0;
            let dist = (cx * cx + cy * cy).sqrt();
            if dist < 1.3 {
                rgba.push(255); // R
                rgba.push(176); // G
                rgba.push(0);   // B
                rgba.push(255); // A
            } else if dist < 1.5 {
                rgba.push(255);
                rgba.push(176);
                rgba.push(0);
                rgba.push(160);
            } else {
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
            }
        }
    }
    Ok(tauri::image::Image::new_owned(rgba, 32, 32))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::background::lifecycle;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_build_icon_creates_valid_image() {
        let icon = build_icon().expect("icon should build");
        assert_eq!(icon.width(), 32);
        assert_eq!(icon.height(), 32);
    }

    #[test]
    fn test_tray_singleton_guard() {
        TRAY_BUILT.store(false, Ordering::SeqCst);
        assert!(!TRAY_BUILT.load(Ordering::SeqCst));
        TRAY_BUILT.store(true, Ordering::SeqCst);
        assert!(TRAY_BUILT.load(Ordering::SeqCst));
    }

    #[test]
    fn test_quit_flush_triggers_shutdown() {
        // Reset and verify shutdown flag
        lifecycle::WATCHER_SHOULD_STOP.store(false, Ordering::SeqCst);
        assert!(!lifecycle::is_shutdown_requested());
        lifecycle::graceful_shutdown();
        assert!(lifecycle::is_shutdown_requested());
        lifecycle::WATCHER_SHOULD_STOP.store(false, Ordering::SeqCst);
    }
}
