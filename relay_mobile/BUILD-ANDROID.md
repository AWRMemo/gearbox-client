# Android APK Build — Linux/WSL2 Only

The `ring` crate (0.17.14, used by relay-core's AES-256-GCM encryption) injects MSVC compiler flags (`/MD`) on Windows hosts, even when cross-compiling for Android. The Android NDK's clang does not recognize these flags, causing the build to fail.

## Linux/WSL2 Build Commands

```bash
# 1. Install Rust Android targets
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

# 2. Install cargo-ndk
cargo install cargo-ndk

# 3. Set NDK path
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/27.0.12077973"

# 4. Build Rust .so for arm64
cd relay_mobile/rust
cargo ndk -t arm64-v8a -o ../android/app/src/main/jniLibs build --release

# 5. Build APK
cd ..
flutter build apk --debug
```

## CI Alternative

Use GitHub Actions with `ubuntu-latest` runner:
```yaml
- uses: actions-rs/toolchain@v1
  with: { toolchain: stable, target: aarch64-linux-android }
- run: cargo ndk -t arm64-v8a -o android/app/src/main/jniLibs build --release
- run: flutter build apk --debug
```

## Verification

```bash
ls relay_mobile/android/app/src/main/jniLibs/arm64-v8a/librelay_mobile_bridge.so
```
