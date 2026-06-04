use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Span {
    name: &'static str,
    start: Instant,
    end: Option<Instant>,
}

/// Lightweight thread-safe startup timer backed by a mutex-protected vector.
///
/// Each span is identified by a `&'static str` name.  If the same name is
/// started multiple times, `end()` only closes the *first* still-running
/// occurrence.
#[derive(Clone)]
pub struct StartupTimer {
    inner: Arc<Mutex<Vec<Span>>>,
}

impl StartupTimer {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Start a new span.
    pub fn start(&self, name: &'static str) {
        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        guard.push(Span {
            name,
            start: Instant::now(),
            end: None,
        });
    }

    /// End the first still-running span matching `name` and log its elapsed
    /// time to `eprintln`.
    pub fn end(&self, name: &'static str) {
        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        for span in guard.iter_mut() {
            if span.name == name && span.end.is_none() {
                span.end = Some(Instant::now());
                let elapsed = span.end.unwrap().saturating_duration_since(span.start);
                eprintln!("[perf] {}: {} ms", name, elapsed.as_millis());
                return;
            }
        }
    }

    /// Print a full table of all spans to stderr.
    pub fn report(&self) {
        let guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        eprintln!("=== Desktop Startup Performance Report ===");
        for span in guard.iter() {
            let elapsed = match span.end {
                Some(end) => end.saturating_duration_since(span.start),
                None => Instant::now().saturating_duration_since(span.start),
            };
            if span.end.is_none() {
                eprintln!(
                    "  {:<28} {:>7} ms (still running)",
                    span.name,
                    elapsed.as_millis()
                );
            } else {
                eprintln!("  {:<28} {:>7} ms", span.name, elapsed.as_millis());
            }
        }
        eprintln!("===========================================");
    }

    /// Return the duration of a completed span, if it exists.
    pub fn get_duration(&self, name: &'static str) -> Option<Duration> {
        let guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        guard
            .iter()
            .find(|s| s.name == name)
            .and_then(|s| s.end.map(|e| e.saturating_duration_since(s.start)))
    }

    /// Build a compact comma-separated summary of all spans for breadcrumb/logging use.
    pub fn summary(&self) -> String {
        let guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let parts: Vec<String> = guard
            .iter()
            .map(|span| {
                let elapsed = match span.end {
                    Some(end) => end.saturating_duration_since(span.start),
                    None => Instant::now().saturating_duration_since(span.start),
                };
                format!("{}={}ms", span.name, elapsed.as_millis())
            })
            .collect();
        parts.join(", ")
    }

    /// Send the collected spans as a Sentry breadcrumb when telemetry is enabled.
    /// Called automatically by `record_startup_telemetry` after the `.setup()`
    /// block finishes; the UI agent may also call it after `window_visible`.
    pub fn send_sentry_breadcrumb(&self) {
        if relay_core::telemetry::is_opted_out() {
            return;
        }
        let msg = self.summary();
        sentry::add_breadcrumb(sentry::Breadcrumb {
            category: Some("startup".into()),
            message: Some(msg),
            level: sentry::Level::Info,
            ..Default::default()
        });
    }
}

impl Default for StartupTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Set once when the main window becomes visible for the first time.
pub static WINDOW_VISIBLE_FLAG: AtomicBool = AtomicBool::new(false);

/// Run a function and record its elapsed time as a span on `timer`.
pub fn record<T>(timer: &StartupTimer, name: &'static str, f: impl FnOnce() -> T) -> T {
    timer.start(name);
    let result = f();
    timer.end(name);
    result
}

/// Record cold-start or warm-start telemetry using the built-in relay-core
/// telemetry module (eprintln-only when `sentry-integration` is off).
pub fn record_startup_telemetry(timer: &StartupTimer, model_cached: bool) {
    // Print the human-readable report to stderr regardless of telemetry state.
    timer.report();

    let ms = timer
        .get_duration("app_start_total")
        .unwrap_or_default()
        .as_millis() as u64;

    let event = if model_cached {
        relay_core::telemetry::TelemetryEvent::AppStartWarm { ms }
    } else {
        relay_core::telemetry::TelemetryEvent::AppStartCold { ms }
    };
    relay_core::telemetry::fire(event);

    // Also push the full span list to Sentry as a breadcrumb when available.
    timer.send_sentry_breadcrumb();
}
