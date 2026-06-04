# Desktop Startup Performance — Sprint 13 Agent 7 Redo Report

**Branch:** `feat/sprint-13-production-hardening`  
**Date:** 2026-05-23  
**Scope:** Instrumentation + documentation of desktop Tauri startup path for cold- and warm-start profiling.

---

## 1. Instrumentation Points Added

### 1.1 `src-tauri/src/main.rs` (lines 54–339)

A single `StartupTimer` (wrapped in an `Arc`) is created at the top of `main()`.  It is stored in a module-level `OnceLock<Arc<StartupTimer>>` (`STARTUP_TIMER`) so the Tauri `RunEvent` loop can access the *same* timer later for the `window_visible` span.

Spans timed in `main()` / `run_setup()`:

| Span name | Where measured | Notes |
|-----------|----------------|-------|
| `app_start_total` | `main()` entry → just before `.run()` | Measures everything up to the event-loop start. Ends after `.setup()` returns. |
| `app_dir_resolve` | `app.path().app_data_dir()` | Usually <5 ms. |
| `db_init` | `db::set_data_dir()` (SQLite pool + schema) | Currently blocks `.setup()`; see bottleneck §3.1. |
| `config_init` | `config::init()` | Reads/creates local settings file. |
| `db_lancedb_init` | `db::vector::init_vector_db()` | LanceDB C++ backend `connect()` + table open. |
| `embedding_init` | `EmbeddingService::try_new()` (ONNX session build) | Can be 1–2 s on first run; see bottleneck §3.2. |
| `model_download_start` | Moment `std::thread::spawn` starts the background model loader | `end()` is called immediately inside the thread. |
| `model_swap` | Inside background thread: `LlamaService::try_load()` success/failure → RwLock swap | Includes llama.cpp backend init + GGUF mmap. |
| `tray_init` | `background::tray::build_tray()` | Usually <50 ms. |
| `watcher_init` | `background::watcher::start_watcher()` | Usually <50 ms. |
| `window_visible` | `tauri::RunEvent::WindowEvent::Focused(true)` on label `"main"` | Fires once only via `WINDOW_VISIBLE_FLAG` atomic. |

### 1.2 `src-tauri/src/performance.rs` (new file, 130 lines)

- **`StartupTimer`** — thread-safe `Mutex<Vec<Span>>` backed timer.
  - `start(name:&'static str)` / `end(name:&'static str)` with automatic `eprintln!("[perf] …")`.
  - `report()` prints a full human-readable table to stderr.
  - `summary()` builds a comma-separated string of all spans for easy breadcrumb ingestion.
  - `send_sentry_breadcrumb()` emits a `sentry::Breadcrumb` (category `"startup"`) **iff** telemetry is not opted-out.
- **`record_startup_telemetry(timer, model_cached)`**:
  - Prints the `report()` table unconditionally.
  - Fires `relay_core::telemetry::TelemetryEvent::AppStartCold { ms }` or `AppStartWarm { ms }`.
  - Calls `timer.send_sentry_breadcrumb()` so every startup attempt leaves a breadcrumb in Sentry when the DSN is configured.
- **`WINDOW_VISIBLE_FLAG`** — `AtomicBool` ensuring the `window_visible` span is recorded exactly once.

### 1.3 Sentry breadcrumb wiring

`relay-core` already defines `TelemetryEvent::AppStartCold / AppStartWarm` and a `sentry-integration` feature.  `performance.rs` uses the real `sentry` crate (already in `Cargo.toml`) to push a `Breadcrumb` with the compact span summary.  If the user has opted out, the breadcrumb is silently skipped.

---

## 2. Cold-Start vs Warm-Start Flow

### Cold start (no model cached)

```
main() ─┬──> app_dir_resolve  (~1 ms)
        ├──> db_init           (~200–800 ms)  ← SQLite pool + schema
        ├──> config_init       (~5–15 ms)
        ├──> db_lancedb_init   (~300–900 ms)  ← LanceDB C++ backend
        ├──> embedding_init    (~300–1800 ms) ← ONNX session build (may download)
        ├──> tray_init         (~30 ms)
        ├──> watcher_init     (~30 ms)
        │
        ├──> window_visible   (<2 s target if bottlenecks deferred)
        │
        └──> [background thread]
              ├──> model_download_start  (~500 MB download, minutes)
              └──> model_swap            (~2–3 s once cached)
```

### Warm start (model + embedding cached)

```
main() ─┬──> app_dir_resolve  (~1 ms)
        ├──> db_init           (~100–200 ms)
        ├──> config_init       (~5 ms)
        ├──> db_lancedb_init   (~200–400 ms)
        ├──> embedding_init    (~200–400 ms)  ← ONNX session rebuild
        ├──> tray_init         (~30 ms)
        ├──> watcher_init     (~30 ms)
        │
        ├──> window_visible   (~800–1200 ms)
        │
        └──> [background thread]
              ├──> model_download_start  (no-op, SHA-256 verified quickly)
              └──> model_swap            (~1.5–2.5 s, llama.cpp mmap)
```

**Targets:**
- Cold-start `window_visible` < 2 s (requires deferring LanceDB + ONNX init — see §4).
- Warm-start `LlamaService` ready < 3 s from `main()` entry.

---

## 3. Bottleneck List (from Static Code Review)

### 3.1 SQLite + FTS schema (`db_init`)
- `relay-core/src/db/mod.rs` line 211–214: `r2d2::Pool::builder().max_size(1).build(...)` creates the pool synchronously.
- On cold OS file cache this involves WAL creation, `migrations.rs` schema application (~150 lines of `CREATE TABLE / INDEX / FTS5`), and the first `PRAGMA journal_mode=WAL`.
- **Estimated impact:** 200–800 ms cold, 50–150 ms warm.

