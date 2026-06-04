# Getting Started with Gearbox Relay

Relay is a local-first, AI-native personal knowledge pipeline. Your highlights are enriched with on-device AI, stored privately on your machine, and publishable as curated Streams.

## Installation

1. Download the latest release from [GitHub Releases](https://github.com/AWRMemo/gearbox-client/releases)
2. Run the installer for your platform
3. Launch Gearbox Relay from your applications menu

## First Launch

On first launch, Relay downloads a small language model (~500 MB) for on-device AI enrichment. This is a one-time download. While the model downloads:

- You can **capture highlights immediately** — the app uses a deterministic keyword-based fallback for tagging and summarization
- A progress indicator shows download status
- When the download completes, the AI model hot-swaps automatically — subsequent captures use full on-device intelligence

The download runs in the background. You do not need to wait for it to finish before using the app.

## Capturing Highlights

### Automatic Clipboard Capture
Relay watches your clipboard. Any text you copy (Ctrl+C) is automatically captured. The app enriches each capture with:
- **Summary** — a one-sentence AI-generated summary of the highlight
- **Tags** — 3-5 keyword tags for searchability
- **Connections** — suggested links to related highlights you've already captured

You can disable auto-capture in Settings.

### Manual Capture
Open the **Capture** tab and paste or type text directly into the input field. Click "Capture" to enrich and save.

### Chrome Extension
Install the Relay Capture browser extension to capture highlighted text from any webpage:
1. Open Chrome and go to `chrome://extensions`
2. Enable "Developer mode"
3. Click "Load unpacked" and select the `extensions/chrome/` directory from the Relay installation
4. Right-click any selected text → "Capture to Relay"
5. Or press `Alt+Shift+R` to capture the current selection

## Search

The **Search** tab provides hybrid search across all your highlights:
- Full-text search via SQLite FTS5
- Semantic search via vector similarity (LanceDB)
- Filter by date range, tags, or source domain
- Keyboard shortcut: `Ctrl+K` (or `Cmd+K` on macOS)

## Review Sessions

The **Review** tab provides spaced-repetition review of your highlights using the SM-2 algorithm. Each day, highlights due for review appear in a card-based session:

1. Click "Start Review" to begin
2. Read the summary. Tap the card to reveal the full text.
3. Grade your recall: **Again**, **Hard**, **Good**, or **Easy**
4. The algorithm schedules the next review based on your grade

Regular review sessions help you retain what you've captured.

## Streams

A **Stream** is a curated collection of highlights on a specific topic. You can publish a Stream to share your curated knowledge:

1. Go to the **Streams** tab
2. Click "Create Stream" and give it a title and description
3. Add highlights from your library to the Stream
4. Click "Export" to save the Stream as a standalone HTML file
5. Share the HTML file with anyone — it opens in any browser and includes a "Subscribe in Relay" link

## System Tray

Relay minimizes to the system tray when you close the window. The tray icon stays in your taskbar/menu bar:

- **Left-click** the tray icon to show Relay
- **Right-click** for menu: Open, Capture Now, Sync Now, Settings, Quit

The clipboard watcher continues running while Relay is minimized to the tray.

## Sync (Optional)

If you create an account and enable sync, your highlights are encrypted with AES-256-GCM before leaving your device. The sync server stores only ciphertext — it cannot read your data.

To enable sync:
1. Go to **Settings** → **Account**
2. Create an account with your email and a sync password
3. Your highlights will sync across devices automatically

## Privacy

- All AI enrichment runs **on-device** — no cloud inference, zero token costs
- Captured text never leaves your machine unless you enable optional sync
- Sync data is end-to-end encrypted — the server cannot decrypt your highlights
- Opt-in crash reporting (Sentry) is **disabled by default** and strips all personal data before transmission
- Read the full privacy policy in [PRIVACY.md](https://github.com/AWRMemo/gearbox-client/blob/main/PRIVACY.md)

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+Shift+C` / `Cmd+Shift+C` | Capture current clipboard content |
| `Ctrl+K` / `Cmd+K` | Focus search input |
| `Alt+Shift+R` | Capture selection (Chrome extension) |

## Need Help?

- [GitHub Issues](https://github.com/AWRMemo/gearbox-client/issues) — report bugs or request features
- [Security disclosures](mailto:security@gearbox.dev) — report vulnerabilities privately
