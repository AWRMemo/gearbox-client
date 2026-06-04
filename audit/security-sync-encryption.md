# Sync Payload Encryption Audit

**Auditor:** Agent 9 (Sprint 12)
**Scope:** `relay-core/src/sync/encrypt.rs`, `relay-core/src/sync/engine.rs`, `relay-core/src/sync/server.rs`, `relay-core/src/sync/conflict.rs`, `relay-sync-server/src/handlers.rs`, `relay-sync-server/src/models.rs`, `src-tauri/src/commands/auth.rs`
**Date:** 2026-05-23

---

## 1. Algorithm Choice

**Finding:** Encryption uses `aes_gcm::Aes256Gcm` (RustCrypto crate). GCM provides both confidentiality and integrity/authentication (AEAD).

**Status:** PASS — AES-256-GCM is approved. No ECB or CBC usage detected.

---

## 2. IV/Nonce Uniqueness

**Finding:** `generate_nonce()` produces a 12-byte random nonce via `ring::SystemRandom`. Every call to `encrypt()` invokes a fresh nonce.

**Status:** PASS — probability of collision is negligible (2^-96). No counter reuse risk detected.

---

## 3. Key Derivation

**Finding:** `derive_key(password, salt)` uses Argon2id with default parameters (`Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::default())`).

**Status:** PASS — key is derived from user password + per-account salt.

**Caveat:** Desktop (`src-tauri/src/commands/auth.rs`) stores `salt_auth` and `salt_encrypt` in plaintext in `sync_credentials`. If the local database is exfiltrated, the salts are known to the attacker. This is expected for password-based KDFs; the salts do not need secrecy but they must be unique per user.

---

## 4. Encrypted Payload: Plaintext Metadata Leakage

**Finding (HIGH):** `EncryptedBlob` (server-side model) contains `id`, `record_type`, and `last_modified` **outside** the ciphertext.

File: `relay-sync-server/src/models.rs:32-37` and `relay-core/src/sync/server.rs:6-11`

```rust
pub struct EncryptedBlob {
    pub id: String,
    pub record_type: String,
    pub ciphertext: String,
    pub last_modified: String,
}
```

The server's `/v1/sync/push` and `/v1/sync/pull` endpoints store and return these fields in cleartext. A passive observer or compromised sync server can:
- Correlate `id` with known public stream IDs.
- Infer record types ("highlights", "streams", etc.).
- Build a timeline graph from `last_modified`.

**AGENTS.md states:** "NEVER send plaintext user data to the sync server. All sync payloads are encrypted blobs."

**Status:** CRITICAL — violates project security boundary.

**Remediation (recommended but out-of-scope per AGENTS.md lock):** The encryption scheme should be redesigned so that the entire JSON payload (including metadata) is inside the ciphertext, and the server receives only a user-derived deterministic "record handle" plus the encrypted payload. However, per AGENTS.md: "NEVER change the sync encryption scheme without a full security review and PRD update."

**Escalation:** Create ticket `SEC-12-SYNC-METADATA` for PRD review.

---

## 5. Conflict Log Leakage

**Finding (LOW):** `sync_conflicts` table stores `local_version` and `remote_version` in plaintext if those values happen to be conflict payloads (they are currently timestamps, not raw text, but `remote_version` in some test cases stores JSON).

**Status:** PASS for current schema — `local_version` and `remote_version` are timestamps/strings, not full record content. However, the `remote_version` field is typed `TEXT` and could accidentally hold plaintext user data in future code paths.

**Recommendation:** Add a code-review guard: `sync_conflicts` must never contain decrypted content.

---

## 6. Password Hash Transmission

**Finding (HIGH):** `SyncServerClient::register` and `SyncServerClient::login` send `password_hash` (which is actually a base64-encoded Argon2id-derived key) over HTTP. The code uses `ureq`, but no TLS/certificate pinning is enforced inside `relay-core::sync::server`. The actual server URL comes from user configuration (`auth.rs:31` defaults to `http://localhost:3000` — note **HTTP**, not HTTPS).

**Status:** HIGH — credential leakage if the production sync endpoint is not upgraded to HTTPS.

**Remediation:**
1. Change default server URL to `https://`.
2. Add a runtime check in `SyncServerClient::register` and `::login` that rejects non-HTTPS targets in Release mode.
3. Rotate stored `password_hash` derivation to use an additional pepper (server-side secret) so the value sent over the wire is not the same as the local key derivation output.

---

## 7. Replay Risk on Pull

**Finding (MEDIUM):** `EncryptedBlob` lacks a sequence number or monotonic counter. A middleperson could replay an older blob payload and the client would decrypt it successfully. LWW timestamp logic would then silently overwrite newer local data if the replayed `last_modified` is forged.

**Status:** MEDIUM — requires timestamp forgery or server compromise.

**Remediation (future):** Tie `last_modified` to a server-side signed timestamp or include a per-user monotonic counter in the encrypted payload.

---

## Summary

| # | Check | Severity | Status |
|---|-------|----------|--------|
| 1 | AES-256-GCM algorithm | — | PASS |
| 2 | Unique nonce per operation | — | PASS |
| 3 | Argon2id key derivation | — | PASS |
| 4 | **Plaintext metadata (`id`, `record_type`, `last_modified`) outside ciphertext** | **Critical** | **Escalated** (`SEC-12-SYNC-METADATA`) |
| 5 | Conflict log plaintext | Low | PASS / watch |
| 6 | Password hash sent over HTTP | **High** | **FIXED** (default url + guard) |
| 7 | Replay / timestamp forging | Medium | Accepted Risk / future hardening |

**Code changes applied:**
- `relay-core/src/sync/server.rs`: Added HTTPS-only check in `register` and `login`.
- `src-tauri/src/commands/auth.rs`: Updated default server URL to `https://relay-sync.gearbox.local/v1` placeholder.

**Escalated ticket:** `SEC-12-SYNC-METADATA` — full redesign of encrypted blob envelope.
