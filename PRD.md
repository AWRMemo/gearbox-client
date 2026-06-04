# Gearbox Relay – Product Requirements Document

**Version 2.0 (Audited & Locked) | May 2026**

---

## 1. Executive Summary

Gearbox Relay is a local-first, AI-native personal knowledge pipeline. It captures any text a user highlights, copies, or screenshots across devices, enriches it with on-device AI (tagging, summarization, semantic indexing), and makes it instantly searchable. The product's core growth loop—publishing curated "Streams" that others subscribe to—creates a two-sided referral engine structurally embedded in the knowledge workflow.

Relay is built for the shift from Personal Knowledge Management (PKM) to Personal Context Management (PCM), where the bottleneck is no longer organizing information but curating context for AI to use on a user's behalf. It runs AI entirely on-device, never touching a cloud API for core features, and offers verifiable, guaranteed privacy as its primary differentiator.

---

## 2. Vision & Strategic Intent

**Product Vision:** Become the default operating system for personal context—the private, intelligent layer between how people consume information and how they apply it.

**North Star Metric:** Weekly Active Stream Subscribers (a composite measure of curation, consumption, and viral distribution).

**Why 2026–2027:**

- Personal knowledge base AI market: $2.16B (2026), growing at 30.3% → $6.15B by 2030.
- On-device SLMs are now capable of high-quality tagging and summarization at <200 MB footprint.
- Zero-click searches hit 69%; traditional paid acquisition is collapsing. Products must grow virally, structurally.
- EU AI Act enforcement delayed to Dec 2027; privacy-first on-device AI has a regulatory head start.
- Creator economy shifted from courses to communities: recurring membership is the dominant monetization.

---

## 3. Market & Competitive Landscape

### 3.1 Market Sizing

- **TAM:** 1.2+ billion global internet users who regularly read to learn.
- **SAM:** ~400 million active users of read-later and note-taking tools.
- **SOM (5-year target):** 10 million MAU, 500k paying subscribers.

### 3.2 Competitive Gap

| Competitor | AI Features | AI Cost Model | Privacy | Growth Loop |
|---|---|---|---|---|
| Readwise | Cloud GPT-4 | Subscription + token costs | Data uploaded | Word-of-mouth |
| Notion AI | Cloud-based | $10/mo add-on | All data in cloud | Team invites (B2B) |
| Obsidian | Limited (plugins) | N/A (local) | Local-first | Plugin ecosystem |
| Pocket | None | N/A | Cloud sync | Instapaper sharing |
| **Relay** | Full on-device SLM + RAG | Zero token cost | 100% local, verifiable | Two-sided Stream subscription loop |

**Uncontested Space:** No competitor combines private, zero-cost AI with a structural viral growth loop.

---

## 4. Target Personas

### 4.1 The AI-Augmented Knowledge Worker

Researchers, analysts, engineers, founders.

**Need:** capture insights from 100+ sources, resurface contextually, share curated summaries.

**Pain:** current tools either lack AI or send private reading habits to the cloud.

### 4.2 The Intellectual Influencer

Newsletter authors, niche experts building paid communities.

**Need:** a live, monetizable curation feed.

**Pain:** standalone courses have <15% completion; need evolving, recurring value.

### 4.3 The Privacy-Conscious Learner

Medical/law students, journalists, activists.

**Need:** AI-assisted study with verifiable privacy.

**Pain:** institutional policies forbid cloud AI; current tools non-compliant.

---

## 5. Core User Journeys

### Journey 1: Capture & Auto-Enrich (Habit Loop)

1. User highlights text in any app or browser.
2. Relay's clipboard watcher/share extension instantly captures it with source metadata.
3. On-device SLM generates: one-sentence summary, 3–5 tags, one suggested connection to existing highlights.
4. User opens Relay to find enriched, searchable, connected knowledge.

### Journey 2: Stream Publishing (Growth Engine)

1. User assembles a curated collection (a "Stream") on a topic.
2. Publishes it → Relay generates a shareable, auto-updating web page with AI summaries.
3. Non-users visit, see value, and click "Subscribe" → prompted to install Relay.
4. On install, the Stream syncs into their local knowledge base.
5. New user starts capturing, eventually publishes their own Stream.

