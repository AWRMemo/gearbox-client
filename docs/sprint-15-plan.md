# Sprint 15 — Sync v2 Integration & Public Beta

**Branch:** `feat/sprint-15-sync-v2-beta` (or work directly on `main` pre-launch)
**Date:** May–June 2026
**Duration:** 3 weeks
**Agents:** 6
**Theme:** Close the last critical security gap (OpaqueBlob v2), wire remaining mobile bridges, and ship the public beta.

---

## Objective

After 14 sprints, Relay is feature-complete for a privacy-first knowledge pipeline. The **only remaining blocker to public beta** is the sync metadata leak (`SEC-12-SYNC-METADATA`). The v2 crypto layer exists in `relay-core/src/sync/opaque_blob.rs` but `SyncEngine` still uses v1 `EncryptedBlob`.

This sprint wires OpaqueBlob v2 into the SyncEngine, patches remaining P1 gaps, and executes the public beta launch checklist.

---

## Deliverables

| # | Deliverable | Agent | Prio | Est. Days | Risk | Test Requirement |
|---|-------------|-------|------|-----------|------|------------------|
| 1 | **Wire OpaqueBlob v2 into `SyncEngine`** | Agent 1 | **P0** | 5 | Medium | ≥10 unit tests for dual-protocol logic (v1 read, v2 write, downgrade shim). Integration test: offline capture → online sync → verify server receives `OpaqueBlob` with no plaintext metadata. |
| 2 | **`SyncEngine` dual-protocol compatibility** | Agent 1 | **P0** | 3 | Medium | Must read v1 blobs, write v2, handle downgrade gracefully. No v1-only path allowed after this sprint. |
| 3 | **Server schema migration for v2** | Agent 2 | **P0** | 3 | Low | Test both v1-only DB and v2-ready DB; migration must be zero-downtime for existing users. |
| 4 | **Mobile telemetry FRB bridge** | Agent 3 | P1 | 2 | Low | FRB roundtrip test + Dart widget test for Settings toggle. |
| 5 | **Desktop Sentry dynamic toggle** | Agent 4 | P1 | 2 | Low | Restart prompt or Sentry re-init on toggle ON without restart. |
| 6 | **Defer LanceDB + ONNX init to background** | Agent 5 | P1 | 3 | Medium | `StartupTimer` must show <500ms improvement after background deferral (warm start). |
| 7 | **Public beta checklist** | Agent 6 | P1 | 3 | Low | Privacy policy, open-source repo public snapshot, DPA template, beta invite system. |
| 8 | **`MISSING_OR_INCOMPLETE.md` audit + close P0s** | All | **P0** | 1 | Low | Sprint closeout procedure per AGENTS.md. |

---

## Agent Assignments

| Agent | Expertise | Primary Files |
|---|---|---|
| **Agent 1** — Sync v2 Lead | Rust, crypto, protocol design | `relay-core/src/sync/engine.rs`, `relay-core/src/sync/opaque_blob.rs`, `relay-core/src/sync/integration_tests.rs` |
| **Agent 2** — Server Backend | Rust, SQLite, migrations | `relay-sync-server/src/db.rs`, `relay-sync-server/src/handlers.rs`, `relay-sync-server/tests/` |
| **Agent 3** — Mobile Bridge | Flutter, FRB, Kotlin/Swift glue | `relay_mobile/rust/src/api/relay_api.rs`, `relay_mobile/lib/services/telemetry_service.dart` |
| **Agent 4** — Desktop Telemetry | Rust, Sentry, Tauri lifecycle | `src-tauri/src/telemetry.rs`, `src/components/SettingsPanel.tsx` |
| **Agent 5** — Performance | Rust, async, background threads | `src-tauri/src/performance.rs`, `src-tauri/src/main.rs`, `src-tauri/src/db/` |
| **Agent 6** — Launch Ops | Docs, CI, legal infra, community | `docs/`, `.github/workflows/`, `PRD.md` |

---

## Key Constraints

