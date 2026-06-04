# PRD Gap Analysis — May 2026

**Status:** Honest audit of PRD v2.0 vs. actual implementation.  
**Purpose:** Prevent premature "beta" declarations; identify true remaining work.

---

## Executive Summary

We are **not close to PRD completeness**. Sprint 15 fixed the critical sync security gap, but the product remains a functional skeleton with significant missing surface area:

- **Core loop:** Capture → enrich → search works on desktop. Mobile capture works but enrichment is stubbed (dummy zero-vectors).
- **Stream publishing:** Local HTTP server exists, but no public URL, no subscription mechanism, no analytics.
- **Sync:** Desktop syncs. Mobile sync wires exist but lack integration tests.
- **AI:** Desktop uses real Qwen 3.5. Mobile uses no AI (stubs).
- **UI polish:** No designer pass; empty states exist but are minimal; no dark mode; no animations.
- **QA:** No end-to-end tests covering the full user journey (capture → publish → subscribe → sync back).
- **DevOps:** No release CI; no code signing; no update mechanism.

**Estimated to PRD-complete:** 6–8 additional sprints (~12–16 weeks with 4–6 agents).

---

## §7 Feature Breakdown — Item-by-Item Audit

### Phase 1: MVP (Weeks 1–4 per PRD)

| # | PRD Item | Status | Gap | Sprint |
|---|---|---|---|---|
| 1 | **Clipboard watcher (desktop)** | ✅ Works | | 7 |
| 2 | **Share extension (mobile)** | ❌ **Not started** | Needs iOS ShareExtension + Android IntentFilter wiring | — |
| 3 | **Manual highlight input** | ✅ Works | Desktop: `CapturePanel`. Mobile: `CaptureScreen` | 1–9 |
| 4 | **Source extraction (URL, title, author)** | ⚠️ **Partial** | Desktop extracts from clipboard text. Mobile does not extract source metadata | — |
| 5 | **On-device AI enrichment: tag, summary, connection suggestion** | ✅ Desktop works | Mobile: `FallbackService` wired but real SLM not loaded; connections disabled | 12 |
| 6 | **Local full-text + semantic search (LanceDB)** | ⚠️ **Partial** | Full-text works. Semantic search works on desktop but LanceDB blocks startup (deferred in Sprint 15). Mobile: no vector search | 15 |
| 7 | **Offline-only mode (no account required)** | ✅ Works | | 1 |
| 8 | **Stream creation & publishing** | ⚠️ **Partial** | Streams can be created and shared via `127.0.0.1`. No public URL, no SSL, no CDN | 4 |
| 9 | **Subscription mechanism (web → app install funnel)** | ❌ **Not started** | No web subscribe page; no install prompt; no funnel tracking | — |
| 10 | **Account system (email-based, minimal)** | ✅ Works | Desktop + mobile FRB auth | 5–6 |
| 11 | **Subscriber feed in-app** | ❌ **Not started** | No "Following" tab; no subscriber list; no feed UI | — |
| 12 | **k-factor measurement instrumentation** | ⚠️ **Partial** | 5 events wired but no dashboard; no conversion tracking | 2 |
| 13 | **Subscription paywall infrastructure (Pro triggers)** | ❌ **Not started** | No paywall UI; no Stripe integration; no tier enforcement | — |

### Phase 2: v1.0 (Weeks 5–8 per PRD)

| # | PRD Item | Status | Gap |
|---|---|---|---|
| 14 | **Themed Review sessions (AI-curated)** | ❌ **Not started** | No spaced-repetition UI; no thematic clustering |
| 15 | **Pro tier: unlimited Streams, advanced AI, multi-device sync, analytics** | ❌ **Not started** | No tier system; no Stripe; no analytics dashboard |
| 16 | **Creator tier: monetizable Streams, verified badge** | ❌ **Not started** | No Stripe Connect; no payout system; no badge UI |
| 17 | **Data export (Markdown, JSON)** | ❌ **Not started** | `export_local_data` exists (ZIP of SQLite) but no Markdown/JSON export |
| 18 | **Reputation system (subscriber counts, trending Streams)** | ❌ **Not started** | No server-side reputation; no trending algorithm |

---

## §6 Technical Architecture — Item-by-Item Audit

### 6.3 AI Pipeline

