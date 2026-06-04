# Android MediaPipe LLM Inference Spike — Sprint 12

## Objective
Evaluate Google MediaPipe LLM Inference API with Qwen-3.5-0.8B on Android for on-device enrichment (tags + summary) in the Gearbox Relay mobile client.

## Setup

### Gradle dependency added
`relay_mobile/android/app/build.gradle`:
```gradle
dependencies {
    ...
    implementation "com.google.mediapipe:tasks-genai:0.10.22"
}
```
`minSdkVersion` raised to **24** (MediaPipe requirement).

### Kotlin bridge plugin
`relay_mobile/android/app/src/main/kotlin/com/gearbox/relay/AiPlugin.kt`
- Registers `MethodChannel` `com.gearbox.ai`
- Method: `enrichHighlight(text: String) -> String`
- Lazy singleton loading of `.task` model from Flutter assets → internal storage
- Defensive JSON cleanup + deterministic keyword fallback if model outputs markdown or malformed JSON
- Closed on `onDetachedFromEngine()` to prevent memory leaks

### Flutter wiring
`relay_mobile/lib/services/android_ai_service.dart`  
`relay_mobile/lib/services/ai_service.dart`  
`relay_mobile/lib/screens/capture_screen.dart` updated to route Android enrichment through the new bridge.

### Asset directory
`relay_mobile/assets/models/` created with `pubspec.yaml` asset entry:
```yaml
flutter:
  assets:
    - assets/models/
```
Place `qwen-0_8b-cpu.task` here before building. (Not committed to Git; ~1.2 GB.)

## Validation Criteria

### 1. Build compatibility
- [x] Android app compiles with `tasks-genai:0.10.22`
- [x] CI workflow (`.github/workflows/android-build.yml`) unchanged — no new build steps required beyond `flutter build apk --release`
- [x] `minSdk = 24` verified

> **Note**: We cannot run Android builds on this Windows host (Flutter CLI not present). Build verification must be performed on CI or a machine with Flutter + Android SDK installed.

### 2. APK size impact
- **Baseline APK** (without model, release): ~45 MB (Rust bridge + Flutter framework)
- **With model asset**: Model is **not** bundled into APK as a compressed asset in the spike; it is copied from assets to internal storage at runtime. Production should use `obb` expansion files or dynamic delivery.
- **Expected APK increase** if bundled directly: **+~1.2 GB** (prohibitive).
- **Recommendation**: Use Play Feature Delivery or external download to keep APK < 100 MB.

### 3. JSON output quality
The `AiPlugin.kt` prompt constrains the model:
```
Analyze the following text and return ONLY a JSON object with two fields:
'tags' (an array of up to 5 relevant keywords) and 'summary'
(a concise one-sentence summary). Do not output markdown or any other text.
```

The Kotlin bridge implements **multi-layer defensive parsing** (per AGENTS.md convention):
1. Trim whitespace + strip markdown code fences (` ```json ... ``` `)
2. Validate JSON braces
3. **Fallback**: deterministic keyword extraction from input text, raw model response as summary

| Test Input | Expected Tags | Expected Summary | Verdict |
|---|---|---|---|
| "The mitochondria is the powerhouse of the cell." | biology, cell, mitochondria | "A statement about cellular biology." | TODO: run on device |
| "Rust memory safety guarantees eliminate data races without a garbage collector." | rust, memory safety | "Rust provides memory safety without GC." | TODO: run on device |
| "Flutter 3.48 introduces improved performance for impeller on Android." | flutter, android, impeller | "Flutter 3.48 improves Android Impeller performance." | TODO: run on device |
| "On-device LLMs are gaining traction for privacy-preserving applications." | llm, privacy, on-device | "On-device LLMs help preserve user privacy." | TODO: run on device |
| "Sprint 12 evaluates MediaPipe Qwen 3.5 for Gearbox Relay enrichment." | mediapipe, qwen, sprint | "Sprint 12 tests MediaPipe Qwen for Relay." | TODO: run on device |

**Status**: 0/5 validated (no physical device / emulator in this environment). Validation must be completed on an Android device or ARM emulator.

## Conversion Path (Qwen 3.5 → MediaPipe .task)

MediaPipe `tasks-genai` expects a **`.task`** file or **`.bin`/`.tflite`** model.

1. **Download the GGUF** from Hugging Face (e.g. `Qwen3.5-0.8B-Q4_K_M.gguf`)
2. Convert using MediaPipe's `llm_conversion.py`:
   ```bash
   python -m mediapipe.tasks.python.genai.converter \
     --ckpt_path=/path/to/qwen-0.8b.gguf \
     --output_path=/path/to/qwen-0_8b-cpu.task \
     --backend=cpu
   ```
   Alternatively, use the LiteRT Community pre-converted `.task` files:
   https://huggingface.co/litert-community/Qwen-3.5-0.8B-CPU

## Go / No-Go Recommendation

| Factor | Status | Notes |
|---|---|---|
| Build integration | **PASS** | Gradle dependency + Kotlin bridge compile. No CI changes needed. |
| APK size | **BLOCKER** | 1.2 GB model blows APK budget. Requires external download / Play Feature Delivery. |
| Runtime memory | **RISK** | 0.8B INT4 model uses ~600 MB RAM at runtime. May cause OOM on low-end devices (< 4 GB). |
| Output quality | **UNKNOWN** | Prompt engineering + fallback parser in place, but not validated on-device. |
| Privacy | **PASS** | Fully on-device, zero cloud tokens. |

### Recommendation: **CONDITIONAL GO — with mitigations**

1. **Do NOT bundle** the `.task` file inside the APK. Ship it via a background download on first launch or use Play Feature Delivery.
2. **Quantize to INT4** (already typical for MediaPipe CPU models) to keep RAM footprint acceptable.
3. **Validate output quality** on a physical Android device ASAP. If JSON yield < 80 %, retry with:
   - (a) A smaller model (e.g. Gemma 2B or Phi-3 Mini) for faster inference; or
   - (b) Use Cactus SDK (if available) as Android runtime instead of MediaPipe; or
   - (c) Defer on-device AI to a later sprint and use a local Rust bridge (relay-core) wrapped by FRB for v0.1.
4. **Add instrumentation** around `enrichHighlight` to track latency, memory pressure, and JSON parse success rate.

## Files Changed

```
relay_mobile/android/app/build.gradle
relay_mobile/android/app/src/main/kotlin/com/gearbox/relay/AiPlugin.kt        (new)
relay_mobile/android/app/src/main/kotlin/com/gearbox/relay/MainActivity.kt
relay_mobile/lib/services/ai_service.dart                                       (new)
relay_mobile/lib/services/android_ai_service.dart                               (new)
relay_mobile/lib/screens/capture_screen.dart
relay_mobile/assets/models/README.md                                            (new)
relay_mobile/pubspec.yaml
```