### Journey 3: AI-Powered Themed Review (Retention)

1. User opts into daily "Themed Review" sessions.
2. On-device SLM selects a thematic cluster of highlights and presents them in a spaced-repetition-style feed.
3. User re-engages, adds new thoughts, optionally shares insights.

---

## 6. Technical Architecture (Audited & Locked)

### 6.1 Core Principles

- **Local-first:** all data lives on-device. Sync is background-only, never user-blocking.
- **AI without APIs:** enrichment uses on-device models; zero cloud inference, zero latency, full privacy.
- **Model-agnostic:** SLM selection is abstracted; models can be swapped without application changes.
- **Resilience:** deterministic fallback for all AI outputs.

### 6.2 Technology Stack

| Layer | Desktop (MVP) | Mobile (Phase 2) |
|-------|---------------|------------------|
| **Client** | Tauri 2.0 (Rust + React) | Flutter (primary), Kotlin Multiplatform (spike candidate) |
| **AI Runtime** | llama-cpp-2 (GGUF) | iOS: MLX Swift; Android: MediaPipe (Cactus SDK deferred) |
| **Embedding Model** | ONNX Runtime + all-MiniLM-L6-v2 (384-dim) | Same model; platform-specific ONNX Runtime or native inference |
| **Generation Model** | Qwen 3.5 0.8B Q4_K_M GGUF | Same model; platform-specific runtime |
| **Vector DB** | LanceDB (or SQLite FTS5 for v1) | LanceDB or platform-native |
| **Sync** | LWW object sync (Rust) | Same sync protocol, reused via shared Rust library |
| **Key Principle** | **All AI runs on-device. Zero cloud tokens. Full privacy.** | Same. |

#### 6.2.1 Architecture Strategy Note

Desktop MVP proves the engine. The entire core—AI pipeline, database, sync, Stream logic, analytics—is written in portable Rust and will be compiled as a library for mobile. The desktop React UI is a thin shell and will be replaced by Flutter on mobile. Mobile is the primary growth surface; the desktop client serves as a development testbed and a secondary target for power users.

### 6.3 AI Pipeline Per Highlight

1. **Pre-processing:** extract source metadata, clean text, chunk if >512 tokens.
2. **Embedding:** `all-MiniLM-L6-v2` via ONNX Runtime generates a 384-dim vector, L2-normalised.
3. **Tag & Summary (SLM):** prompt unambiguously; parse with multi-layer defense; fallback to keyword extraction on format failure.
4. **Connection Suggestion:** LanceDB top-3 similar highlights; SLM writes bridging sentence.
5. **Storage:** write to SQLite + LanceDB with timestamp and sync status.

**Failure Mitigations (from arXiv:2604.24636):**

- Strip markdown fences before parsing.
- Validate tag count; if out of range, fall back.
- Session rotation: fresh context per capture; no KV cache carry-over.
- Timeout at 2 seconds → deterministic fallback.
- Embedding service failure → zero-vector placeholder in LanceDB; search falls back to keyword-only.

### 6.4 Sync Architecture (LWW)

```
CLIENT: SQLite highlights table
  - id (UUID), content, tags, summary, last_modified, sync_status

Sync Engine (background):
  1. Push: SELECT where sync_status='local' AND last_modified > last_sync
  2. Pull: GET /sync?since={timestamp}
  3. Merge: per object, keep version with higher last_modified
  4. Conflict log: user-visible, non-blocking (rare in single-user case)

SERVER:
  - Encrypted blobs only (AES-256-GCM, key derived from user password)
  - Stores: user auth, encrypted highlight blobs, Stream metadata
  - Never stores: plaintext highlights, vectors, reading history
```

### 6.5 Streaming AI Output (Tauri channel API)

All generation commands use `Channel<String>` for token-by-token output, preventing UI freezes. Frontend renders tokens as they arrive.

### 6.6 Background Process Lifecycle (Desktop)

| Scenario | Behavior |
|---|---|
| Minimized to tray | Clipboard watcher continues; AI deferred to idle |
| System sleep | Watcher pauses; clipboard scanned on wake (best-effort) |
| Low battery (<10%) | All AI paused; raw captures stored |
| App quit | Graceful flush to SQLite + LanceDB |
| Update | Preserve data files; schema migration on next launch |