| PRD Step | Desktop | Mobile | Gap |
|---|---|---|---|
| Embedding (EmbeddingGemma 768-dim) | ✅ ONNX all-MiniLM-L6-v2 (384-dim) | ❌ Dummy zero-vectors | Wrong model; mobile missing entirely |
| Tag & Summary (SLM) | ✅ Qwen 3.5 GGUF | ❌ No SLM loaded | Mobile spike says GO but not integrated |
| Connection Suggestion | ⚠️ Code exists but not wired in UI | ❌ | Connection suggestion returned but never shown to user |
| Fallback | ✅ Multi-layer parser | ✅ Same | |

### 6.4 Sync Architecture

| PRD Requirement | Desktop | Mobile | Gap |
|---|---|---|---|
| Encrypted blobs (AES-256-GCM) | ✅ v2 OpaqueBlob active | ✅ v2 protocol supported via FRB | |
| Background sync | ✅ Spawns thread at login | ⚠️ `sync_now()` wired but not auto-scheduled | Mobile lacks background sync scheduler |
| Conflict log user-visible | ✅ `SyncConflictPanel` exists | ❌ No Flutter conflict UI | |

### 6.6 Background Process Lifecycle

| Scenario | PRD Requirement | Actual | Gap |
|---|---|---|---|
| Minimized to tray | Clipboard watcher continues; AI deferred to idle | ❌ Not implemented | No system tray icon; no idle detection |
| System sleep | Watcher pauses; scanned on wake | ❌ Not implemented | No sleep/wake handling |
| Low battery (<10%) | All AI paused; raw captures stored | ❌ Not implemented | No battery monitoring |
| Graceful flush on quit | Flush to SQLite + LanceDB | ❌ Not implemented | App exits immediately |
| Update | Preserve data; schema migration | ⚠️ Schema migrations exist but no update mechanism | No auto-updater (Tauri updater not configured) |

---

## §11 Model Selection Bake-Off

| Requirement | Status | Gap |
|---|---|---|
| Candidates: LFM2.5, Qwen-3.5, Gemma 4 E2B | ❌ **Not done** | We accepted Qwen-3.5 based on theoretical fit; no comparative bake-off run |
| 50-sample test per candidate | ❌ **Not done** | A/B test was 20 samples vs Fallback, not model bake-off |
| Latency <1.5s (8GB), <3s (4GB) | ⚠️ **Unverified** | Desktop measured; mobile latency unknown (no physical device tests) |
| Tag precision >80% | ✅ **Achieved** | Qwen 3.0/3.0 human relevance score |
| Compliance >95% | ✅ **Achieved** | 100% parse yield in test fixture |
| RAM <500MB | ⚠️ **Unverified on mobile** | Desktop OK; iOS/Android physical device validation pending |

---

## §12 Mobile Framework Spike

| Requirement | Status | Gap |
|---|---|---|
| APK size <20MB (no model) | ⚠️ **Unmeasured** | Never checked actual Flutter APK size |
| Cold start <2s | ❌ **Not measured** | No start-time telemetry on mobile |
| Cactus SDK integration <4h | N/A | Cactus not used; MLX Swift chosen for iOS |
| Model load <3s | ❌ **Unverified** | Physical device validation pending |

---

## §13 Success Metrics & Gates

| Gate | Metric | PRD Target | Actual | Status |
|---|---|---|---|---|
| MVP Week 2 | Capture-to-enrichment latency | <2s | <2s (desktop) | ✅ |
| MVP Week 2 | Tagging relevance | >85% useful | 3.0/3.0 vs Fallback 1.8/3.0 | ✅ (exceeds) |
| MVP Week 4 | Stream visitor→install conversion | >5% | **0%** — no public Streams, no install funnel | ❌ **Not started** |
| MVP Week 4 | k-factor | >0.3 | **0** — no viral mechanism live | ❌ **Not started** |
| v1.0 Week 8 | DAU/MAU | >20% | **N/A** — no users | ❌ **Not started** |
| v1.0 Week 8 | Stream publisher rate | >10% | **N/A** — no users | ❌ **Not started** |
| v1.0 Week 8 | Pro NRR | >100% | **N/A** — no billing | ❌ **Not started** |
| v1.0 Week 8 | NPS | >50 | **N/A** — no users | ❌ **Not started** |

