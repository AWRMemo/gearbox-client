# Privacy Policy — Gearbox Relay Beta

**Effective Date:** 2026-05-24  
**Version:** 1.0-beta  
**Applies to:** Gearbox Relay desktop (Tauri) and mobile (Flutter) applications, and the optional `relay-sync-server` sync service.

---

## 1. What Gearbox Relay Is

Gearbox Relay is a local-first, AI-native personal knowledge pipeline. Your highlights, notes, and Streams live primarily on your device. AI enrichment (tagging, summarisation) happens entirely on-device using locally-run small language models (SLMs). No cloud AI inference is used for core features.

---

## 2. What Data Is Collected

### 2.1 Core Features — Zero Collection

For the core app functionality (capturing text, AI enrichment, browsing history, creating Streams), **no personal data is sent to any server**:

- **Captured text** stays in your local SQLite database.
- **AI-generated tags and summaries** are produced by on-device models (Qwen-3.5-0.8B via llama-cpp-2, embeddings via ONNX Runtime).
- **Search vectors** are stored in an on-device LanceDB instance.
- **Stream content** is rendered from local data; optional local-only HTTP sharing binds to `127.0.0.1` (loopback) and is never exposed to the public internet.

### 2.2 Sync Service — Encrypted Blobs Only

If you choose to create an account and enable sync, your data is transmitted to the Gearbox sync server as **encrypted blobs** using the OpaqueBlob v2 protocol:

- The server receives only an opaque `blob_id` (random UUIDv4) and a base64-encoded AES-256-GCM ciphertext.
- The server **cannot** decrypt, inspect, index, or read the contents of your highlights, tags, summaries, or Stream metadata.
- All encryption keys are derived from your sync password on your device using Argon2id. Gearbox does not hold your password or your keys.
- The server’s only access to metadata is an internal ingestion timestamp (`received_at`) and the acknowledgement status of blobs. It does not know the semantic `id`, `record_type`, or `last_modified` of any record.

### 2.3 Telemetry — Performance & Crash Reporting

We collect **opt-in** telemetry to improve stability and performance:

| What | When | PII Status |
|------|------|------------|
| Crash reports (stack traces) | When the app panics or crashes | Scrubbed — user text, highlight content, summaries, and stream titles are stripped before transmission |
| Startup timing spans | Every cold/warm start | Aggregate only; no user content |
| Enrichment latency | After each AI enrichment | Duration in milliseconds only; no prompt text |
| Parse success rate | After AI output parsing | Boolean + model version; no content |

Telemetry is **disabled by default** (opt-in). You can toggle it at any time in **Settings → Telemetry**. When opted out, no telemetry data leaves your device.

---

## 3. What Data Looks Like to the Server

If an attacker or server operator inspects sync traffic, they see only:

```json
{
  "blob_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "payload": "U2FsdXJlIGlzIG9wYXF1ZS4="
}
```

The `payload` is AES-256-GCM ciphertext. Without your device-derived key, it is computationally infeasible to recover the plaintext.

---

## 4. Third-Party Sharing

We **do not** sell, rent, or share your personal data with third parties for advertising or marketing.

The only external service invoked is **Sentry** (used solely for crash reporting when telemetry is enabled), configured with a `before_send` PII scrubber that strips:

- User identifiers and request metadata
- Highlight text, summaries, and stream titles
- Any free-form text fields that could contain user content

No other third-party services (OpenAI, Anthropic, Google Cloud, etc.) receive your data.

---

## 5. User Rights

At any time, you may:

1. **Export your data** via **Settings → Export Data**. This creates a ZIP file containing your local SQLite database and settings.
2. **Delete individual highlights** by selecting a highlight and choosing Delete.
3. **Clear all local data** via **Settings → Clear Data**. This irreversibly removes all highlights, streams, and settings from your device.
4. **Delete your sync account** by contacting us (see §7). Deleting an account purges all encrypted blobs from the sync server.

Because core data is local-first, **you retain full possession of your data on your device** even if you stop using the sync service.

---

## 6. Data Retention

- **Local data (highlights, tags, summaries):** Retained until you delete them or uninstall the app. Because Relay is local-first, your device is the primary custodian.
- **Sync blobs:** Retained on the server until you delete your account or acknowledge them. Acknowledged blobs are garbage-collected after 90 days.
- **Telemetry (if opted in):** Retained in Sentry for 90 days, then purged automatically.
- **Server access logs:** Retained for 30 days for rate-limiting and abuse detection, then purged.

---

## 7. Contact

If you have questions about this Privacy Policy or wish to exercise your rights:

- **Email:** privacy@gearbox.dev
- **GitHub Issues:** https://github.com/AWRMemo/gearbox-client/issues (public client repo)
- **Mailing Address:** Gearbox Labs, Inc. — Privacy Office

We will respond to all inquiries within 72 hours.

---

## 8. Changes

We may update this policy as the beta progresses. Material changes will be announced in-app and via GitHub releases. Continued use after changes constitutes acceptance.

---

*This privacy policy is provided as part of the Gearbox Relay public beta (Sprint 15).*