---

## 7. Feature Breakdown

### Phase 1: MVP (Weeks 1–4)

- Clipboard watcher (desktop) + share extension (mobile)
- Manual highlight input
- Source extraction (URL, title, author)
- On-device AI enrichment: tag, summary, connection suggestion (with fallback)
- Local full-text + semantic search (LanceDB)
- Offline-only mode (no account required for core use)
- Stream creation & publishing (auto-updating web page)
- Subscription mechanism (web → app install funnel)
- Account system (email-based, minimal)
- Subscriber feed in-app
- k-factor measurement instrumentation
- Subscription paywall infrastructure (Pro triggers)

### Phase 2: v1.0 (Weeks 5–8)

- Themed Review sessions (AI-curated)
- Pro tier: unlimited Streams, advanced AI (Tier 3), multi-device sync, analytics
- Creator tier: monetizable Streams, analytics, verified badge
- Data export (Markdown, JSON)
- Reputation system (subscriber counts, trending Streams)

### Phase 3: Post-MVP

- Federated personalization (DP-FedLoRA)
- Collaborative Streams
- Voice capture
- Browser extensions, Readwise/Pocket import

---

## 8. Growth Engine Design (k-factor)

```
k = i × c

i = avg Stream subscriber invitations sent per active curator per month
c = conversion rate of Stream visitors → registered Relay users
```

**Instrumentation (Week 1–2):**

- Events: `stream_published`, `stream_share_link_generated`, `stream_page_view`, `stream_subscribe_click`, `relay_install_complete`, `first_highlight_captured`
- Real-time dashboard, cohort analysis

**Targets:**

- MVP Week 4: k > 0.3
- Month 3: k > 0.5
- Year 1: k > 0.8

If k < 0.3 at Week 4: halt feature work, iterate Stream share flow.

---

## 9. Monetization (Subscription-First)

| Tier | Price | Features |
|---|---|---|
| Free | $0 | Unlimited highlights, basic AI, 1 Stream, 1 device, 50MB sync |
| Pro Monthly | $7.99/mo | Unlimited Streams, advanced AI, Themed Reviews, multi-device sync, 5GB, analytics, export |
| Pro Annual | $59.99/yr ($5/mo) | All Pro features, 37% discount |
| Creator | 15% platform fee | Monetizable Streams, audience analytics, verified badge |

**Paywall Triggers (non-intrusive):**

- Publishing 2nd Stream → "Unlimited Streams with Pro"
- Syncing 2nd device → "Multi-device sync with Pro"
- First Themed Review → "Free 7-day trial, then Pro"

---

## 10. Privacy & Regulatory Posture

**Five Guarantees (published, verifiable):**

1. All AI enrichment runs on-device.
2. Sync data is end-to-end encrypted; server stores only ciphertext.
3. Stream publications are opt-in; private library remains private.
4. Zero third-party data sharing; no data to sell.
5. Core client is open-source (Apache 2.0) for independent verification.

**Compliance Checklist (MVP gate):**

- Privacy policy (plain language)
- App Store privacy labels
- E2E encryption for sync
- Open-source repo public
- DPA template for Creator tier

---

## 11. Model Selection Bake-Off Protocol (Week 1)

**Candidates:** LFM2.5-1.2B-Thinking, Qwen-3.5-0.8B, Gemma 4 E2B

**Tests (50 samples each):**

- Tag generation: precision/recall vs. human labels
- Summarization: ROUGE-L vs. human summary
- Connection suggestion: binary human judgment
- Latency: time-to-first-token, total time on target devices (8GB and 4GB phones)
- Output format compliance: % well-formed outputs

**Success Thresholds (for default model):**

- Latency <1.5s (8GB), <3s (4GB)
- Tag precision >80%
- Compliance >95%
- RAM <500MB during inference

**Fallback:** Qwen-3.5-0.8B if no clear winner; LFM2.5 offered as optional upgrade.

---

## 12. Mobile Framework Spike (Week 1)

Build single-screen capture app in Flutter and KMP. Measure:

