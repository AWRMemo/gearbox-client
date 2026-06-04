# Sprint 16 — Mobile AI Integration: From Spike to Production

**Theme:** Replace mobile enrichment stubs (dummy zero-vectors, `FallbackService` only) with real on-device SLMs (iOS MLX Swift + Android MediaPipe). Unify the Flutter AI facade, harden output parsers, and build a model download manager.

**Duration:** 4 weeks  
**Agents:** 4 (parallel)  
**Risk:** High — both platforms lack physical-device validation; model files not in repo; Android APK size crisis.

---

## Executive Summary

Mobile capture works but enrichment is either stubbed (iOS: no model loaded) or unvalidated (Android: MediaPipe compiles but model asset missing). This sprint makes mobile enrichment production-grade by:

1. **iOS:** Completing MLX Swift bridge, adding runtime model download, and validating on-device inference
2. **Android:** Resolving MediaPipe model delivery (Play Feature Delivery or background download), validating inference
3. **Flutter:** Unifying error handling, adding desktop-parity multi-layer parser, exposing connection suggestions
4. **Infrastructure:** Background model download manager with progress UI, retry, SHA256 verification

---

## Current State (End of Sprint 15)

| Platform | Component | Status | Blocker |
|---|---|---|---|
| **iOS** | `MlxBridgePackage` SPM with `mlx-swift-lm` | ✅ Compiles in CI (macOS) | Model never downloaded; no runtime validation |
| **iOS** | `AiPlugin.swift` MethodChannel bridge | ✅ Exists | Never tested with Flutter |
| **Android** | `AiPlugin.kt` MediaPipe bridge | ✅ Compiles in CI (ubuntu) | Model asset `qwen-0_8b-cpu.task` (~1.2 GB) not in repo; no runtime validation |
| **Flutter** | `AiService` facade | ✅ Routes by platform | Returns raw JSON string; no structured parsing |
| **Flutter** | `capture_screen.dart` | ✅ Calls `_aiService.enrichHighlight` | No error recovery UI; no loading state |
| **All** | Model download system | ❌ Not started | iOS downloads from HF at runtime; Android needs Play Feature Delivery or background download |
| **All** | Enrichment quality validation | ❌ Not started | No mobile parser tests; no A/B vs desktop Qwen |

---

## Agent Assignments

### Agent 1 — iOS MLX Swift Integration Lead

**Goal:** Make iOS enrichment work end-to-end: model download → load → inference → JSON parse → Flutter display.

**Files:**
- `relay_mobile/ios/MlxBridgePackage/Package.swift`
- `relay_mobile/ios/MlxBridgePackage/Sources/MlxBridge/MlxBridge.swift`
- `relay_mobile/ios/Runner/AiPlugin.swift` (create if missing)
- `relay_mobile/ios/Runner/AppDelegate.swift`
- `relay_mobile/lib/services/ios_ai_service.dart`
- `relay_mobile/lib/services/ai_service.dart`

**Deliverables:**
1. Fix iOS MethodChannel wiring — ensure `com.gearbox.ai` channel is registered in `AppDelegate.swift`
2. Add model download on first launch:
   - Check `FileManager` for model existence
   - If missing, download `mlx-community/Qwen3.5-0.8B-OptiQ-4bit` from Hugging Face with progress callback
   - Store in `ApplicationSupport/relay/models/` (excluded from iCloud backup)
   - SHA256 verify after download (hash hardcoded in Swift)
3. Update `MlxBridge.enrichHighlight` to return structured JSON matching desktop `EnrichmentOutput` (`tags: Vec<String>, summary: String, connection_suggestion: Option<ConnectionSuggestion>`)
4. Handle MLX memory warnings: call `releaseModel()` in `AppDelegate.didReceiveMemoryWarning`
5. Defensive parser (3-layer, matching desktop parity):
   - Strip markdown fences
   - Extract first JSON object
   - `JSONDecoder` strict deser → `Codable` struct
   - Fallback: deterministic keyword extraction from input text
6. **Test evidence:** 5 iOS unit tests in `MlxBridgeTests/` (create test target):
   - `testEnrichHighlightReturnsValidJson`
   - `testEnrichHighlightHandlesMalformedOutput`
   - `testModelDownloadProgressUpdates`
   - `testMemoryWarningReleasesModel`
   - `testFallbackKeywordExtraction`

**Estimated:** 8 days  
**Risk:** High — requires Mac/Xcode to validate; CI only runs compile-check. Physical device testing in Sprint 17.

---

### Agent 2 — Android MediaPipe Integration Lead

**Goal:** Resolve the Android model delivery crisis and make enrichment work end-to-end.

**Files:**
- `relay_mobile/android/app/src/main/kotlin/com/gearbox/relay/AiPlugin.kt`
- `relay_mobile/android/app/build.gradle`
- `relay_mobile/android/app/src/main/AndroidManifest.xml`
- `relay_mobile/lib/services/android_ai_service.dart`
- `relay_mobile/lib/services/ai_service.dart`