---

## §10 Privacy & Regulatory — Compliance Checklist

| Requirement | Status | Gap |
|---|---|---|
| Privacy policy (plain language) | ✅ Written | Not reviewed by lawyer; not published on website |
| App Store privacy labels | ❌ **Not started** | No App Store submission yet |
| E2E encryption for sync | ✅ OpaqueBlob v2 | |
| Open-source repo public | ✅ Snapshot published | `gearbox-client` repo is public Apache 2.0 |
| DPA template for Creator tier | ✅ Written | Not reviewed by lawyer |
| **GDPR Art. 30 (Records of Processing)** | ❌ **Not started** | Required for any EU users |
| **CCPA compliance** | ❌ **Not started** | Required for California users |
| **SOC 2 Type II** | ❌ **Not started** | Required for institutional sales |

---

## Quality & Testing Gaps

| Area | PRD Standard | Actual | Gap |
|---|---|---|---|
| Parser unit tests | ≥10 well-formed + 10 malformed per model | 20 well-formed + 20 malformed | ✅ Meets |
| Sync integration tests | Offline→online with mismatched timestamps | 3 integration tests (v1 only previously); 7 v2 tests added | ✅ Meets |
| Server test coverage | 0 → 14 (PRD standard not specified) | 14 tests; missing blob lifecycle, rate-limit edge cases | ⚠️ Low coverage |
| Mobile UI tests | Not specified | **0** | ❌ Critical gap |
| End-to-end capture→publish→subscribe→sync journey | Not specified but implied | **0** | ❌ Critical gap |
| Performance benchmark regression suite | `relay-bench` exists | Single `quality_bench.rs`; no CI gate | ⚠️ Ad-hoc |
| Accessibility (a11y) audit | Not specified | No a11y testing | ❌ Unknown |
| Security audit (external) | Sprint 12 internal audit | Internal only; no external pentest | ⚠️ Risk |

---

## What "Beta" Actually Requires (Revised Definition)

The PRD §13 gates define MVP Week 4 readiness. We have not met them:

1. **Stream visitor→install conversion >5%**: Requires public Stream hosting (not `127.0.0.1`), SSL, install funnel, analytics. None exist.
2. **k-factor >0.3**: Requires real users, subscription mechanism, viral tracking. None exist.

**Revised Beta Criteria (minimum viable):**

| Criterion | Why It Matters | Status |
|---|---|---|
| Full capture→enrich→search→publish→view journey works end-to-end on desktop | Core loop must be intact | ✅ |
| Same journey works on mobile (iOS + Android) | Mobile is primary growth surface | ⚠️ Partial — capture works, enrichment stubbed |
| Stream can be viewed by non-owner without app install | Growth loop requires this | ❌ Local only |
| Sync works desktop ↔ mobile | Multi-device is Pro-tier selling point | ⚠️ Desktop↔server works; mobile↔server unverified |
| No critical security findings | Trust is the differentiator | ✅ OpaqueBlob v2 fixed |
| No P0 crashes in 7-day dogfooding | Stability gate | ❌ No dogfooding period completed |
| Designer/UI pass complete | First impression matters for retention | ❌ No designer engaged |
| Analytics dashboard operational | Must measure k-factor to iterate | ❌ No dashboard |

**Verdict:** We are **~40% to PRD MVP completeness** by strict reading, **~60%** if we count capture→enrich→search as the "MVP core." Beta is inappropriate.

---

## Recommended Next Steps

1. **Do NOT declare beta.** Remove `beta-v1.0` tag; replace with `sprint-15` only.
2. **Define "internal alpha"** — team-only, no public invites, no public Streams. Goal: dogfood the capture→publish loop for 2 weeks.
3. **Sprint 16 scope:** Mobile enrichment integration (iOS MLX + Android MediaPipe), Stream public URL, subscribe funnel skeleton.
4. **Designer engagement:** UI/UX pass before any public-facing milestone.
5. **QA regime:** E2E journey tests (Selenium/Playwright or Maestro for Flutter) before next tag.
6. **Legal review:** Privacy policy + DPA before any user data collection.

---

**Document owner:** Sprint 15 Agent 6 (Launch Ops)  
**Last updated:** 2026-05-24  
**Next review:** Sprint 16 closeout