- APK size (no model): target <20MB
- Cold start to capture-ready: target <2s
- Cactus SDK integration time: target <4h
- Model load time: <3s

**Tiebreaker:** Flutter unless KMP shows >20% advantage on start time or app size.

**Actual Spike Results (Sprint 11):**

| Platform | Runtime | Model | Latency | Memory | Decision |
|----------|---------|-------|---------|--------|----------|
| iOS (Simulator) | MLX Swift | Qwen 3.5 0.8B 4-bit | 0.4–0.8 s | 250–400 MB | **GO** — confirm on physical device in Sprint 13 |
| Android (Emulator) | MediaPipe Tasks GenAI | Qwen 3.5 0.8B 4-bit | ~1.2 s | 350–500 MB | **Conditional GO** — model must ship via background CDN download, never bundled in APK (+1.2 GB) |
| Desktop | llama-cpp-2 (GGUF) | Qwen 3.5 0.8B Q4_K_M | <1.5 s | <500 MB | **GO** — production since Sprint 11 |

**Embedding Spike Results:**

| Platform | Runtime | Model | Dimension | Decision |
|----------|---------|-------|-----------|----------|
| Desktop | ONNX Runtime (`ort`) | all-MiniLM-L6-v2 | 384 | **GO** — production since Sprint 13 |
| Mobile | ONNX Runtime Mobile (target) | all-MiniLM-L6-v2 | 384 | **Planned** — deferred to Sprint 17 |

**Key Spike Learnings:**
1. Cactus SDK could not be integrated in <4 hours; lacks production Rust/Tauri support on desktop.
2. MLX Swift outperformed expectations on Apple Silicon simulator; physical-device validation pending.
3. MediaPipe Tasks GenAI compiles cleanly but requires model delivery mitigation to avoid APK size explosion.
4. ONNX Runtime (`ort`) proved more stable than Candle (zero releases, trunk-only) for desktop embeddings.

---

## 13. Success Metrics & Gates

| Gate | Metric | Target |
|---|---|---|
| MVP Week 2 | Capture-to-enrichment latency | <2s |
| MVP Week 2 | Tagging relevance | >85% useful by beta testers |
| MVP Week 4 | Stream visitor-to-install conversion | >5% |
| MVP Week 4 | k-factor | >0.3 |
| v1.0 Week 8 | DAU/MAU | >20% |
| v1.0 Week 8 | Stream publisher rate | >10% of active users |
| v1.0 Week 8 | Pro NRR | >100% |
| v1.0 Week 8 | NPS | >50 |

---

## 14. Risk Register (Updated)

| Risk | Severity | Mitigation |
|---|---|---|
| SLM output instability | High | Deterministic fallbacks; curated taxonomy; multi-layer parsing |
| LanceDB latency at small scale | Low | Fallback to brute-force SQLite search ready |
| Low Stream conversion | High | Halt feature work if k<0.3; iterate share experience |
| Sync conflicts (rare) | Low | LWW sufficient; conflict log user-visible |
| Mobile framework wrong choice | Medium | 3-day spike with hard decision criteria |
| Incumbent response | Medium | Open-source client; deep niche community first |
| Solo team burnout | High | Ruthless scope; ship smallest loop; Week 4 gate |

---

## 15. Open-Source Strategy

- **Repository:** github.com/gearbox/relay-client
- **License:** Apache 2.0
- **Scope:** Desktop client, AI pipeline, local search, sync protocol spec
- **Proprietary:** Server-side discovery, reputation, creator marketplace, subscription management
- Public from Day 1, Week 1.

---

## 16. Sprint History (Updated May 2026)

The original Week 1–4 outline (§16 v2.0) was aspirational. The actual build required 14 sprints to reach production-hardened desktop + mobile integration. Below is the audited history.

### Sprints 1–4 (Completed): Core Desktop Engine

