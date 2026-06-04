# Sprint 6: Desktop Habit Loop & Sync Resilience
**Technical Specification · No Half-Measures**
**Version 1.0 · May 2026**

---

## 1. Executive Summary

Sprint 6 transforms Relay from a "manual bookmarking tool" into an **ambient knowledge pipeline**. The clipboard background watcher, system tray integration, offline sync queue, and conflict apply-action are all production-hardened before launch.

Without these features, Relay requires the user to consciously open the app and click Capture every time. With them, knowledge capture becomes frictionless — the app runs silently in the tray, captures clipboard changes automatically, and syncs reliably even on flaky networks.

---

## 2. Scope

### 2.1 Clipboard Background Watcher
- Background thread polling the OS clipboard every 500ms via `arboard`.
- Deduplication by content hash (SHA-256 of text).
- Auto-capture when text changes, respecting the `auto_capture_enabled` setting.
- Skip if text is empty, whitespace-only, or identical to last capture.
- Reuse the existing `do_enrich()` logic (non-streaming) for background captures.
- Persist to SQLite + LanceDB on capture.

### 2.2 System Tray Integration
- Tray icon (32×32 solid color generated programmatically — no external asset needed).
- Left-click: open/raise main window.
- Right-click menu: Open Relay, Capture Now, Sync Now, Settings, Quit.
- Sync status indicator: tooltip shows "Synced," "Syncing…", or "N conflicts".
- On window close: minimize to tray instead of quitting.

### 2.3 Background Lifecycle
- **Graceful quit:** On `CloseRequested` or `ctrlc`, flush all pending SQLite writes and LanceDB vectors before exit.
- **Low battery (Windows):** Before each background AI enrichment, check `GetSystemPowerStatus`. If battery is <10% and not charging, skip AI and store raw text only. Resume when battery rises.
- **Sleep/wake:** Deferred to Phase 2 (requires OS-specific power event registration on each platform).

### 2.4 Offline Sync Queue
- When `sync_now()` fails with a network error, the sync report is queued for retry.
- Background retry thread wakes every 30s.
- Exponential backoff: 30s → 60s → 120s → 240s → 300s (cap).
- Max 10 retry attempts before marking as permanently failed (logs error, stops retrying).
- Queue stored in-memory (sufficient for MVP; persistent queue in Phase 2).

### 2.5 Conflict Resolution Apply-Action
- `accept_remote`: Deserialize `remote_version` JSON from the conflict row, apply it to the live record using the same logic as `sync::engine::apply_remote()`, then mark the conflict as resolved.
- `keep_local`: Mark the conflict as resolved (existing behavior).
- `manual_merge`: Deferred to Phase 2.

### 2.6 OS Notification on Background Capture
- `notify-rust` cross-platform notification.
- Title: "Relay captured".
- Body: One-sentence summary snippet (truncated to 80 chars).
- Clicking notification opens/raises the Relay window.

### 2.7 Frontend Auto-Capture Toggle
- SettingsPanel checkbox: "Auto-capture clipboard on copy".
- Stored in `sync_metadata` key `auto_capture_enabled`.
- Default: `true`.

---

## 3. Architecture

### 3.1 New Rust Modules

```
src-tauri/src/
  background/
    mod.rs           -- init(), shutdown(), globals for AI/embedding services
    watcher.rs       -- ClipboardWatcher thread, dedup, auto-capture trigger
    tray.rs          -- TrayIcon, menu, event handlers
    lifecycle.rs     -- RunEvent hooks, graceful quit, battery check
  sync/
    queue.rs         -- OfflineSyncQueue, retry scheduler, backoff logic
```

### 3.2 Global State for Background Thread

The background watcher thread cannot access Tauri-managed state directly. We store clones in `OnceLock` globals at app startup:

```rust
static AI_SERVICE_GLOBAL: OnceLock<Arc<RwLock<Arc<dyn AIService>>>> = OnceLock::new();
static EMBEDDING_SERVICE_GLOBAL: OnceLock<Option<Arc<EmbeddingService>>> = OnceLock::new();
static APP_HANDLE_GLOBAL: OnceLock<tauri::AppHandle> = OnceLock::new();
```

These are set in `main.rs` `.setup()` and read by the background thread.

### 3.3 Tray + Window Close Behavior

```
User clicks X on window
    → WindowEvent::CloseRequested
    → api.prevent_close()
    → window.hide()
    → Tray icon remains

User right-clicks tray → "Quit"
    → lifecycle::graceful_shutdown()
    → flush SQLite + LanceDB
    → std::process::exit(0)
```

### 3.4 Low Battery Check (Windows)

```rust
#[cfg(windows)]
fn is_battery_critical() -> bool {
    use winapi::um::winbase::GetSystemPowerStatus;
    use winapi::um::winbase::SYSTEM_POWER_STATUS;
    let mut status: SYSTEM_POWER_STATUS = unsafe { std::mem::zeroed() };
    unsafe {
        if GetSystemPowerStatus(&mut status) != 0 {
            status.BatteryFlag & 0x01 != 0 // Battery present
                && status.ACLineStatus == 0 // Not on AC
                && status.BatteryLifePercent <= 10
        } else {
            false
        }
    }
}
```

---

## 4. Exit Gates

| Gate | Requirement |
|---|---|
| `cargo test --workspace` | ≥115 tests (101 current + ~15 new) |
| `cargo clippy --all-targets -- -D warnings` | Clean |
| `pnpm test` | ≥90 tests (85 current + ~5 new) |
| **Manual E2E** | Copy text from browser → Relay notification → app not open → open app → highlight present with enrichment |
| **Airplane mode test** | Capture while offline → sync queue fills → restore network → auto-retry succeeds |
| **Battery test** | Windows laptop unplugged, battery <10% → background capture stores raw text only → plug in → AI resumes |

---

*This specification is the irrevocable source of truth for Sprint 6.*
