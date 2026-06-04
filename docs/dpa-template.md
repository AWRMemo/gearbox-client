# Data Processing Addendum (DPA) — Gearbox Relay Creator Tier

**Version:** 1.0-beta  
**Effective Date:** 2026-05-24  
**Applies to:** Creator tier users who publish public Streams and opt into the Gearbox sync service.

---

## 1. Roles and Responsibilities

| Role | Party | Scope | Responsibility |
|------|-------|-------|----------------|
| **Data Controller** | You (the user) | All core data | You decide what content to capture, enrich, and publish as Streams. You control whether sync is enabled, who subscribes to your public Streams, and when data is deleted locally. |
| **Data Controller** | Gearbox Labs, Inc. | Account metadata | Gearbox controls the purposes and means of processing hashed email addresses, device tokens, and session metadata necessary to operate the sync and authentication service. |
| **Data Processor** | Gearbox Labs, Inc. | Sync infrastructure | We process encrypted sync blobs and account metadata solely to provide the sync and Stream-hosting service. We do not determine the purposes or means of processing the encrypted contents. |

**Clarification:**
- For **highlights, tags, summaries, and Stream content**, you remain the sole Data Controller. Gearbox cannot access the plaintext of this data because it is encrypted client-side with keys derived from your password.
- For **sync server infrastructure** (transport, storage, and authentication of encrypted blobs), Gearbox acts as a Data Processor under your control, as well as an independent Data Controller for the minimal account metadata required to operate the service.

When sync is **disabled**, Gearbox acts as neither controller nor processor for your local data, because no data is transmitted to our infrastructure.

---

## 2. Description of Processing

| Activity | Data Subjects | Personal Data Involved | Purpose |
|----------|-------------|------------------------|---------|
| Sync transport | Creator and subscribers | Encrypted blobs containing highlights and stream metadata (opaque to Gearbox) | Multi-device data consistency |
| Stream page hosting | Subscribers | Obfuscated deep-link identifiers (`relay://stream/{id}`) | Public Stream access via loopback URLs |
| Crash/performance telemetry | Creator | Scrubbed stack traces and millisecond timing metrics (no content) | Service reliability and improvement |
| Account management | Creator | Email (hashed), device tokens (opaque), protocol version | Authentication and push notifications |

**Important:** Gearbox does **not** have access to the plaintext of highlights, AI-generated summaries, tags, or Stream content. All such data is encrypted client-side before transmission.

---

## 3. Sub-Processors

| Sub-Processor | Function | Location | Data Access |
|---------------|----------|----------|-------------|
| **Sentry** (Functional Software, Inc.) | Crash reporting and error tracking | US (EU-hosted option available upon request) | Scrubbed stack traces and performance timing only; no user content |

No other sub-processors are used for core services. All AI inference (tagging, summarisation, embeddings) runs on-device and does not invoke a sub-processor.

---

## 4. Security Measures

Gearbox implements the following technical and organisational measures:

### 4.1 Encryption

- **Data at rest (local):** SQLite database resides in the user’s app data directory, protected by OS-level file permissions.
- **Data in transit (sync):** AES-256-GCM authenticated encryption with protocol-version AAD (`relay-sync-v2`).
- **Key derivation:** Argon2id from the user’s sync password; Gearbox does not store passwords or derived keys.

### 4.2 Network Security

- **Loopback-only sharing:** The optional local HTTP server for Stream previews binds exclusively to `127.0.0.1:0` (OS-assigned port). No public interface exposure.
- **TLS:** All sync traffic is transmitted over TLS 1.3.
- **Rate limiting:** The sync server enforces per-user upload rate limits to prevent abuse.

### 4.3 Access Control

- Server infrastructure access is restricted to authorised infrastructure engineers.
- Audit logging records all infrastructure access events.
- Production systems use hardware security modules (HSMs) or encrypted secret stores for service credentials.

### 4.4 Code Integrity

- Client source code (desktop and mobile) is published under Apache 2.0 on GitHub.
- CI pipelines (`cargo clippy`, `cargo test`, `flutter analyze`) must pass before any release.
- Automated dependency audits (`cargo audit`, `pnpm audit`) are run weekly.

---

## 5. Data Retention

| Data Category | Retention Period | Controller | Deletion Mechanism |
|-------------|------------------|------------|-------------------|
| Highlights, tags, summaries (local) | Until user deletion or app uninstall | You (user) | In-app delete or OS uninstall |
| Encrypted sync blobs (unacknowledged) | Until user deletes account | Gearbox (processor for transport) | Immediate purge on account deletion |
| Encrypted sync blobs (acknowledged) | 90 days after client acknowledgement | Gearbox (processor for storage) | Automatic GC for acknowledged blobs; immediate purge on account deletion |
| Account metadata (hashed email, device tokens) | Until account deletion | Gearbox (controller) | Hard delete from `users` and `devices` tables |
| Scrubbed telemetry (Sentry) | 90 days | Gearbox (controller) | Automatic Sentry retention policy |
| Server access / sync logs | 90 days | Gearbox (controller) | Automatic rotation and purge |

---

## 6. Breach Notification

In the event of a confirmed breach of personal data (including account metadata or unencrypted payloads), Gearbox will:

1. **Notify you within 72 hours** of confirmation.
2. Provide details on the nature of the breach, categories of data affected, and measures taken.
3. Cooperate with your supervisory authority if required.

Because sync payloads are encrypted with client-held keys, a server-side compromise would not expose the plaintext of your highlights or Streams.

---

## 7. Audit and Compliance Rights

You may request:

- A summary of the security measures in place (this document).
- Confirmation that no additional sub-processors have been engaged without notice.
- Access logs related to your account metadata (30-day lookback).

---

## 8. Termination

On termination of the Creator tier subscription or account deletion:

1. Gearbox ceases all sync processing.
2. All encrypted blobs and account metadata are purged within 7 days.
3. Any remaining loopback Stream URLs become invalid.
4. Telemetry data is anonymised and retained only in aggregated form.

---

## 9. Contact

- **DPA Inquiries:** dpa@gearbox.dev
- **Security Issues:** security@gearbox.dev
- **General Support:** beta@gearbox.dev

---

*This DPA is provided as a template for Gearbox Relay Creator tier users during the public beta. The final executed version may be customised upon request.*