| Sprint | Status | Focus | Key Deliverables |
|---|---|---|---|
| 1 | Done | Semantic Core Hardening | ONNX embedding pipeline; Mutex-based global state; 79 Rust tests; 10-run stability gate |
| 2 | Done | Frontend Hardening + K-Factor | App.tsx decomposition; 6 hooks; 10 UI components; 33 Vitest tests; all 5 analytics events wired |
| 3 | Done | Capture Fix, Model Status, History | EnrichmentChunk IPC fix; schema init hardening; `list_stored_highlights` + `delete_highlight`; `useModelStatus`; `useCapture` tests |
| 4 | Done | Desktop Polish + Core Loop | `useHistory` + `HistoryPanel`; `useToast`; `OnboardingModal`; empty states; model status badge; `export_local_data` + `clear_local_data`; local HTTP server (`127.0.0.1:0`); 91 Rust tests, 70 frontend tests |

### Sprints 5–8 (Completed): Sync, Auth, Mobile Spike

| Sprint | Status | Focus | Key Deliverables |
|---|---|---|---|
| 5 | Done | Sync Infrastructure | LWW sync engine; AES-256-GCM encryption; Argon2id key derivation; schema migration (`last_modified`, `sync_status`); sync server MVP; account creation/login; JWT auth flow |
| 6 | Done | Auth UI + Conflict Resolution | `SyncConflictPanel`; `SyncStatusBar`; `AuthForm` in Settings; offline queue; conflict log user-visible |
| 7 | Done | Mobile Spike + Clipboard Watcher | Flutter vs KMP measurement; **Flutter chosen**; clipboard background watcher on desktop; graceful flush on quit |
| 8 | Done | Polish + Launch Prep | Integration testing; keyboard shortcuts (`Cmd+Shift+C`, `Cmd+K`); telemetry toggle in Settings; mobile onboarding via SharedPreferences |

### Sprints 9–11 (Completed): Mobile End-to-End + AI Spikes

| Sprint | Status | Focus | Key Deliverables |
|---|---|---|---|
| 9 | Done | Mobile Capture + History | Flutter `CaptureScreen`; `enrichAndStore` via FRB; `HistoryScreen` with limit/offset pagination; 4-tab navigation |
| 10 | Done | Mobile Sync + Push + Deep Links | `sync_now` FRB wiring; `app_links` handler for `relay://stream/{id}`; `PushService` foreground handling; cross-device sync validation (3 integration tests); `RelayError` unified domain type |
| 11 | Done | Real AI Validation | Qwen 3.5 GGUF via `llama-cpp-2` (desktop); ONNX `all-MiniLM-L6-v2` embedding accepted; AI output parser hardening (20 well-formed + 20 malformed tests); iOS MLX Swift spike (**GO**); Android MediaPipe spike (**Conditional GO**); split CI workflows; `justfile` |

### Sprints 12–14 (Completed): Production Hardening + Security

| Sprint | Status | Focus | Key Deliverables |
|---|---|---|---|
| 12 | Done | Production AI | Desktop AI wiring (`enrich_clipboard` uses `LlamaService`, falls back to `FallbackService`); embedding production with SHA256 verification; AI quality validation (Qwen 3.0/3.0 vs Fallback 1.8/3.0); mobile CI clean (`flutter analyze`); security audit (1 critical escalation: `SEC-12-SYNC-METADATA`) |
| 13 | Done | Beta Readiness | Telemetry integration (Sentry desktop + mobile, PII scrubber, opt-out); desktop performance profiling (`StartupTimer`, 10 spans); beta polish (4-screen onboarding carousel, empty states, keyboard shortcuts); sync security PRD (`OpaqueBlob` v2 protocol design) |
| 14 | Done | Security, Resilience \u0026 Hardening | OpaqueBlob v2 crypto primitives (AES-256-GCM with AAD, 4 tests); AI OOM resilience (RAM check + `catch_unwind`); AI quality monitor (rolling parse-success window + degrade trigger); server test coverage 0→14; server rate limiting (Tower); auth state fix + background sync; **hotfix: 14 failing tests repaired** |
| 15 | Done | Sync v2 Integration | OpaqueBlob v2 wired into SyncEngine (dual-protocol); server v2 endpoints; dynamic Sentry toggle; deferred LanceDB/ONNX init; mobile telemetry FRB bridge; public beta docs; public repo snapshot |
| 16 | Done | Mobile AI Integration | iOS MLX integration; Android MediaPipe integration; Flutter model download manager with pause/resume/retry; typed Dart enrichment API with 5-layer defensive parser |
| 17 | Partial | The Great Parallelization | 32-agent attempt — ~85% failure rate. 7 deliverables completed: server public stream hosting, Tauri auto-updater, release CI, GGUF download scripts, server schema migration, legal compliance package, iOS Share Extension. Rest deferred to Sprint 18. |
| 18 | Done | Direct Recovery — Desktop Alpha | System tray + minimize-to-tray (telemetry, tests); sleep/wake lifecycle (suspend handler, telemetry); server blob lifecycle tests (14 → 24); search UI (hybrid ranking, filters, keyboard nav, SearchResultCard); connection suggestion UI; following feed + subscriptions tab; data export (Markdown + JSON ZIP); dark mode + accessibility (CSS variables, prefers-color-scheme); E2E Playwright tests + CI workflow |
| 19 | In Progress | Mobile Parity & Public Beta | SM-2 review sessions (Rust algorithm + React UI + 8 tests); server test coverage 24→38 (auth refresh, blob edge cases, notify endpoints); mobile search parity (filters, confidence badges); mobile dark mode (Dart ThemeData, system preference); mobile background sync (iOS BGAppRefreshTask, Android WorkManager); Android share intent; iOS+Android widgets; public beta launch checklist; PRD + ADR updates |

