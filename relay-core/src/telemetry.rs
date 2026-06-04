use std::time::Instant;

/// Telemetry event types that carry no PII.
#[derive(Debug, Clone)]
pub enum TelemetryEvent {
    /// `enrich_highlight_latency_ms`: latency from capture start to enrichment done.
    EnrichLatency { ms: u64, parse_success: bool },
    /// `sync_attempt`: count, time, success/failure.
    SyncAttempt { ms: u64, success: bool },
    /// `app_start_cold`: time from process start to interactive window.
    AppStartCold { ms: u64 },
    /// `app_start_warm`: time from process start when model already cached.
    AppStartWarm { ms: u64 },
    /// `tray_menu_clicked`: user clicked a tray menu item.
    TrayMenuClicked { item: String },
    /// `app_quit_via_tray`: user exited the app via the tray "Quit" item.
    AppQuitViaTray,
    /// `system_suspend`: OS entered sleep/suspend state.
    SystemSuspend,
    /// `system_wake`: OS resumed from sleep/suspend.
    SystemWake,
    /// `model_download_started`: user or system initiated model download.
    ModelDownloadStarted,
    /// `model_download_progress`: 10% milestone reached.
    ModelDownloadProgress { percent: u32, bytes_downloaded: u64, total_bytes: u64 },
    /// `model_download_success`: model verified and ready.
    ModelDownloadSuccess,
    /// `model_download_failed`: download or verification error.
    ModelDownloadFailed { reason: String },
}

/// Convenience: measure enrichment latency (call `start = Instant::now()` before enrichment,
/// then `record_enrich_latency(start, parsed_ok)` after).
pub fn record_enrich_latency(start: Instant, parse_success: bool) {
    let ms = start.elapsed().as_millis() as u64;
    fire(TelemetryEvent::EnrichLatency { ms, parse_success });
}

/// Convenience: measure sync latency.
pub fn record_sync_latency(start: Instant, success: bool) {
    let ms = start.elapsed().as_millis() as u64;
    fire(TelemetryEvent::SyncAttempt { ms, success });
}

/// Fire a telemetry event.  If the user has opted out, this is a no-op.
pub fn fire(event: TelemetryEvent) {
    if is_opted_out() {
        return;
    }

    match event {
        TelemetryEvent::EnrichLatency { ms, parse_success } => {
            capture_message(
                "enrich_highlight_latency_ms",
                &format!("latency={ms}ms,parse_success={parse_success}"),
                SentryLevel::Info,
            );
        }
        TelemetryEvent::SyncAttempt { ms, success } => {
            capture_message(
                "sync_attempt",
                &format!("latency={ms}ms,success={success}"),
                SentryLevel::Info,
            );
        }
        TelemetryEvent::AppStartCold { ms } => {
            capture_message(
                "app_start_cold",
                &format!("latency={ms}ms"),
                SentryLevel::Info,
            );
        }
        TelemetryEvent::AppStartWarm { ms } => {
            capture_message(
                "app_start_warm",
                &format!("latency={ms}ms"),
                SentryLevel::Info,
            );
        }
        TelemetryEvent::TrayMenuClicked { item } => {
            capture_message(
                "tray_menu_clicked",
                &format!("item={item}"),
                SentryLevel::Info,
            );
        }
        TelemetryEvent::AppQuitViaTray => {
            capture_message(
                "app_quit_via_tray",
                "user quit via tray menu",
                SentryLevel::Info,
            );
        }
        TelemetryEvent::SystemSuspend => {
            capture_message("system_suspend", "OS entering sleep", SentryLevel::Info);
        }
        TelemetryEvent::SystemWake => {
            capture_message("system_wake", "OS resuming from sleep", SentryLevel::Info);
        }
        TelemetryEvent::ModelDownloadStarted => {
            capture_message("model_download_started", "SLM download initiated", SentryLevel::Info);
        }
        TelemetryEvent::ModelDownloadProgress {
            percent,
            bytes_downloaded,
            total_bytes,
        } => {
            capture_message(
                "model_download_progress",
                &format!("percent={percent},bytes={bytes_downloaded},total={total_bytes}"),
                SentryLevel::Info,
            );
        }
        TelemetryEvent::ModelDownloadSuccess => {
            capture_message("model_download_success", "SLM verified and ready", SentryLevel::Info);
        }
        TelemetryEvent::ModelDownloadFailed { ref reason } => {
            capture_message(
                "model_download_failed",
                &format!("reason={reason}"),
                SentryLevel::Error,
            );
        }
    }
}

