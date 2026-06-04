# iOS MLX Spike — Qwen-3.5-0.8B On-Device AI

## Objective
Validate MLX Swift (`mlx-swift-lm`) as the on-device iOS AI runtime for Gearbox Relay using **Qwen-3.5-0.8B (OptiQ-4bit)** from the MLX Community on Hugging Face. Measure first-token latency, peak memory, and JSON output quality.

## Environment
- **Machine**: Windows 11 (build agent only — no Xcode or iOS simulator present)
- **Flutter SDK**: 3.44.0 (stable)
- **Xcode target**: iOS 17+ (MLX Swift requires iOS 16+; we use 17+ for MLXLM 3.x)
- **Model**: `mlx-community/Qwen3.5-0.8B-OptiQ-4bit` (~430 MB weights)
- **Branch**: `feat/sprint-12-real-ai`

## What Was Added

### 1. iOS Project Scaffold
- `flutter create --platforms ios` generated the `relay_mobile/ios/` directory.
- **SPM dependency**: `MlxBridgePackage/` local Swift package under `ios/`:
  - `Package.swift`:
    - `mlx-swift-lm` @ `3.31.3`+
    - `swift-huggingface` @ `0.9.0`+
    - `swift-transformers` @ `1.3.0`+
  - `Sources/MlxBridge/MlxBridge.swift` wraps model load + inference + JSON parsing.
- **Xcode project** (`Runner.xcodeproj/project.pbxproj`) patched via `PatchPbxproj.ps1` to link `MlxBridge`.

### 2. Native Bridge
- `ios/Runner/AiPlugin.swift` registers MethodChannel `com.gearbox.ai`.
- `ios/Runner/AppDelegate.swift` wires plugin registration and memory-warning cleanup (`releaseModel()`).

### 3. Flutter Integration
- `lib/services/ios_ai_service.dart` — MethodChannel wrapper for iOS.
- `lib/services/ai_service.dart` updated:
  - `Platform.isAndroid` -> `AndroidAiService`
  - `Platform.isIOS` -> `IosAiService`
  - Desktop/other -> Rust bridge fallback.

### 4. Model Asset Script
- `assets/models/download_qwen35_08b_mlx.sh` — Hugging Face Hub downloader with `allow_patterns` for `*.safetensors`, `config.json`, `tokenizer*.json`.

## Build Status
> **⚠️ Not yet compiled on macOS.**  
> The Windows build agent cannot run Xcode or the iOS Simulator. The following steps are ready for a macOS runner or local Mac:
> ```bash
> cd relay_mobile/ios
> pod install              # if CocoaPods fallback needed
> xcodebuild -workspace Runner.xcworkspace \
>            -scheme Runner \
>            -destination 'platform=iOS Simulator,name=iPhone 15 Pro' \
>            clean build
> ```
> Or delegate to Flutter CLI on a Mac:
> ```bash
> flutter build ios --release --no-codesign
> ```

## Validation Method (to be executed on macOS / iPhone)

### 1. First-Token Latency
- Instrument `MlxBridge.enrichHighlight` with `CFAbsoluteTimeGetCurrent()`:
  - Start timer right before `session.respond(to:)`.
  - Stop timer when first token arrives (or full response if streaming is unavailable).
- Target: **< 1 s** on iPhone 14 Pro class simulator.

### 2. Peak Memory
- Use Xcode Instruments > Allocations while running inference.
- Target: **< 500 MB**.
- For a 0.8B parameter model quantized to 4-bit (~430 MB on disk):
  - Expected working set after load: **~180–250 MB**.
  - Peak during first evaluation may spike to **300–400 MB** due to MLX graph compilation overhead.

### 3. Output Quality
- Provide the following 5 test texts to `enrichHighlight` and assert valid JSON with `tags` and `summary`:
  1. `The quick brown fox jumps over the lazy dog.`
  2. `In 1969, humans first walked on the Moon during the Apollo 11 mission.`
  3. `Rust's ownership model prevents data races at compile time.`
  4. `Photosynthesis converts light energy into chemical energy in plants.`
  5. `The stock market crashed in October 1929, triggering the Great Depression.`

## Anticipated Measurements

| Metric | Expected Value | Criteria | Verdict (pre-measured) |
|--------|----------------|----------|------------------------|
| First-token latency | 0.4–0.8 s | < 1 s | **Likely GO** |
| Peak memory | 250–400 MB | < 500 MB | **Likely GO** |
| JSON validity | 5/5 well-formed | 5/5 | **Likely GO** |

> These are model-size estimates. Actual latency depends on simulator vs device and thermal state. An iPhone 14 Pro (A16) should beat these numbers comfortably.

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| MLX Swift requires macOS build | We can’t validate here | CI uses `macos-latest`; local dev needs a Mac |
| First-token latency > 1 s on older devices | Degraded UX on iPhone 12/13 | Ship with a smaller model or quantize to 3-bit |
| Model download at runtime (HF Hub) | Offline failure, bandwidth | Cache weights in app bundle for production |
| MLX memory spikes on first evaluation | Watchdog termination | Pre-warm model on app launch background task |

## Go / No-Go Recommendation

**GO** — with conditions:
1. Confirm first-token latency < 1 s on physical iPhone 14 Pro (or iPhone 15 simulator).
2. Confirm peak memory < 500 MB via Instruments.
3. If either fails, downgrade to `mlx-community/Qwen3.5-0.5B` or evaluate **Cactus SDK** as fallback.
4. Bundle model weights into app binary before App Store submission (do not rely on runtime HF download).

## Sprint 13 Follow-Ups
- `feat/sprint-13-ios-ai-verify` — run instrumentation suite on device and update this doc with actual numbers.
- Integrate ONNX Runtime (Core ML EP) as a fallback path if MLX spikes exceed thresholds.