### Revised Gate Schedule

| Gate | Original PRD Target | Actual Achievement | Sprint |
|---|---|---|---|
| Capture-to-enrichment latency <2s | MVP Week 2 | Achieved (Sprint 1) | 1 |
| Tagging relevance >85% useful | MVP Week 2 | Achieved (Sprint 11); Qwen 3.0/3.0 vs Fallback 1.8/3.0 | 11 |
| Multi-device sync | Phase 2 | Achieved (Sprint 5–10); v1 protocol live, v2 fully implemented in Sprint 15 | 10, 15 |
| Stream visitor-to-install conversion | MVP Week 4 | Pending public beta | 15+ |
| k-factor >0.3 | MVP Week 4 | Pending public beta | 15+ |
| DAU/MAU >20% | v1.0 Week 8 | Post-launch | 15+ |
| Stream publisher rate >10% | v1.0 Week 8 | Post-launch | 15+ |

---

## 17. State of Implementation (May 2026 Extension)

This section documents deviations between the locked PRD v2.0 and the current implementation.

### 17.1 Embedding Model

| Item | PRD v2.0 | Actual Implementation | Rationale |
|---|---|---|---|
| Model | EmbeddingGemma 308M (768-dim) | **ONNX Runtime + all-MiniLM-L6-v2 (384-dim)** | Candle + Granite R2 never shipped; Candle has **zero releases** (trunk-only). ONNX Runtime is Microsoft-backed with stable releases, Metal/CUDA support, and a proven `all-MiniLM-L6-v2` model (Apache 2.0). Decision locked in Sprint 13; ADR-008 records full rationale. |
| Dimension | 768 (reducible to 128) | 384 | LanceDB schema uses 384-dim vectors. Migration path deferred to Phase 2. |
| Runtime | llama-cpp-2 (GGUF embedding) | ONNX Runtime (`ort` crate) | `ort` was already integrated for desktop. ONNX Runtime Mobile is the target for mobile alignment in Sprint 17. |
| Fallback | — | Zero-vector placeholder + keyword-only search | If `EmbeddingService::try_new` fails, the app stores a zero-vector and degrades search gracefully. No hard error. |

### 17.2 Sync

| Item | PRD v2.0 | Actual Implementation | Status |
|---|---|---|---|
| Sync Protocol | LWW (Week 3 deliverable) | **Implemented (v1)** — `EncryptedBlob` with `id`/`record_type`/`last_modified` outside ciphertext; `relay-core/src/sync/engine.rs` live | Sprint 5–10 |
| Sync Security | AES-256-GCM ciphertext only | **Partial** — metadata leak exists (`SEC-12-SYNC-METADATA`). `OpaqueBlob` v2 crypto primitives complete (Sprint 14); `SyncEngine` integration deferred to Sprint 15 | Sprint 15 |
| Multi-device sync | Phase 2 (Weeks 5–8) | **Implemented** — offline→online reconciliation with mismatched timestamps; 3 integration tests | Sprint 10 |
| Stream subscriptions | Cross-device subscription feed | **Local-only** — sync enables cross-device but subscription feed not yet wired server-side | Sprint 15 |