- **Never use a cloud AI API.** All AI runs on-device. Desktop uses llama-cpp-2 (GGUF). Mobile may use Cactus SDK or MLX Swift.
- **Never commit `.env` files or files containing `GITHUB_TOKEN`, `CACTUS_API_KEY`, or encryption keys.**
- **Never introduce Yjs or any CRDT library.** We use simple LWW timestamp-based sync for v1.
- **Never change the sync encryption scheme** (AES-256-GCM, key derived from user password) without a full security review and PRD update.
- **Never send plaintext user data to the sync server.** All sync payloads are encrypted blobs.
- **HTTP Server Loopback Security Rule:** Any local HTTP server started for Stream sharing MUST bind to `127.0.0.1:0` (loopback-only, OS-assigned port).

---

## Sprint 15 → Public Beta Gate

After Sprint 15 merges to `main`:

1. `cargo test --workspace` → 100% pass (0 failures, ignored tests only)
2. `cargo clippy --workspace --all-targets -- -D warnings` → clean
3. `pnpm test` → 92/92 pass
4. `flutter analyze --fatal-infos` → clean (CI validates)
5. **Sync metadata audit:** scan `relay-core/src/sync/` for any plaintext `id`/`record_type`/`last_modified` outside ciphertext → must be zero
6. **Tag `beta-v1.0`** on `main`
7. **Snapshot public repo** per AGENTS.md dual-repo procedure
8. **Beta invites:** 10 users, 1-week feedback window

---

## Sprint 16+ Tentative (Post-Beta)

Based on beta feedback, ranked by likely impact:

1. **Themed Review sessions** — AI-curated spaced-repetition feed
2. **Data export** — Markdown, JSON (low engineering cost, high user request)
3. **Creator tier monetization** — Stripe integration, analytics dashboard
4. **Reputation system** — subscriber counts, trending Streams
5. **Browser extension** — Chrome/Safari capture (major growth lever)
6. **Mobile AI physical-device validation** — iOS MLX, Android MediaPipe on real hardware

---

## Context

### Current State (End of Sprint 14)

**Desktop:** capture, enrich, search, Stream publishing, sync v1, telemetry, onboarding, keyboard shortcuts, empty states, model status badge, export/clear data, local HTTP server (`127.0.0.1:0`), clipboard watcher with retry, OOM resilience, quality monitor.

**Mobile:** Flutter capture, history, Stream curation, sync, push notifications, deep-links, onboarding, Sentry wiring. AI runtime bridged via FRB. Compiles in CI (`flutter analyze` clean).

**Server:** auth (register/login/refresh), blob storage, rate limiting, device tokens, 14 tests. Schema has `refresh_token_hash` and `refresh_token_expires_at`.

**AI:** Qwen 3.5 GGUF (desktop), ONNX all-MiniLM-L6-v2 embedding, multi-layer defensive parser (40 parser tests), quality monitor with degrade trigger.

**Critical Gap:** Sync metadata leak (`SEC-12-SYNC-METADATA`). `EncryptedBlob` sends `id`, `record_type`, `last_modified` outside ciphertext. `OpaqueBlob` v2 crypto primitives complete (AES-256-GCM with AAD, 4 unit tests) but `SyncEngine` integration is the remaining blocker.

### Why This Sprint Is the Beta Unblocker

The PRD §9 monetization gates (Pro/Annual/Creator tiers) and §8 k-factor targets cannot be measured without real users. Real users require a privacy guarantee. The sync metadata leak breaks Guarantee #2 ("Sync data is end-to-end encrypted; server stores only ciphertext"). Fixing this is the last technical prerequisite for public beta.

---

## Notes

- **Staging branch:** Given we are pre-launch, this sprint may work directly on `main` or use lightweight feature branches. The AGENTS.md staging-branch procedure (`feat/sprint-<n>-<name>`) was designed for post-launch stability; adapt as needed.
- **CI:** All PRs must include test evidence (`cargo test`, `pnpm test`, `flutter analyze` output) in the description.
- **Documentation:** Update `PRD.md` §17 and `AGENTS.md` Sprint 15 section as deliverables complete.
