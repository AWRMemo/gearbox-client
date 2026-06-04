# Development Setup

## Windows Prerequisites

1. **Rust Toolchain** (MSVC target)
   - Install via [rustup.rs](https://rustup.rs/)
   - Ensure `stable-x86_64-pc-windows-msvc` is active.

2. **Protocol Buffers Compiler (`protoc`)**
   - Install via WinGet: `winget install Google.Protobuf`
   - The path must be exposed in `.cargo/config.toml` (see below).

3. **Node.js / pnpm**
   - `pnpm install` from the `src/` directory for frontend dependencies.

## Required `.cargo/config.toml`

Create `.cargo/config.toml` in the repository root with the following content. This is **mandatory** on Windows to avoid `crt-static` linking issues and to supply the `protoc` path.

```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=-crt-static"]

[env]
PROTOC = "C:\\Users\\<YOUR_USER>\\AppData\\Local\\Microsoft\\WinGet\\Packages\\Google.Protobuf_Microsoft.Winget.Source_8wekyb3d8bbwe\\bin\\protoc.exe"
```

> **Note:** Replace `<YOUR_USER>` with your actual Windows username. If you installed `protoc` via a different method, adjust the path accordingly.

## Build Verification

```bash
# From src-tauri/
cargo clippy --all-targets -- -D warnings  # must pass; CI gate
cargo fmt -- --check                        # must pass
cargo test --workspace --release            # all tests must pass
```

## Known Windows-Specific Issues

- **ONNX Runtime (`ort`)**: requires `libclang.dll` to be available in `PATH` (from LLVM/Clang installation).
- **SQLite + LanceDB**: no additional setup; bundled automatically.
- **Tauri dev server** (`cargo tauri dev`) expects `pnpm dev` on port `1420`. Windows Firewall may prompt on first launch.

## Quickstart

```bash
# 1. Install frontend deps
cd src/
pnpm install

# 2. Run desktop dev (from repo root)
cargo tauri dev
```