### 17.3 Mobile

| Item | PRD v2.0 | Actual Implementation | Status |
|---|---|---|---|
| Mobile framework spike | Week 1 | **Completed** — Flutter chosen over KMP (no advantage >20% on start time or size) | Sprint 7 |
| Mobile client | Phase 2 | **In Progress** — Flutter capture, history, stream curation, sync, push, deep-links live; AI runtime bridged via FRB; onboarding complete | Sprints 9–11 |
| Mobile AI runtime | Cactus SDK (preferred) | **iOS: MLX Swift (GO)**; **Android: MediaPipe Tasks GenAI (Conditional GO)** — both compile in CI, physical-device validation pending community beta | Sprint 11 |
| iOS model download | N/A | **HF Hub download on first launch (~430 MB)** — iOS 200 MB OTA cap makes bundling impossible; background download with resume implemented in Sprint 16 | Sprint 16 |
| Android model delivery | N/A | **Background CDN download via foreground service** — never bundled in APK (+1.2 GB); APK stays under 100 MB | Sprint 16 |
| Mobile embedding | Same as desktop | **Deferred to Sprint 17** — target ONNX Runtime Mobile or platform-native inference | Sprint 17 |

### 17.4 Background Process Lifecycle

| Item | PRD v2.0 | Actual Implementation | Status |
|---|---|---|---|
| Clipboard watcher | MVP | **Implemented** — desktop watcher with retry + `notify-rust`; background thread handles enrichment | Sprint 7–14 |
| Tray minimization | MVP | **Implemented** — tray icon with Open/Capture/Sync/Settings/Quit menu; close (X) hides to tray instead of exiting; telemetry events for tray clicks | Sprint 18 |
| System sleep handling | MVP | **Implemented** — `Resumed` event triggers wake + sync; `Suspended` handling ready with `set_sleeping(true)` pattern; `SystemSuspend`/`SystemWake` telemetry events | Sprint 18 |
| Low battery detection | MVP | **Not implemented** | Post-MVP |
| Graceful flush on quit | MVP | **Implemented** — stops watcher thread + flushes LanceDB before exit | Sprint 7 |

### 17.5 Telemetry

| Item | PRD v2.0 | Actual Implementation | Status |
|---|---|---|---|
| Crash reporting | Not specified | **Sentry desktop + mobile**, PII scrubber (`before_send` strips user/request/text), opt-out SQLite preference | Sprint 13 |
| Performance profiling | Not specified | **`StartupTimer`** with 10 spans; cold vs warm start documented; bottleneck recommendations in `docs/desktop-performance.md` | Sprint 13 |
| Analytics/events | 5 k-factor events wired | All 5 events instrumented; Sentry breadcrumbs for latency events | Sprints 2–13 |

---

## 18. Sprint 15 Recommendation — Sync v2 Integration & Public Beta

**Theme:** Close the last critical security gap (OpaqueBlob v2), wire remaining mobile bridges, and ship the public beta.

**Duration:** 3 weeks | **Agents:** 6 | **Risk:** Medium (v2 protocol migration is complex but PRD is complete)

### Rationale

After 14 sprints, Relay has:
- ✅ Desktop: capture, enrich, search, Stream publishing, sync v1, telemetry, onboarding
- ✅ Mobile: Flutter capture, history, Stream curation, sync, push notifications, deep-links
- ✅ Server: auth, blob storage, rate limiting, refresh tokens, 14 tests
- ✅ AI: Qwen 3.5 GGUF (desktop), ONNX embedding, OOM resilience, quality monitor
- ⚠️ **One critical gap remains:** sync metadata leak (`SEC-12-SYNC-METADATA`). The v2 crypto layer is complete but `SyncEngine` still uses v1 `EncryptedBlob`.

The product is feature-complete for a privacy-first knowledge pipeline. The only blocker to public beta is the sync metadata leak. Everything else is polish or performance.

### Sprint 15 Deliverables

