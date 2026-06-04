#![allow(clippy::len_zero)]

pub mod ai;
pub mod background;
pub mod commands;
pub mod config;
pub mod db;
pub mod e2e_tests;
pub mod export;
pub mod performance;
pub mod sync;
pub mod telemetry;
pub mod web;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use ai::embedding::EmbeddingService;
use ai::embedding_model_manager;
use ai::fallback::FallbackService;
use ai::llama_service::LlamaService;
use ai::model_manager;
use ai::service::AIService;
use commands::account::{
    get_user_profile, get_user_profile_by_id, get_user_tier, set_user_email, set_user_tier,
};
use commands::auth::{create_account, get_auth_status, log_in, log_out};
use commands::capture::enrich_clipboard;
use commands::data_management::{clear_local_data, export_local_data};
use commands::highlight::{delete_highlight, get_history_paginated, list_stored_highlights};
use commands::model_status::{get_model_status, re_download_embedding_model, MODEL_LOADED};
use commands::review::{get_review_session, grade_review_item};
use commands::search::search;
use commands::stream::{
    add_to_stream, create_stream, delete_stream, export_stream_html, generate_stream_html,
    get_device_id, get_stream, get_stream_highlights, get_stream_public_url, list_my_streams,
    log_stream_page_view, log_stream_subscribe_click, remove_from_stream, share_stream,
};
use commands::subscribe::{
    get_subscriber_feed, get_subscriptions, is_subscribed_to_stream, subscribe_to_stream,
    unsubscribe_from_stream,
};
use commands::sync::{get_conflicts, get_sync_status, resolve_conflict, sync_now};
use commands::telemetry::{get_telemetry_opt_out, set_telemetry_opt_out, toggle_telemetry};
use performance::StartupTimer;
use tauri::{Emitter, Manager};

/// Whether the embedding service is unavailable (initialized as false).
pub static EMBEDDING_UNAVAILABLE: AtomicBool = AtomicBool::new(false);

/// Global handle to the shared AI service so background tasks (clipboard watcher)
/// can access it without holding a Tauri State reference.
static AI_SERVICE_GLOBAL: OnceLock<Arc<RwLock<Arc<dyn AIService>>>> = OnceLock::new();

/// Global handle to the optional embedding service for background indexing.
static EMBEDDING_SERVICE_GLOBAL: OnceLock<Option<Arc<EmbeddingService>>> = OnceLock::new();

/// Background init error surfaced to the first capture/search attempt.
static INIT_ERROR: OnceLock<Arc<Mutex<Option<String>>>> = OnceLock::new();

/// Global startup timer so the `.run()` event-loop closure can record
/// `window_visible` without creating a fresh timer.
static STARTUP_TIMER: OnceLock<Arc<StartupTimer>> = OnceLock::new();

/// Whether the background LanceDB + ONNX init thread has finished.
pub static VECTOR_INIT_COMPLETE: AtomicBool = AtomicBool::new(false);

/// Atomically take the background init error, if any, so it is surfaced only once.
pub fn take_init_error() -> Option<String> {
    INIT_ERROR
        .get()
        .and_then(|arc| {
            arc.lock()
                .ok()
                .and_then(|mut guard| guard.take())
        })
}