### 3.2 LanceDB C++ backend (`db_lancedb_init`)
- `src-tauri/src/db/vector.rs` line 71–76: `connect(...).execute().await` loads the native LanceDB core.
- Even when the `vectors.lance` directory exists, the SDK opens DuckDB metadata and can trigger arrow IPC scans.
- **Estimated impact:** 300–900 ms cold, 200–400 ms warm.

### 3.3 ONNX embedding session (`embedding_init`)
- `src-tauri/src/ai/embedding.rs` line 32–35: `Session::builder()?.commit_from_file(...)` compiles the ONNX graph for the CPU execution provider (MKL-DNN / DirectML on Windows, ARM Compute on macOS).
- The `ort` crate does **not** cache the compiled execution plan between process restarts.
- **Estimated impact:** 300–1800 ms cold (first download may expand to minutes), 200–500 ms warm.

### 3.4 llama.cpp backend + GGUF mmap (`model_swap`)
- `src-tauri/src/ai/llama_service.rs` line 31–33: `LlamaBackend::init()` (one-time global init) + `LlamaModel::load_from_file()` (mmap 500 MB file + graph build).
- `LlamaBackend::init()` is thread-safe per upstream docs but takes 500–800 ms on first call.
- **Estimated impact:** 2–3 s warm (background), 4–5 s cold if OS page cache is empty.

### 3.5 Model download (`model_download_start`)
- `src-tauri/src/ai/model_manager.rs` line 84–178: blocking `reqwest` download of ~500 MB to `&app_dir/models/`.
- Only happens on first run or after cache invalidation.
- Intentionally moved to a background thread so it **never blocks** `.setup()` or window visibility.

---

## 4. Sprint 14 Optimization Recommendations

| # | Recommendation | Owner | Effort | Impact |
|---|----------------|-------|--------|--------|
| 1 | **Defer LanceDB init to a background thread** — do not call `init_vector_db()` in `.setup()`. Instead, store the `app_dir` in a global and lazily open LanceDB on the first call to `store_vector`/`search_vectors`. | DB dev | 1 day | High (removes 300–900 ms from startup) |
| 2 | **Defer ONNX embedding init to a background thread** — start with `Option::None`; spawn a thread that downloads (if needed) and builds the `Session`. First `encode()` call blocks until ready. | AI dev | 1 day | High (removes 200–1800 ms from startup) |
| 3 | **Pre-warm `LlamaBackend` in parallel with DB init** — spawn a thread immediately after `.setup()` begins that calls `LlamaBackend::init()` (no model file needed). This overlaps the 500–800 ms global init with SQLite/LanceDB work. | AI dev | 0.5 day | Medium (shaves 500 ms off `model_swap`) |
| 4 | **Add a splash window / loading overlay** — Tauri can show a small "Relay" window in `.setup()` before the heavy init steps, then swap to the main window when `window_visible` fires. Improves *perceived* speed even if real latency stays the same. | Frontend dev | 2 days | Medium (user perception) |
| 5 | **Cache ONNX execution plan between runs** — investigate `ort` APIs for serialising the compiled session (or use a warm-up file). Currently blocked on `ort` crate support. | AI dev | 3 days | Low (blocked upstream) |
| 6 | **Real M1 Mac measurement** — ask a community volunteer to run a tagged build and paste the `eprintln` performance report into an issue. | Community / QA | 0.5 day | Info |
| 7 | **Telemetry audit** — confirm `sentry::Breadcrumb` ingestion works with the hosted Sentry project once DSN is configured. | Infra dev | 0.5 day | Low |

---

## 5. Files Changed

| File | Lines | Change |
|------|-------|--------|
| `src-tauri/src/main.rs` | 1–339 | New `STARTUP_TIMER` global; `run_setup()` extracted; timing spans around every startup phase; `handle_window_event()` for `window_visible`; `native_dialog_alert()` helper. |
| `src-tauri/src/performance.rs` | 1–130 | New `StartupTimer` struct with `start`/`end`/`report`/`summary`/`send_sentry_breadcrumb`; `WINDOW_VISIBLE_FLAG`; `record_startup_telemetry()`. |
| `docs/desktop-performance.md` | 1–164 | This report. |

---

## 6. Compile Evidence

```bash
$ cargo check -p relay
    Finished dev profile [unoptimized + debuginfo] target(s) in 3.05s

$ cargo clippy -p relay --all-targets -- -D warnings
    Finished dev profile [unoptimized + debuginfo] target(s) in 6.81s
```

No warnings, no errors. The crate compiles clean on the current branch.

---

## 7. Notes & Caveats

- **Real measurement pending community beta.**  The environment used for this task is a Windows development PC, not an M1 Mac.  Cold-start numbers above are *estimates* derived from static code review and the known behaviour of `llama-cpp-2`, `ort`, `lancedb`, and `rusqlite`.
- **Debug build:** Measured compilation is unoptimised.  Release builds (`cargo tauri build`) will improve llama.cpp kernel initialisation and ONNX runtime graph compilation times, but the relative bottleneck ranking should remain the same.
- **ONNX session caching:** The `ort` crate (2.0.0-rc.12) does not expose a stable API for serialising compiled sessions.  Recommendation #5 in §4 is exploratory only.
- **Telemetry opt-out:** `send_sentry_breadcrumb()` checks `relay_core::telemetry::is_opted_out()` before calling `sentry::add_breadcrumb`.  No Sentry traffic is generated when the user has opted out or when no DSN is configured.

---

*End of report.*
