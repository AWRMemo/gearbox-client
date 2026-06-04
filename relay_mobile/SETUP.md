# Sprint 8 Launch Checklist

## Prerequisites

```bash
# 1. Install Flutter SDK (3.38+)
#    Download: https://docs.flutter.dev/get-started/install/windows/mobile

# 2. Verify toolchain
flutter doctor
dart --version
flutter_rust_bridge_codegen --version   # should be 2.12.0

# 3. Install Android SDK + NDK
#    Android Studio → SDK Manager → SDK Tools → NDK (Side by side)
#    Set ANDROID_NDK_HOME env var to NDK root

# 4. Add Rust mobile targets
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
```

## Build Steps

```bash
cd relay_mobile

# 5. Install Flutter dependencies
flutter pub get

# 6. Run FRB codegen (generates lib/src/rust/ Dart bindings)
flutter_rust_bridge_codegen generate

# 7. Uncomment FRB imports in relay_service.dart
#    Remove MethodChannel fallback calls and uncomment the api.* lines

# 8. Verify Dart analysis
flutter analyze

# 9. Build debug APK
flutter build apk --debug

# 10. Launch on emulator or device
flutter run
```

## Verification Checklist

- [ ] `flutter analyze` passes with no errors
- [ ] `flutter build apk --debug` succeeds
- [ ] App launches without crash on emulator
- [ ] Capture: paste text → summary + tags appear
- [ ] Search: type query → FTS5 results render
- [ ] Settings: sync now → status updates
- [ ] Deep link: `adb shell am start -a android.intent.action.VIEW -d "relay://subscribe/test"` opens subscribe screen
- [ ] Desktop → mobile sync: highlights appear on both devices
