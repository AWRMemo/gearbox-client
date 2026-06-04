# Privacy Policy — Gearbox Relay

**Effective May 2026**

## Summary

Relay is a local-first AI knowledge pipeline. Your data stays on your device by default. No cloud AI APIs are ever called for core features. No advertising or profiling analytics SDKs are used. Opt-in crash and performance telemetry is disabled by default.

## What We Collect

### Core Features — Zero Collection

For capturing, enriching, searching, and browsing your highlights, **no personal data is sent to any server**. Your highlighted text, AI-generated tags and summaries, search vectors, and Stream content all live in a local SQLite database on your device.

### Sync Service — Encrypted Blobs Only

If you create an account and enable sync, your data is transmitted as **AES-256-GCM encrypted blobs** (OpaqueBlob v2 protocol). The server receives only a random UUIDv4 `blob_id` and the ciphertext. It cannot decrypt, inspect, or index your highlights. Encryption keys are derived from your sync password via Argon2id on-device and never leave your machine.

### Telemetry — Opt-In Only

Two lightweight telemetry systems exist, both **disabled by default**:

**Crash Reporting (Sentry):** When enabled, Sentry receives scrubbed crash reports. A `before_send` PII scrubber strips user identifiers, highlight text, summaries, stream titles, and request metadata before transmission. Only stack traces and timing spans are retained. Server IPs and device fingerprints are not captured.

**Performance Telemetry:** When enabled, the app records anonymous timing events: enrichment latency (milliseconds), sync latency, cold/warm startup time, model download progress. No content — only durations and boolean success/failure flags.

You can toggle both at any time in **Settings → Telemetry**. When opted out, no telemetry data leaves your device.

### k-Factor Analytics (Local Only)

Six anonymous event types are recorded in your local SQLite database for product improvement:

1. `relay_install_complete` — app first launched
2. `first_highlight_captured` — user captured their first highlight
3. `stream_published` — user published a Stream
4. `stream_share_link_generated` — user shared a Stream link
5. `stream_page_view` — someone visited a Stream page (anonymized device UUID)
6. `stream_subscribe_click` — visitor subscribed to a Stream

Each event contains: event name, timestamp, and device UUID (random, not linked to identity). **No IP addresses, no location data, no email addresses, no browser fingerprints.**

## Where Data Lives

All events are stored in a local SQLite database (`relay.db`) on your device. They never leave your device unless you explicitly opt in to aggregate sharing.

## Aggregate Sharing (Opt-In Only)

You may choose to share **anonymous aggregate counts** with the Relay sync server to help measure growth. This sends numbers like "Stream ABC received 47 views and 3 subscribes today." It never sends individual event rows or your highlights. You can toggle this off at any time in Settings.

## Your Rights

- **Export:** `Settings → Export Data` saves a ZIP of all your local data.
- **Delete:** `Settings → Clear All Data` permanently removes all local data (irreversible).
- **Revoke consent:** Disable telemetry and aggregate sharing in Settings.

## Third Parties

Relay uses no cloud AI providers and no ad networks.

The only external service is **Sentry** (Functional Software, Inc.), used solely for opt-in crash reporting. When enabled, Sentry data is PII-scrubbed before transmission. Sentry does not receive your highlight content. FCM (Firebase Cloud Messaging) is used as a transport for push notifications on mobile; it does not receive your highlight content.

## Open Source

The Relay client is open source (Apache 2.0). You can inspect the telemetry and data handling code at any time: `src-tauri/src/telemetry.rs`, `relay-core/src/telemetry.rs`.

## Contact

[Gearbox Relay repository](https://github.com/AWRMemo/gearbox-client)
