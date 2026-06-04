# Chrome Extension Spike — Relay Capture

## Feasibility Assessment: GO

### Key Findings

1. **Manifest V3 constraints:** Service workers can't access `document.execCommand('copy')` or the clipboard API directly from background. However:
   - `contextMenus` API works for capturing selected text from context menu clicks (the selected text is passed in `info.selectionText`)
   - `chrome.scripting.executeScript` can inject a function that reads `window.getSelection().toString()` for keyboard shortcut capture
   - `clipboardRead` permission works for capturing clipboard content via content scripts

2. **Local HTTP access:** Service workers CAN call `http://127.0.0.1:{port}` endpoints via `fetch()`. The existing `get_stream_public_url` loopback server can be repurposed.

3. **Permissions needed:** `contextMenus`, `activeTab`, `clipboardRead`, `scripting`, and `host_permissions` for `http://127.0.0.1:*/*`.

### Architecture

```
Chrome Extension
  ├── context menu "Capture to Relay" → selected text
  ├── Alt+Shift+R shortcut → selected text
  └── POST http://127.0.0.1:3000/v1/extension/capture
                                    ↓
                      relay-sync-server (loopback)
                                    ↓
                      db::store::store_highlight()
```

### Server Endpoint Needed
The relay sync server needs a new unauthenticated endpoint `POST /v1/extension/capture` that accepts `{ text, source_url }` and stores a highlight to the local SQLite DB. This endpoint must bind to `127.0.0.1` only.

### Recommendation: **GO** for Sprint 21
- Prototype works in Chrome as unpacked extension
- ~3 days for production implementation: server endpoint, extension polish, error handling, options page
- No cross-origin issues since all calls are loopback
