# ADR-013: Browser Extension Spike — GO Decision

## Status
Accepted

## Context
Sprint 20 deliverable 7 required a Chrome extension spike to assess feasibility of capturing text to Relay from a browser.

## Decision
**GO** for Sprint 21 full implementation. The prototype confirmed:
- Manifest V3 service workers can call `http://127.0.0.1:3000` endpoints
- Context menu captures `selectionText` natively (no clipboard needed)
- Keyboard shortcut `Alt+Shift+R` works via `chrome.scripting.executeScript`
- Permissions: `contextMenus`, `activeTab`, `scripting`, `clipboardRead` are reasonable

## Consequences
- Sprint 21 will add `POST /v1/extension/capture` endpoint to the relay sync server
- Extension will be distributed as unpacked (later via Chrome Web Store)
- Firefox variant deferred to Sprint 22 (Manifest V3 support is still maturing)
