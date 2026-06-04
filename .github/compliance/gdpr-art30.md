# GDPR Article 30 — Records of Processing Activities

**Version:** 1.0-beta  
**Effective Date:** 2026-05-25  
**Entity:** Gearbox Labs, Inc.  
**Article:** Article 30 of Regulation (EU) 2016/679 (GDPR)

---

## 1. Identity and Contact Details

| Field | Value |
|-------|-------|
| **Controller** | Gearbox Labs, Inc. |
| **Data Protection Officer** | privacy@gearbox.dev |
| **DPA Contact** | dpa@gearbox.dev |

---

## 2. Purposes of Processing

| # | Purpose | Legal Basis | Description |
|---|---------|-------------|-------------|
| 2.1 | Local knowledge management | Legitimate interest (user-initiated) | Capture, enrich, search, and organise personal text highlights, notes, and Streams on the user's own device. |
| 2.2 | Multi-device sync | Contract (optional sync service) | Transmit encrypted sync blobs over the network so that a user's data is consistent across their authorised devices. |
| 2.3 | Service reliability | Legitimate interest (with opt-in consent) | Collect scrubbed crash reports and performance timing metrics to improve application stability. |
| 2.4 | Account authentication | Contract (user account) | Manage user credentials, session tokens, and device registration for the optional sync service. |
| 2.5 | Public Stream sharing | Consent (Creator tier, explicit publish action) | Host a loopback-only preview page when a user chooses to publish a Stream. |

---

## 3. Categories of Personal Data and Data Subjects

| Data Subject | Data Categories | Source |
|-------------|-----------------|--------|
| End user (Free/Pro tier) | Captured text, AI tags, AI summaries, search vectors | User capture or clipboard import |
| End user (Creator tier) | Highlight collections, Stream metadata, subscriber counts | User curation and publish actions |
| Sync account holder | Hashed email, device tokens (opaque), protocol version | User registration |
| Telemetry participant (opt-in) | Scrubbed stack traces, millisecond timing spans, parse-success booleans | Application runtime |

**No special categories (Art. 9 GDPR):** Gearbox Relay does not process health, biometric, genetic, or political opinion data.

---

## 4. Recipients and Categories of Recipients

| Recipient Category | What Is Shared | Legal/Contractual Basis |
|-------------------|----------------|------------------------|
| **No recipient** (local data) | N/A | N/A — data never leaves the device for core features. |
| **Gearbox sync server** | OpaqueBlob v2 encrypted blobs (`blob_id` + AES-256-GCM ciphertext only) | Contract — user voluntarily enables sync. |
| **Sentry (Functional Software, Inc.)** | Scrubbed crash reports and performance timing (PII stripped before transmission) | Legitimate interest + opt-in consent. DPA in place. |

**No onward transfers:** Encrypted blobs are not decrypted, indexed, or re-shared by Gearbox. Sentry data is not sold or profiled.

---

## 5. Transfers of Personal Data to Third Countries

| Transfer | Destination | Safeguard |
|----------|-------------|-----------|
| Sentry crash/performance data | United States | EU-hosted Sentry instance available upon request; Standard Contractual Clauses (SCCs) in place for US-hosted events. |
| Sync server traffic | Determined by user's closest CDN edge / data centre | TLS 1.3 in transit; encryption keys held by user. |

---

## 6. Retention Periods

| Data Category | Retention Period | Rationale |
|--------------|------------------|-----------|
| Local highlights, tags, summaries | Until user deletion or app uninstall | Local-first architecture; user retains full possession. |
| Encrypted sync blobs (unacknowledged) | Until user deletes account | Needed for multi-device consistency. |
| Encrypted sync blobs (acknowledged) | 90 days after client acknowledgement | Garbage-collection window for rollbacks and device re-sync. |
| Account metadata (hashed email, tokens) | Until account deletion | Required for authentication and push routing. |
| Telemetry (opt-in) | 90 days | Sentry automatic retention; no long-term storage by Gearbox. |
| Server access logs | 30 days | Rate-limiting, abuse detection, and audit. |

---

## 7. Security Measures (Art. 32)

| Layer | Measure |
|-------|---------|
| **Encryption at rest (local)** | OS-level file permissions on SQLite DB; app sandbox on mobile. |
| **Encryption in transit (sync)** | AES-256-GCM authenticated encryption with protocol-version AAD (`relay-sync-v2`). |
| **Key derivation** | Argon2id from user sync password; Gearbox never stores the password or derived key. |
| **Network security** | TLS 1.3 for all sync traffic; loopback-only (`127.0.0.1:0`) for optional Stream preview. |
| **Access control** | Role-based access on server infra; audit logging of engineer access; HSM/encrypted secret stores for service credentials. |
| **Code integrity** | Apache 2.0 client source; CI gates (`cargo clippy`, `cargo test`, `flutter analyze`); weekly `cargo audit` / `pnpm audit`. |
| **Availability** | Rate limiting on sync server; encrypted blob replication across at least two availability zones. |

---

## 8. Data Subject Rights Fulfilment

| Right | How Fulfilled |
|-------|---------------|
| **Right of access (Art. 15)** | User exports local data via Settings → Export Data (ZIP of SQLite + settings). |
| **Right to rectification (Art. 16)** | User edits highlights, tags, and summaries locally; changes sync automatically on next push. |
| **Right to erasure (Art. 17)** | "Clear Data" in Settings wipes local data; account deletion purges server blobs and metadata within 7 days. |
| **Right to restrict processing (Art. 18)** | Disable sync and telemetry at any time in Settings; app remains fully functional offline. |
| **Right to data portability (Art. 20)** | Export ZIP contains standard SQLite schema and JSON settings; no proprietary lock-in. |
| **Right to object (Art. 21)** | Telemetry toggle OFF in Settings; no profiling or automated decision-making occurs. |

---

## 9. Record Maintenance

This document is reviewed **quarterly** and updated within 14 days of any material change to processing activities, data categories, or sub-processors. DPO contact: privacy@gearbox.dev.

---

*This record is maintained in accordance with GDPR Article 30. The information reflects the Gearbox Relay public beta as of Sprint 17.*
