# Gearbox Relay — Development Reference

Local-first, AI-native personal knowledge pipeline. Capture text highlights → on-device SLM enriches with tags/summary → searchable knowledge base. Publish curated "Streams" that others subscribe to.

## Build & Development Commands

```bash
# Desktop (Tauri)
cargo tauri dev                       # Dev server with hot-reload (port 1420)
cargo tauri build                     # Production bundle
cargo clippy --all-targets -- -D warnings  # Lint (must pass)
cargo fmt -- --check                  # Format check
cargo test --workspace                # Full Rust test suite

# Frontend (src/)
pnpm install                          # Install dependencies
pnpm dev                              # Vite dev server (standalone)
pnpm build                            # Production build
pnpm lint                             # ESLint + Prettier check
pnpm test                             # Vitest unit tests
```

## Project Structure

```
relay/
  src-tauri/          # Rust backend: Tauri commands, AI pipeline, LanceDB, sync
    src/
      main.rs         # App entry, tray, lifecycle
      commands/       # #[tauri::command] handlers (thin; delegate to services)
      ai/             # Model loading, prompt templates, output parsers, fallbacks
      db/             # SQLite schema, migrations, LanceDB wrapper
      sync/           # LWW object sync protocol (no CRDT)
      models/         # GGUF model registry manifest
  src/                # React frontend: capture UI, Stream viewer, search
  relay-core/         # Shared Rust library: domain types, AI trait, DB, sync engine
  relay_mobile/       # Flutter app (iOS + Android)
  relay-bench/        # AI quality benchmark harness
```

## Code Style

**Rust:** `#[tauri::command]` functions return `Result<T, String>`. Use `thiserror` for domain errors, convert to `String` at command boundary. All AI operations return tokens via `tauri::ipc::Channel<String>` (never block the UI via `invoke`).

**React:** Named exports only. Components under 200 lines. State in local hooks; no global store. Use `@tauri-apps/api` types, not raw `any`.

**Crucial convention:** AI output parsers must use multi-layer defensive parsing — strip markdown fences first, validate field count, fall back to deterministic keyword extraction on any parse failure.

## Architecture Decisions (Locked)

- **Model-agnostic:** AI tier is abstracted behind a service trait. Default model: Qwen-3.5-0.8B.
- **LWW sync, not CRDT:** Single-user multi-device. Conflict resolution: higher `last_modified` timestamp wins.
- **Subscription-first:** Free tier has genuine value. Pro/Annual/Creator tiers gated by Stream count, device count, and Themed Reviews — never AI quality.
- **Privacy posture:** Core client is Apache 2.0 open-source. Five published privacy guarantees.
- **AI runtime separation:** Desktop AI runs via llama-cpp-2 (GGUF). Mobile AI uses MLX Swift (iOS) or MediaPipe (Android).

## Sprint History

### Sprint 11 — End-to-End Mobile Loop
Capture UI, enriched history, Stream curation, local HTTP share, deep-link subscribe, push notifications, cross-device sync validation.

### Sprint 12 — Real AI: From Fallback to Production
Desktop Qwen-3.5-0.8B GGUF via llama-cpp-2, ONNX embedding, AI parser hardening (40 tests), iOS MLX Swift spike, Android MediaPipe spike, onboarding UI, toast system, history pagination, CI split.

### Sprint 13 — Production Hardening & Beta Readiness
Desktop AI wiring, embedding production (SHA256), AI quality validation (Qwen 3.0/3.0 vs Fallback 1.8/3.0), mobile CI, sync security PRD, telemetry integration, performance profiling, beta polish (carousel, empty states, shortcuts).

### Sprint 14 — Security, Resilience & Hardening
OpaqueBlob v2 crypto primitives, AI OOM resilience, AI quality monitor, telemetry wiring, auth state fix, mobile Sentry, server test coverage (0→14), server rate limiting.

### Sprint 15 — Sync v2 Integration & Public Beta Prep
OpaqueBlob v2 wired into SyncEngine, server v2 endpoints, dynamic Sentry toggle, deferred LanceDB/ONNX init, mobile telemetry FRB, public beta docs, public repo snapshot.

### Sprint 16 — Mobile AI Integration
iOS MLX integration, Android MediaPipe integration, Flutter model download manager, typed Dart enrichment API with 5-layer parser.

### Sprint 17 — Server & Infrastructure
Public stream hosting, Tauri auto-updater config, release CI (Windows/macOS/Linux), GGUF download scripts, server schema migration, legal compliance package (GDPR, CCPA, SOC2), iOS Share Extension.

### Sprint 18 — Desktop Alpha
System tray + minimize-to-tray, sleep/wake lifecycle, server blob lifecycle tests, search UI, connection suggestion UI, following feed + subscriptions, data export (Markdown + JSON), dark mode + a11y, E2E Playwright tests.

### Sprint 19 — Desktop Launch Readiness
Model agility infrastructure (manifest registry, model-name-aware quality monitor), system tray polish, stream export sharing, review sessions, Chrome extension store-ready, telemetry dynamic toggle, tracing structured logging, v1→v2 sync auto-migration, r2d2 pool concurrency.

## Boundaries — NEVER

- **NEVER** use a cloud AI API (OpenAI, Anthropic, etc.). All AI runs locally.
- **NEVER** commit `.env` files or credentials.
- **NEVER** modify `src-tauri/src/ai/fallback.rs` without updating its unit tests.
- **NEVER** introduce Yjs or any CRDT library.
- **NEVER** bind local servers to `0.0.0.0` — use `127.0.0.1:0` only.
- **NEVER** change the sync encryption scheme (AES-256-GCM, Argon2id key derivation) without a full security review.