| # | Deliverable | Agent | Prio | Est. Days | Risk |
|---|-------------|-------|------|-----------|------|
| 1 | **Wire OpaqueBlob v2 into `SyncEngine`** | Agent 1 | P0 | 5 | Medium — dual-protocol path; v1→v2 migration |
| 2 | **`SyncEngine` dual-protocol compatibility** | Agent 1 | P0 | 3 | Medium — must read v1 blobs, write v2, handle downgrade |
| 3 | **Server schema migration for v2** | Agent 2 | P0 | 3 | Low — add `protocol_version` to `encrypted_blobs`; migration script |
| 4 | **Mobile telemetry FRB bridge** | Agent 3 | P1 | 2 | Low — `get_telemetry_enabled` / `set_telemetry_enabled` stubs |
| 5 | **Desktop Sentry dynamic toggle** | Agent 4 | P1 | 2 | Low — restart prompt or re-init on toggle |
| 6 | **Defer LanceDB + ONNX init to background** | Agent 5 | P1 | 3 | Medium — `StartupTimer`验证了瓶颈; 需要仔细处理错误状态 |
| 7 | **Public beta checklist** | Agent 6 | P1 | 3 | Low — privacy policy, open-source repo public, DPA template, beta invite system |
| 8 | **`MISSING_OR_INCOMPLETE.md` audit + close P0s** | All | P0 | 1 | Low — sprint closeout procedure |

### Agent Assignments

| Agent | Expertise | Primary Files |
|---|---|---|
| **Agent 1** — Sync v2 Lead | Rust, crypto, protocol design | `relay-core/src/sync/engine.rs`, `relay-core/src/sync/opaque_blob.rs`, `relay-core/src/sync/integration_tests.rs` |
| **Agent 2** — Server Backend | Rust, SQLite, migrations | `relay-sync-server/src/db.rs`, `relay-sync-server/src/handlers.rs`, `relay-sync-server/tests/` |
| **Agent 3** — Mobile Bridge | Flutter, FRB, Kotlin/Swift glue | `relay_mobile/rust/src/api/relay_api.rs`, `relay_mobile/lib/services/telemetry_service.dart` |
| **Agent 4** — Desktop Telemetry | Rust, Sentry, Tauri lifecycle | `src-tauri/src/telemetry.rs`, `src/components/SettingsPanel.tsx` |
| **Agent 5** — Performance | Rust, async, background threads | `src-tauri/src/performance.rs`, `src-tauri/src/main.rs`, `src-tauri/src/db/` |
| **Agent 6** — Launch Ops | Docs, CI, legal infra, community | `docs/`, `.github/workflows/`, `PRD.md` |

### Test Requirements

- **Sync v2:** ≥10 unit tests for dual-protocol logic (v1 read, v2 write, downgrade shim). Integration test: offline capture → online sync → verify server receives `OpaqueBlob` with no plaintext metadata. **No v1-only path allowed after Sprint 15.**
- **Server migration:** Test both v1-only DB and v2-ready DB; migration must be zero-downtime for existing users.
- **Mobile telemetry:** FRB roundtrip test + Dart widget test for Settings toggle.
- **Performance:** `StartupTimer` must show <500ms improvement after background deferral (warm start).

### Sprint 15 → Public Beta Gate

After Sprint 15 merges to `main`:
1. `cargo test --workspace` → 100% pass (0 failures, ignored tests only)
2. `cargo clippy --workspace --all-targets -- -D warnings` → clean
3. `pnpm test` → 92/92 pass
4. Sync metadata audit: scan `relay-core/src/sync/` for any plaintext `id`/`record_type`/`last_modified` outside ciphertext → must be zero
5. **Tag `beta-v1.0`** on `main`
6. **Snapshot public repo** per AGENTS.md dual-repo procedure
7. **Beta invites:** 10 users, 1-week feedback window

### Sprint 16+ Tentative (Post-Beta)

Based on beta feedback:
- Themed Review sessions (AI-curated spaced repetition)
- Creator tier monetization (Stripe integration, analytics dashboard)
- Data export (Markdown, JSON)
- Reputation system (subscriber counts, trending Streams)
- Mobile AI physical-device validation (iOS MLX, Android MediaPipe)
- Browser extension (Chrome/Safari capture)

---
