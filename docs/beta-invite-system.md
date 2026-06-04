# Beta Invite System — Operational Guide

**Version:** 1.0-beta  
**Date:** 2026-05-24  
**Scope:** First 10 beta users ("Alpha Ten")

---

## 1. Generating Invite Codes

### 1.1 Code Format

Invite codes are 12-character alphanumeric strings, generated manually or via a simple script:

```bash
# One-liner to generate a batch of codes
python3 -c "import secrets; [print(secrets.token_urlsafe(9)[:12].upper()) for _ in range(10)]"
```

Example codes: `RELAY7XK9A2B`, `BETA9PLZ4WQ`

### 1.2 Issuance Log

Track issued codes in a secure spreadsheet or password manager:

| Code | Issued To | Date | Platform | Redeemed |
|------|-----------|------|----------|----------|
| RELAY7XK9A2B | user@example.com | 2026-05-24 | macOS | No |

**Do not** store invite codes in plain text inside the repo or in public GitHub issues.

### 1.3 Redemption Flow

1. Download the beta installer from the private release page (link provided via email).
2. Launch the app. The onboarding modal will prompt for an invite code.
3. Enter the 12-character code. The app validates it locally (no server check required).
4. On successful validation, proceed to the onboarding carousel and the main app.

---

## 2. Onboarding Flow — Capture → Enrich → Publish Stream

The expected first-session journey is designed to show value in under 60 seconds.

### Step 1: Install & Onboarding (1 minute)

- App opens to the **4-screen onboarding carousel**:
  1. Welcome to Relay (what the app is)
  2. Local AI (privacy-first, on-device tagging)
  3. Sync & Share (optional multi-device sync, Stream publishing)
  4. Keyboard Shortcuts (`Cmd+Shift+C` for capture, `Cmd+K` for search)
- Optionally enter invite code if not redeemed at launch.
- Dismissal state is stored in `localStorage` (`relay_onboarding_seen`) / `SharedPreferences` (mobile).

### Step 2: First Capture (30 seconds)

- **Desktop:** Press `Cmd+Shift+C` or click the tray icon → "Capture Clipboard".
- **Mobile:** Open app → tap the floating capture button.
- Paste or type a highlight (e.g., a sentence from an article).

### Step 3: AI Enrichment (5–15 seconds)

- The on-device SLM (Qwen-3.5-0.8B) generates:
  - A **summary** (1–2 sentences)
  - **Tags** (3–6 keywords)
- If the model is still downloading or unloaded, the app falls back to the deterministic keyword extractor instantly.
- The enriched highlight is saved to the local SQLite + LanceDB database.

### Step 4: Browse History (optional)

- Open the **History** panel (desktop sidebar / mobile tab).
- View paginated highlights. Scroll to load more (`limit`/`offset` pagination).
- Search by tag or full-text via the search bar.

### Step 5: Publish a Stream (1 minute)

- Tap **New Stream**.
- Multi-select highlights from history.
- Add a Stream title (max 120 characters).
- Tap **Publish**. A local-only loopback URL is generated:
  - `http://127.0.0.1:{port}/stream/{id}`
- Share this URL with a subscriber. It is valid only while the app is running.

### Step 6: Subscriber Deep-Link (optional)

- A subscriber clicks `relay://subscribe/{id}` on their device.
- The app opens to the Stream preview and offers to subscribe.

---

## 3. Feedback Collection

### 3.1 GitHub Issues (Preferred)

Use the public client repository for bug reports and feature requests:

- **URL:** https://github.com/AWRMemo/gearbox-client/issues/new/choose
- **Templates:**
  - 🐛 Bug Report
  - 💡 Feature Request
  - 🐢 Performance Regression

**Required fields for bug reports:**
- Platform (macOS / Windows / Linux / iOS / Android)
- App version (found in Settings → About)
- Steps to reproduce
- Expected vs actual behavior
- Screenshots or screen recordings (if UI-related)

### 3.2 Discord (Informal / Quick Chats)

- **Invite link:** Provided separately to each beta user via welcome email.
- **Channels:**
  - `#beta-feedback` — general impressions
  - `#bugs` — confirmed or suspected bugs
  - `#performance` — latency, startup time, memory usage
  - `#streams` — publishing and subscription issues

### 3.3 Email (Private / Security Issues)

- **General beta support:** beta@gearbox.dev
- **Security vulnerabilities:** security@gearbox.dev (do not open public issues for security bugs)

---

## 4. Known Issues — Beta Limitations

The following issues are expected during the beta and do not indicate a broken build. They are tracked in `MISSING_OR_INCOMPLETE.md` (P1/P2 priority).

### P1 — Medium Priority (Should Close Before Public Beta)

