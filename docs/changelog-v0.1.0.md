# Relay v0.1.0 — Internal Alpha / Beta

**Release date:** Sprint 19, May 2026  
**Build target:** Desktop (Tauri) + Mobile (Flutter)

---

## Features

- **Capture anything** — clipboard watcher (desktop), share intent (Android), future share extension (iOS)
- **AI enrichment** — on-device tagging + summarization with Qwen 3.5 0.8B (desktop), MLX Swift (iOS), MediaPipe (Android)
- **Hybrid search** — full-text (FTS5) + semantic (384-dim vector embeddings)
- **Stream publishing** — curate and publish themed collections as auto-updating web pages
- **Subscription feed** — follow published Streams from other users
- **Connection suggestions** — AI finds links between your highlights
- **Spaced repetition review** — SM-2 algorithm for periodic recall practice
- **Data export** — ZIP with JSON + Markdown
- **Dark mode** — respects system preference, toggle in Settings
- **System tray** — minimize to tray, background capture continues (desktop)
- **Sleep/wake lifecycle** — pause on suspend, sync on wake (desktop)
- **Background sync** — periodic + push-triggered (iOS/Android)

## Privacy Guarantees

1. All AI enrichment runs on-device — zero cloud tokens, full privacy
2. Sync data is end-to-end encrypted; server stores only ciphertext
3. Stream publications are opt-in; private library remains private
4. Zero third-party data sharing
5. Core client is open-source (Apache 2.0)

## Known Limitations

See `docs/known-issues.md` for the full list.

## Installation

1. Download the installer for your platform
2. Launch Relay — onboarding will guide you through first capture
3. No account required for offline use