fn main() {
    let timer = Arc::new(StartupTimer::new());
    STARTUP_TIMER.set(timer.clone()).ok();
    timer.start("app_start_total");

    let timer_setup = timer.clone();
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(move |app| {
            let is_first_run = run_setup(app, timer_setup.clone())?;

            if is_first_run {
                if let Ok(user_id) = config::get_device_id() {
                    let _ = db::analytics::log_event(
                        "relay_install_complete",
                        None,
                        None,
                        Some(user_id),
                        None,
                    );
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            enrich_clipboard,
            get_model_status,
            list_stored_highlights,
            get_history_paginated,
            delete_highlight,
            search,
            create_stream,
            delete_stream,
            add_to_stream,
            remove_from_stream,
            get_stream,
            list_my_streams,
            get_stream_highlights,
            share_stream,
            generate_stream_html,
            export_stream_html,
            get_stream_public_url,
            log_stream_page_view,
            log_stream_subscribe_click,
            get_device_id,
            subscribe_to_stream,
            unsubscribe_from_stream,
            is_subscribed_to_stream,
            get_subscriber_feed,
            get_subscriptions,
            get_user_profile,
            get_user_profile_by_id,
            set_user_email,
            get_user_tier,
            set_user_tier,
            export_local_data,
            clear_local_data,
            create_account,
            log_in,
            log_out,
            get_auth_status,
            sync_now,
            get_sync_status,
            get_conflicts,
            resolve_conflict,
            get_telemetry_opt_out,
            set_telemetry_opt_out,
            toggle_telemetry,
            re_download_embedding_model,
            get_review_session,
            grade_review_item,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Gearbox Relay");

    // End the total-startup span now that `.setup()` has finished and the
    // event loop is about to start.
    timer.end("app_start_total");

    let app_dir = config::get_app_dir()
        .cloned()
        .unwrap_or_else(|_| std::path::PathBuf::new());
    let model_cached = model_manager::is_model_cached_global(&app_dir);
    performance::record_startup_telemetry(&timer, model_cached);

    app.run(|app_handle, event| {
        if let Some(t) = STARTUP_TIMER.get() {
            handle_window_event(app_handle, &event, t);
        }
        background::lifecycle::on_run_event(app_handle, event);
    });
}

/// Centralised setup routine so `main()` stays readable.
fn run_setup(
    app: &mut tauri::App,
    timer: Arc<StartupTimer>,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Resolve app data directory
    timer.start("app_dir_resolve");
    let dir = match app.path().app_data_dir() {
        Ok(d) => d,
        Err(e) => {
            let msg = format!("failed to resolve app data directory: {e}");
            native_dialog_alert(native_dialog::MessageType::Error, &msg)?;
            return Err(msg.into());
        }
    };
    timer.end("app_dir_resolve");

    // Create app data directory
    if let Err(e) = std::fs::create_dir_all(&dir) {
        let msg = format!("failed to create app data directory: {e}");
        native_dialog_alert(native_dialog::MessageType::Error, &msg)?;
        return Err(msg.into());
    }

    // Initialise SQLite pool + schema (timed as db_init).
    timer.start("db_init");
    db::set_data_dir(dir.clone());
    timer.end("db_init");

    // Initialise structured logging (tracing subscriber).
    // Writes to stderr. Level controlled by RUST_LOG env var (default: warn).
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_target(false)
        .try_init();

    // Load model manifest (bundled or runtime override)
    timer.start("manifest_init");
    let resource_dir = app.path().resource_dir().ok();
    let manifest = model_manager::load_manifest(
        resource_dir.as_deref(),
        &dir,
    );
    model_manager::init_manifest(manifest);
    timer.end("manifest_init");

    // Initialize config
    timer.start("config_init");
    let is_first_run = match config::init(&dir) {
        Ok(v) => v,
        Err(e) => {
            let msg = format!("failed to initialize config: {e}");
            native_dialog_alert(native_dialog::MessageType::Error, &msg)?;
            return Err(msg.into());
        }
    };
    timer.end("config_init");

    // Initialize telemetry after DB dir is ready but before heavy I/O.
    timer.start("telemetry_init");
    match std::env::var("SENTRY_DSN") {
        Ok(dsn) if !dsn.is_empty() => {
            crate::telemetry::init(Some(&dsn));
        }
        _ => {
            crate::telemetry::init(None);
        }
    }
    timer.end("telemetry_init");

    // -----------------------------------------------------------------------
    // Spawn background thread for LanceDB + ONNX init so the window appears
    // immediately. The main thread manages app state; init completes lazily.
    // -----------------------------------------------------------------------
    let bg_dir = dir.clone();
    let timer_bg = timer.clone();
    let init_error: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let init_error_bg = init_error.clone();
    INIT_ERROR.set(init_error.clone()).ok();

    std::thread::spawn(move || {
        let lance_result = performance::record(&timer_bg, "lancedb_init", || {
            db::vector::init_vector_db(&bg_dir)
        });
        if let Err(e) = lance_result {
            eprintln!("Warning: LanceDB initialization failed: {e}");
            if let Ok(mut guard) = init_error_bg.lock() {
                *guard = Some(format!("LanceDB init failed: {e}"));
            }
        }

        let emb_result = performance::record(&timer_bg, "embedding_init", || {
            embedding_model_manager::ensure_embedding_model(&bg_dir).and_then(
                |(model_path, tokenizer_path)| {
                    EmbeddingService::try_new(&model_path, &tokenizer_path)
                },
            )
        });

        match emb_result {
            Ok(es) => {
                let arc = Arc::new(es);
                EMBEDDING_SERVICE_GLOBAL.set(Some(arc.clone())).ok();
                // Set the global flag last so consumers see the service is ready.
                EMBEDDING_UNAVAILABLE.store(false, Ordering::SeqCst);
            }
            Err(e) => {
                eprintln!("Warning: Embedding service unavailable: {e}");
                EMBEDDING_UNAVAILABLE.store(true, Ordering::SeqCst);
                if let Ok(mut guard) = init_error_bg.lock() {
                    if guard.is_none() {
                        *guard = Some(format!("Embedding init failed: {e}"));
                    }
                }
            }
        }

        VECTOR_INIT_COMPLETE.store(true, Ordering::SeqCst);
    });

    // Placeholder state — background thread will swap the global when ready.
    app.manage(None::<Arc<EmbeddingService>>);

    // Start with fallback immediately so the app window appears
    let fallback: Arc<dyn AIService> = Arc::new(FallbackService);
    let shared = Arc::new(RwLock::new(fallback));
    app.manage(shared.clone());

    // Background: download models and swap to LlamaService when ready
    let bg_dir = dir.clone();
    let shared_bg = shared.clone();
    let timer_bg = timer.clone();
    let bg_manifest = model_manager::load_manifest(None, &bg_dir);
    let bg_app_handle = app.handle().clone();
    timer.start("model_download_start");
    std::thread::spawn(move || {
        timer_bg.end("model_download_start");

        // Spawn a sub-thread to emit download progress events to the frontend
        // while the main download runs in the outer thread.
        let progress_app = bg_app_handle.clone();
        let progress_running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let flag = progress_running.clone();
        std::thread::spawn(move || {
            use crate::ai::model_manager::{
                MODEL_DOWNLOAD_PROGRESS, MODEL_DOWNLOAD_STATE, MODEL_DOWNLOAD_TOTAL,
            };
            loop {
                let state = MODEL_DOWNLOAD_STATE.load(Ordering::SeqCst);
                if state != 1 && state != 0 {
                    // Download finished or errored — emit final event and exit
                    let total = MODEL_DOWNLOAD_TOTAL.load(Ordering::SeqCst);
                    let downloaded = MODEL_DOWNLOAD_PROGRESS.load(Ordering::SeqCst);
                    let pct = if total > 0 {
                        ((downloaded as f64 / total as f64) * 100.0) as u32
                    } else {
                        0
                    };
                    let _ = progress_app.emit(
                        "relay://model-progress",
                        serde_json::json!({
                            "percent": pct,
                            "downloaded_bytes": downloaded,
                            "total_bytes": total,
                            "state": if state == 2 { "done" } else { "error" },
                        }),
                    );
                    break;
                }
                let total = MODEL_DOWNLOAD_TOTAL.load(Ordering::SeqCst);
                let downloaded = MODEL_DOWNLOAD_PROGRESS.load(Ordering::SeqCst);
                let pct = if total > 0 {
                    ((downloaded as f64 / total as f64) * 100.0) as u32
                } else {
                    0
                };
                let _ = progress_app.emit(
                    "relay://model-progress",
                    serde_json::json!({
                        "percent": pct,
                        "downloaded_bytes": downloaded,
                        "total_bytes": total,
                        "state": "downloading",
                    }),
                );
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            flag.store(false, Ordering::SeqCst);
        });

        let result = model_manager::ensure_model(&bg_dir, &bg_manifest).and_then(|model_path| {
            eprintln!("Model found at: {}", model_path.display());
            LlamaService::try_load(&model_path)
        });

        // Wait for progress emitter to finish before completing
        while progress_running.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        timer_bg.start("model_swap");
        match result {
            Ok(ai) => {
                eprintln!("AI model loaded successfully, swapping from fallback.");
                MODEL_LOADED.store(true, Ordering::SeqCst);
                if let Ok(mut guard) = shared_bg.write() {
                    *guard = ai;
                }
            }
            Err(e) => {
                eprintln!("AI model unavailable, staying with fallback: {e}");
            }
        }
        timer_bg.end("model_swap");
    });

    // Manage auth state
    app.manage(Arc::new(RwLock::new(None::<commands::auth::AuthState>)));

    // Manage offline sync queue
    let queue = crate::sync::queue::OfflineSyncQueue::new();
    let queue_clone = Arc::clone(&queue);
    app.manage(queue);

    // Set globals for background watcher access
    AI_SERVICE_GLOBAL.set(shared.clone()).ok();
    // EMBEDDING_SERVICE_GLOBAL is set by the background init thread.

    // Wire the offline sync queue with a closure that performs sync
    let auth_for_queue = app
        .state::<Arc<RwLock<Option<commands::auth::AuthState>>>>()
        .inner()
        .clone();
    queue_clone.set_sync_fn(move || {
        let auth = {
            let guard = auth_for_queue
                .read()
                .map_err(|e| format!("Lock poisoned: {e}"))?;
            guard.as_ref().ok_or("Not authenticated")?.clone()
        };
        let client = std::sync::Arc::new(crate::sync::server::SyncServerClient::new(
            auth.server_url.clone(),
        ));
        let engine = crate::sync::engine::SyncEngine::new(client, auth.jwt, auth.encryption_key);
        engine.sync_now().map(|_| ())
    });

    // Build tray and start clipboard watcher
    let handle = app.handle().clone();

    timer.start("tray_init");
    if let Err(e) = background::tray::build_tray(&handle) {
        eprintln!("Tray setup failed: {e}");
    }
    timer.end("tray_init");

    timer.start("watcher_init");
    if let Err(e) = background::watcher::start_watcher(&handle) {
        eprintln!("Watcher setup failed: {e}");
    }
    timer.end("watcher_init");

    Ok(is_first_run)
}

/// Window-event helper that records the first `window_visible` event.
fn handle_window_event(
    _app_handle: &tauri::AppHandle,
    event: &tauri::RunEvent,
    timer: &StartupTimer,
) {
    if let tauri::RunEvent::WindowEvent {
        label,
        event: tauri::WindowEvent::Focused(true),
        ..
    } = event
    {
        if label == "main" && !performance::WINDOW_VISIBLE_FLAG.swap(true, Ordering::SeqCst) {
            timer.start("window_visible");
            timer.end("window_visible");
            eprintln!("[perf] window_visible: main window is now focused and visible");
        }
    }
}

/// Thin wrapper around `native_dialog` so we don't repeat the boiler-plate.
fn native_dialog_alert(
    ty: native_dialog::MessageType,
    text: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    native_dialog::MessageDialog::new()
        .set_type(ty)
        .set_title("Startup Error")
        .set_text(text)
        .show_alert()?;
    Ok(())
}