| # | Issue | Impact | Workaround |
|---|-------|--------|------------|
| 5 | **LanceDB init blocks startup** | Cold-start window visible time increases by ~300–900 ms | None; background deferral planned |
| 6 | **ONNX embedding init blocks startup** | Cold-start window visible time increases by ~200–1800 ms | None; background deferral planned |
| 7 | **Mobile AI lacks real-device validation** | iOS MLX and Android MediaPipe compile in CI but are unverified on physical hardware | Test on your device and report latency/memory |
| 8 | **Desktop M1 performance unmeasured** | No Apple Silicon hardware in CI; numbers extrapolated | macOS beta users: paste `eprintln` perf reports |
| 9 | **PRD §17 embedding engine mismatch** | Documentation still references Candle + Granite; actual is ONNX + all-MiniLM-L6-v2 | See `docs/embedding-engine-decision.md` |
| 10 | **Mobile `ui-standards.md` missing** | No formal mobile UI guidelines document | N/A (internal docs) |
| 11 | **No automated Qwen GGUF download** | Users must manually place the model file in `src-tauri/models/` | Manual download instructions provided |
| 12 | **`sync_now()` requires re-login** | After `create_account`, sync spawns in background; user may need to log in again for it to take effect | Log out and log back in once after account creation |

### P2 — Low Priority (Polish & Tech Debt)

| # | Issue | Impact | Workaround |
|---|-------|--------|------------|
| 13 | **Zero mobile UI automated tests** | Flutter widget/integration tests not yet written | Manual testing only |
| 14 | **Server test coverage <80%** | Missing blob lifecycle, rate-limit edge cases, device token tests | N/A (engineering backlog) |
| 15 | **No conflict resolution e2e test** | Sync conflicts resolved correctly in unit tests, but no full end-to-end validation | N/A (engineering backlog) |
| 17 | **Embedding model CDN has no fallback** | Single Hugging Face download URL; no mirror or retry | Retry manually if download fails |
| 18 | **ONNX uses CPU only** | No Metal (macOS) or CUDA (Windows/Linux) GPU acceleration | CPU inference is acceptable for 30M-parameter model |
| 21 | **Server lacks `protocol_version` column** | Dual-protocol v1/v2 migration path untested server-side | N/A (engineering backlog) |
| 23 | **No load-test for rate limiting** | Tower rate limit tested in unit tests only, not under load | N/A (engineering backlog) |
| 24 | **Deep-link `id` validation server-side** | Validation exists client-side only; server accepts any path segment | N/A (security backlog) |
| 25 | **No Windows installer code-signing** | Unsigned Windows installer may trigger SmartScreen warnings | Click "More info → Run anyway" |

### Known UI/UX Gaps

- **Desktop Sentry dynamic toggle:** Enabling telemetry in Settings requires an app restart to take effect.
- **Mobile telemetry FRB:** Wired but Flutter Settings screen toggle is stubbed on some builds.
- **Onboarding modal on mobile:** Uses `SharedPreferences` gate; uninstalling resets it.

---

## 5. Success Metrics to Track

### 5.1 k-Factor (Viral Growth Loop)

These events are instrumented from day 1. Beta users should trigger each at least once:

| Event | Trigger Point | Target |
|-------|---------------|--------|
| `relay_install_complete` | App first launch | 100% of invitees |
| `first_highlight_captured` | First successful capture + enrich | >80% of invitees within 24 h |
| `stream_published` | User taps "Publish" on a Stream | >30% of invitees within 7 days |
| `stream_page_view` | A subscriber opens the loopback URL | >2 views per published Stream |
| `stream_subscribe_click` | A subscriber taps "Subscribe" | >1 click per published Stream |

**k-Factor formula:**
```
k = (stream_subscribe_click count) / (stream_published count)
```
Target for beta: `k > 0.3`.

### 5.2 Engagement

| Metric | Source | Target |
|--------|--------|--------|
| **DAU/MAU ratio** | Telemetry opt-in cohort | >40% |
| **Capture latency** | `EnrichLatency` telemetry event (desktop) | <2 s median (warm model), <5 s (cold start) |
| **Sync success rate** | `SyncAttempt` telemetry event | >95% |
| **AI parse yield** | `relay-core` quality log | >98% |

### 5.3 Performance

| Metric | Source | Target |
|--------|--------|--------|
| **Cold-start `window_visible`** | `StartupTimer` spans in stderr logs | <2 s with deferred LanceDB/ONNX (Sprint 15) |
| **Warm-start model ready** | Background `model_swap` span | <3 s from `main()` entry |
| **Memory peak (desktop)** | OS task manager / `top` | <1.5 GB with model loaded |
| **Memory peak (mobile)** | Xcode / Android Studio profiler | <500 MB (iOS), <600 MB (Android) |

### 5.4 Reporting Metrics

Beta users are **not required** to self-report metrics. If telemetry is enabled, they are collected automatically. If opted out, we rely on:

- GitHub issue labels: `performance`, `latency`, `memory`
- Discord `#performance` channel screenshots
- Direct email with attached `eprintln` logs (desktop) or Flutter logs (mobile)

---

## 6. Beta Closeout

After the first 10 users have been active for 14 days:

1. Review all GitHub issues tagged `beta-15`.
2. Prioritise P0/P1 gaps for Sprint 15 closure.
3. Decide on expanding the beta to 50 users or moving to open release.
4. Update this document with actual observed metrics and revised thresholds.

---

*End of Beta Invite System guide (Sprint 15 P1).*