**Deliverables:**
1. **Model delivery strategy (decision required):**
   - **Option A (recommended):** Background download service — app bundle excludes model; download on first launch from CDN (e.g., CloudFront with signed URLs)
   - **Option B:** Play Feature Delivery — model as on-demand module; adds Google Play dependency
   - **Option C:** Bundled asset with compression — APK still >500 MB; unacceptable
   - **Decision:** Option A. Implement `ModelDownloadService.kt` (foreground service with notification)
2. Update `AiPlugin.kt`:
   - Replace asset copy with filesystem path check
   - Remove hardcoded `qwen-0_8b-cpu.task` reference; accept model path from Flutter
   - Add `getModelStatus()` → `downloading` / `ready` / `error`
   - Add progress callback to Flutter via EventChannel
3. Update `ai_service.dart` to show download progress UI before enrichment
4. Defensive parser (3-layer, matching desktop parity):
   - Same as iOS: strip fences → extract JSON → strict deser → fallback keywords
   - Return `tags`, `summary`, `connection_suggestion` fields
5. **Test evidence:** 5 Kotlin unit tests in `android/app/src/test/kotlin/com/gearbox/relay/`:
   - `testEnrichHighlightReturnsValidJson`
   - `testEnrichHighlightHandlesMalformedOutput`
   - `testModelDownloadServiceProgressUpdates`
   - `testLlmInferenceLazyLoad`
   - `testFallbackKeywordExtraction`

**Estimated:** 10 days  
**Risk:** Very High — model delivery is unsolved in the industry at this scale; APK size gate is hard

---

### Agent 3 — Flutter AI Facade + Parser Unification

**Goal:** Unify iOS/Android/desktop enrichment behind a single typed API with desktop-parity defensive parsing.

**Files:**
- `relay_mobile/lib/services/ai_service.dart`
- `relay_mobile/lib/services/android_ai_service.dart`
- `relay_mobile/lib/services/ios_ai_service.dart`
- `relay_mobile/lib/services/relay_service.dart`
- `relay_mobile/lib/models/enrichment_output.dart` (new)
- `relay_mobile/lib/models/connection_suggestion.dart` (new)
- `relay_mobile/lib/screens/capture_screen.dart`

**Deliverables:**
1. Create typed Dart models:
   - `EnrichmentOutput` with `List<String> tags`, `String summary`, `ConnectionSuggestion? connectionSuggestion`
   - `ConnectionSuggestion` with `String sourceHighlightId`, `String bridgingSentence`
2. Update `AiService.enrichHighlight(String text)` to return `Future<EnrichmentOutput>` instead of raw JSON string
3. Add multi-layer defensive parser in Dart (parity with desktop `llama_service.rs`):
   - Layer 1: Strip markdown fences
   - Layer 2: Extract first `{...}` via brace-depth scan
   - Layer 3: `jsonDecode` into `EnrichmentOutput`
   - Layer 4: Loose field extraction with null safety
   - Layer 5: Deterministic keyword fallback (split text → filter length > 4 → distinct → take 5)
4. Add error recovery UI in `capture_screen.dart`:
   - Loading spinner during enrichment ("Analyzing with local AI...")
   - Retry button on parse failure
   - "AI model not ready — download in progress" state with progress bar
5. Wire `connection_suggestion` field through to the history card UI (even if null for now)
6. **Test evidence:** 10 Dart widget tests:
   - 5 well-formed JSON → valid `EnrichmentOutput`
   - 5 malformed JSON → fallback keyword extraction
   - 1 test for loading state → success animation
   - 1 test for download-in-progress → disabled capture button

**Estimated:** 6 days  
**Risk:** Medium — mostly Dart code, no hardware dependency

---

### Agent 4 — Model Download Manager (Cross-Platform)

**Goal:** Build a reusable background download service with progress, retry, and integrity verification.

**Files:**
- `relay_mobile/lib/services/model_download_service.dart` (new)
- `relay_mobile/lib/services/model_download_manager.dart` (new)
- `relay_mobile/lib/widgets/model_download_progress.dart` (new)
- `relay_mobile/ios/Runner/ModelDownloadPlugin.swift` (new)
- `relay_mobile/android/app/src/main/kotlin/com/gearbox/relay/ModelDownloadService.kt` (new)

**Deliverables:**
1. **Dart abstraction:** `ModelDownloadManager`
   - `downloadModel({required String platform, required String modelUrl, required String sha256})`
   - `Stream<DownloadProgress> get progressStream` (bytes downloaded / total)
   - `Future<bool> verifyIntegrity(String path, String expectedSha256)`
   - Exponential backoff retry (3 attempts: immediate, 5s, 30s)
   - Cancellation support
