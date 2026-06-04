# Known Issues — v1.0.1

## Security

### ~~SEC-12-SYNC-METADATA: Metadata leak outside ciphertext~~ **RESOLVED**
OpaqueBlob v2 protocol is now the default for new accounts. The v2 outer envelope (`blob_id` + opaque `payload`) contains no plaintext metadata. Server-side v2 endpoints (`POST/GET /v2/sync/blobs`) are deployed. Existing v1 accounts remain on v1 (backward-compatible); new accounts use v2 automatically.

**Status:** ✅ **RESOLVED (Sprint 20)**

## Monetization

### Stripe Sandbox Only
Stripe checkout, portal, and webhook endpoints are implemented but use sandbox test keys. Production activation requires creating live products/price IDs in the Stripe Dashboard and swapping `STRIPE_SECRET_KEY` / `STRIPE_WEBHOOK_SECRET` to live keys.

### ~~set_user_tier() removed~~ **RESOLVED**
The insecure client-side `set_user_tier()` command has been removed. Tier is now server-authoritative, set exclusively by Stripe webhooks.

**Status:** ✅ **RESOLVED (Sprint 21)**

## Performance

### LanceDB ANN Query Eventually Consistent
Vector similarity search via LanceDB may return stale results immediately after inserts. The system falls back to SQLite FTS5 + brute-force cosine similarity for read-after-write consistency.

### Desktop Cold Start
First launch after install is slow (~10s) due to LanceDB + ONNX model initialization. Background initialization (Sprint 15) improves warm starts.

## Mobile

### Background Sync CI-Only
iOS background sync (`BGAppRefreshTask`) and Android WorkManager sync are code-complete but have only been validated in CI simulators. Real device testing pending.

### Mobile AI Physical-Device Validation Pending
iOS MLX Swift and Android MediaPipe AI enrichment compile in CI but have not been latency-tested on physical devices.

### iOS Share Extension
Basic implementation exists but requires device provisioning profile testing.

### Mobile Widgets
iOS WidgetKit and Android AppWidget provider compile in CI but have not been visually verified.

## Desktop

### ~~Tray Icon~~ **RESOLVED**
The tray icon now loads from `icons/icon.ico` instead of a solid-blue RGBA buffer.

**Status:** ✅ **RESOLVED (Sprint 21)**

### ~~Low Battery Detection~~ **RESOLVED**
The battery monitoring function is implemented and wired to the watcher loop. Highlights are deferred for batch enrichment on wake/AC restore.

**Status:** ✅ **RESOLVED (Sprint 20)**

## Server

### Stripe Webhook Signature Verification
Webhook signature verification uses a manual implementation (not the Stripe SDK). Tested locally with Stripe CLI `stripe listen --forward-to`.

### No Stripe SDK
Both `async-stripe` (v1.0 RC) and `stripe-rust` (v0.12) were evaluated and rejected. Stripe REST API calls use raw `reqwest` with form-encoded bodies. Re-evaluate when `async-stripe` reaches v1.0 stable.

### Firebase/FCM Manual Credential Setup
Push notifications require manual provisioning of `google-services.json` and `service-account.json`. See `relay_mobile/firebase-setup.md`.

## Extension

### Chrome Extension Unpacked Only
The Chrome extension is functional but not published to the Chrome Web Store. Requires store listing, screenshots, and privacy disclosures for publication.
