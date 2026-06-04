# Gearbox Relay

Local-first, AI-native personal knowledge pipeline. Capture text highlights, enrich them with on-device AI (tags, summary, semantic connections), and publish curated "Streams" for others to subscribe to. **All AI runs on-device. Zero cloud tokens. Guaranteed privacy.**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)

## Tech Stack

| Layer | Desktop | Mobile |
|---|---|---|
| Framework | Tauri 2.0 (Rust + React) | Flutter 3.48 |
| Text AI | llama-cpp-2 (Qwen 3.5 0.8B GGUF) | iOS: MLX Swift, Android: MediaPipe |
| Embeddings | ONNX Runtime (`ort`) + all-MiniLM-L6-v2 (384-dim) | ONNX Runtime Mobile (planned) |
| Vector DB | LanceDB | LanceDB (via relay-core) |
| Search | SQLite FTS5 + hybrid LanceDB | SQLite FTS5 |
| Sync | LWW object sync, OpaqueBlob v2 (AES-256-GCM) | Same protocol via relay-core |
| Database | SQLite (r2d2 connection pool) | SQLite via relay-core |

## Quick Start

```bash
# Install dependencies
pnpm install
cargo build

# Run desktop dev server (Tauri + Vite, port 1420)
pnpm tauri dev

# Run mobile (requires Flutter 3.48+)
cd relay_mobile && flutter run
```

GGUF model download: `scripts/download-qwen-gguf.ps1` (Windows) or `.sh` (Linux/macOS).

## Architecture

```
src/                 React frontend (capture UI, search, Stream viewer)
src-tauri/src/       Rust backend
  ai/                LlamaService (llama-cpp-2), EmbeddingService (ONNX), quality monitor
  commands/          12 Tauri command handlers (capture, search, streams, auth, sync, export)
  background/        Clipboard watcher, lifecycle (tray, sleep/wake, battery), deferred enrichment
  web/               Local HTTP stream server (127.0.0.1:0)
relay-core/          Shared Rust library
  ai/                AIService trait + FallbackService (deterministic keyword extraction)
  db/                SQLite schema, FTS search, LanceDB vector store, streams, analytics
  sync/              LWW sync engine, OpaqueBlob v2 encryption, conflict resolution
relay_mobile/        Flutter app (iOS + Android)
  lib/screens/       7 screens (capture, history, search, stream editor, settings, onboarding, subscribe)
  lib/services/      AI facade, enrichment parser, model download manager, deep links, push
  ios/               MLX Swift inference, Share Extension, ModelDownloadPlugin
  android/           MediaPipe inference, ModelDownloadService (foreground), AiPlugin
relay-bench/         AI quality benchmark harness
scripts/             GGUF model download scripts
```

## Privacy

All AI runs on-device. Sync blobs are end-to-end encrypted (AES-256-GCM, keys derived from user password via Argon2id). Opt-in crash reporting (Sentry) is disabled by default and PII-scrubbed. Read the full policy: [PRIVACY.md](PRIVACY.md).

## Status

**Sprint 19 — Public Beta Preparation.** The core loop (capture → enrich → search → publish → subscribe → sync) works end-to-end on desktop and mobile. See [MISSING_OR_INCOMPLETE.md](MISSING_OR_INCOMPLETE.md) for current gaps.

## Contributing

Apache 2.0. PRs welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for conventions and [SECURITY.md](SECURITY.md) for vulnerability reporting.