/// Check whether telemetry is disabled via `settings.telemetry_disabled`.
/// Stored in SQLite so it syncs across devices.
/// Defaults to **opted-out** (telemetry OFF) when the DB is unavailable or key not set.
/// Set to `"false"` to opt-in (telemetry ON).
pub fn is_opted_out() -> bool {
    let val: Option<String> = crate::db::open_db().ok().and_then(|conn| {
        conn.query_row(
            "SELECT value FROM settings WHERE key = 'telemetry_disabled'",
            [],
            |row| row.get(0),
        )
        .ok()
    });
    // Telemetry is opt-in: default OFF (true) unless explicitly set to "false".
    val.as_deref() != Some("false")
}

/// Set opt-out preference.
pub fn set_opt_out(opt_out: bool) -> Result<(), String> {
    let conn = crate::db::open_db()?;
    let val = if opt_out { "true" } else { "false" };
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('telemetry_disabled', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![val],
    )
    .map_err(|e| format!("Failed to set telemetry opt-out: {e}"))?;
    Ok(())
}

/// Read the Sentry DSN from environment or settings.
/// Returns `None` if no DSN is configured (telemetry is silent).
pub fn sentry_dsn() -> Option<String> {
    std::env::var("SENTRY_DSN").ok().or_else(|| {
        crate::db::open_db().ok().and_then(|conn| {
            conn.query_row(
                "SELECT value FROM settings WHERE key = 'sentry_dsn'",
                [],
                |row| row.get::<_, String>(0),
            )
            .ok()
        })
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SentryLevel {
    Info,
    Warning,
    Error,
}

impl SentryLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            SentryLevel::Info => "info",
            SentryLevel::Warning => "warning",
            SentryLevel::Error => "error",
        }
    }
}

/// No-op capture implementation — compiles and links with no dependencies by default.
/// The `sentry` crate integration in `src-tauri` (Desktop) and `sentry_flutter` (Mobile)
/// will replace this at the binary level via conditional compilation or FFI wiring.
#[cfg(not(feature = "sentry-integration"))]
fn capture_message(event_type: &str, message: &str, level: SentryLevel) {
    eprintln!(
        "[telemetry] level={} event={} {}",
        level.as_str(),
        event_type,
        message
    );
}

/// Hook that binaries (src-tauri, relay-mobile-bridge) can call to override
/// the default no-op reporter with a real Sentry client.
#[cfg(feature = "sentry-integration")]
static SENTRY_HUB: std::sync::OnceLock<Option<sentry::ClientInitGuard>> =
    std::sync::OnceLock::new();

/// Initialise the Sentry integration for the `sentry-integration` feature.
/// Safe to call multiple times — only the first call takes effect.
#[cfg(feature = "sentry-integration")]
pub fn init_sentry(dsn: Option<&str>) {
    let _ = SENTRY_HUB.get_or_init(|| {
        let dsn = dsn.and_then(|s| if s.is_empty() { None } else { Some(s) });
        if let Some(dsn_str) = dsn {
            let guard = sentry::init((
                dsn_str,
                sentry::ClientOptions {
                    release: Some("relay@0.1.0".into()),
                    environment: Some(
                        std::env::var("SENTRY_ENV")
                            .unwrap_or_else(|_| "production".into())
                            .into(),
                    ),
                    ..Default::default()
                },
            ));
            Some(guard)
        } else {
            None
        }
    });
}

#[cfg(feature = "sentry-integration")]
fn capture_message(event_type: &str, message: &str, level: SentryLevel) {
    let sentry_level = match level {
        SentryLevel::Info => sentry::Level::Info,
        SentryLevel::Warning => sentry::Level::Warning,
        SentryLevel::Error => sentry::Level::Error,
    };
    sentry::capture_message(&format!("{event_type}: {message}"), sentry_level);
}

/// Optional helper that binaries can call to attach
/// the current device_id as a tag — never as a user identifier.
pub fn current_device_id() -> Option<String> {
    crate::config::get_device_id().ok().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_opt_out_default() {
        // If DB is not initialised, telemetry defaults to OFF (opted-out, safe privacy default).
        assert!(is_opted_out());
    }

    #[test]
    fn test_record_enrich_latency_does_not_panic() {
        let start = Instant::now();
        // Should not panic even if DB is unavailable.
        record_enrich_latency(start, true);
        record_enrich_latency(start, false);
    }

    #[test]
    fn test_record_sync_latency_does_not_panic() {
        let start = Instant::now();
        record_sync_latency(start, true);
    }

    #[test]
    fn test_sentry_level_as_str() {
        assert_eq!(SentryLevel::Info.as_str(), "info");
        assert_eq!(SentryLevel::Warning.as_str(), "warning");
        assert_eq!(SentryLevel::Error.as_str(), "error");
    }
}