2. **iOS native plugin:** `ModelDownloadPlugin`
   - Uses `URLSession.downloadTask(withResumeData:)` for resumable downloads
   - Background download capability (BGTaskScheduler for completeness)
   - Stores in `ApplicationSupport/relay/models/`
3. **Android native service:** `ModelDownloadService`
   - Foreground service with notification (required for Android 10+ background downloads)
   - Uses `DownloadManager` for reliability
   - Notification shows progress bar
4. **Flutter UI widget:** `ModelDownloadProgress`
   - Circular progress indicator with percentage
   - "Resume" / "Cancel" / "Retry" buttons
   - Appears in `OnboardingScreen` (step 3: "Download AI model")
5. **Test evidence:**
   - Dart: 3 unit tests for retry logic, cancellation, integrity verification
   - iOS: 2 unit tests for resume data, completion handler
   - Android: 2 unit tests for notification creation, download completion

**Estimated:** 8 days  
**Risk:** Medium — well-understood domain; platform APIs are stable

---

## Sprint 16 → Sprint 17 Handoff

Sprint 16 delivers compilation, unit tests, and simulator validation. **Sprint 17** requires physical devices:

| Validation | Sprint 16 (Compile + Simulator) | Sprint 17 (Physical Device) |
|---|---|---|
| iOS latency | Simulator estimate (<1s assumed) | Real iPhone 14+: measure cold/warm start |
| iOS memory | Simulator estimate (<500MB assumed) | Xcode Instruments: Allocations |
| Android latency | Emulator estimate | Real Pixel 7+: measure cold/warm start |
| Android memory | Emulator estimate | Android Studio Profiler |
| APK size | CI build (without model) | Bundle with download service |
| JSON quality | 5 canned inputs per platform | 20 real-world highlights A/B vs desktop Qwen |

---

## Cross-Agent Dependencies

```
Agent 4 (Download Manager)
    ↓ provides progress stream
Agent 3 (Flutter Facade)
    ↓ calls platform enrich
Agent 1 (iOS MLX) ↔ Agent 2 (Android MediaPipe)
    ← both consume model path from Agent 4
    → both return JSON consumed by Agent 3
```

**Risk mitigation:** Agents 1 and 2 can develop against a mock `ModelDownloadManager` (local file path) while Agent 4 builds the real download system. Agent 3 writes parser unit tests against canned JSON strings, independent of platform.

---

## Sprint 16 Decision Log (Pre-Flight)

| Decision | Options | Verdict | Rationale |
|---|---|---|---|
| Android model delivery | A) Background download CDN | **A** | Option B (Play Feature Delivery) requires Google Play account + API; Option C makes APK >500 MB unacceptable. CDN download is industry standard (Perplexity, Pi). |
| iOS model delivery | A) HF Hub at runtime | **A** | iOS apps cannot exceed 200 MB OTA; MLX model is ~430 MB. Must download on first launch. Use `NSURLSession` background task. |
| Parser location | Dart vs. Native | **Dart** | Desktop parser is Rust. Mobile parsers in Dart allow hot-reload fixes without app store resubmission. Native parsers only for performance-critical paths. |
| Connection suggestion | Include in Sprint 16? | **Yes, but nullable** | Desktop returns `connection_suggestion`. Mobile parser should deserialize it but UI can ignore it. Prevents API mismatch. |
| Quality validation | Sprint 16 vs. 17 | **Sprint 17** | Sprint 16 focuses on compilation and unit tests. Physical device A/B testing requires hardware access, which agents don't have in CI. |

---

## Sprint 16 Success Criteria

1. `flutter build ios --release --no-codesign` completes without errors
2. `flutter build apk --release` completes without errors (without model asset)
3. Dart parser tests: 10/10 pass (5 well-formed + 5 malformed)
4. iOS unit tests: 5/5 pass (simulator)
5. Android unit tests: 5/5 pass (emulator)
6. Model download service unit tests: 7/7 pass (Dart + platform)
7. `flutter analyze --fatal-infos` clean
8. Capture screen shows loading state, retry, and download-in-progress UI
9. No `FallbackService` or dummy zero-vectors in the mobile enrichment path (real AI wired, even if model file is mock/test)

---

## Notes

- **Model agnosticism preserved:** Both iOS and Android use Qwen-3.5-0.8B. The `AiService` facade is model-agnostic — swapping to another GGUF/task model requires only changing the download URL and SHA256.
- **Privacy guarantee #1 maintained:** All inference on-device. No cloud API calls for enrichment.
- **Fallback preserved:** If model load fails or parse fails, deterministic keyword extraction still produces tags/summary.
- **CI limitation:** Physical device validation is impossible in GitHub Actions. Sprint 16 ships compile-clean + simulator/emulator tests. Sprint 17 is "device validation sprint."

---

**Document owner:** Sprint 16 planning session  
**Last updated:** 2026-05-24  
**Next review:** Sprint 16 mid-point (week 2)
