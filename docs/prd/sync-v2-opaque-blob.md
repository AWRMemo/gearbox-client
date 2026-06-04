# Sync Protocol v2: OpaqueBlob

> **Normative specification for the v2 sync protocol.**  
> **Status:** `DESIGNED` — implementation scheduled Sprint 14.  
> **Supersedes:** v1 `EncryptedBlob` protocol (retained during deprecation window).  
> **Authority:** This document. Deviations require PRD amendment + security review.

---

## 1. Goals

1. **Zero-knowledge server:** The sync server must not learn the semantic `id`, `record_type`, or `last_modified` of any user record.
2. **Minimal overhead:** Payload size increase <10% vs. v1.
3. **Backward compatibility:** v1 and v2 clients coexist during a deprecation window (Sprint 14–17).
4. **No new primitives:** Reuse existing AES-256-GCM + Argon2id key derivation. Do not introduce new cryptographic algorithms.

---

## 2. Terminology

| Term | Definition |
|------|------------|
| **OpaqueBlob** | The v2 transport structure. A single encrypted payload with random `blob_id`. |
| **Inner Blob** | The decrypted JSON object containing all semantic metadata + data. |
| **Outer Envelope** | The minimal plaintext structure visible to the server (`blob_id`, `payload`). |
| **LWW** | Last-Write-Wins conflict resolution (timestamp comparison). |

---

## 3. Data Structures

### 3.1 Outer Envelope (Server-Visible)

```json
{
  "blob_id": "uuid-v4",
  "payload": "base64-string"
}
```

**Fields:**

- `blob_id` (string, UUIDv4): Deduplication key. Generated fresh per upload attempt. No semantic link to inner `id`.
- `payload` (string, base64): The AES-256-GCM ciphertext of the serialized Inner Blob.

**Server invariants:**
- `blob_id` is unique per `(user_id, blob_id)` tuple. Duplicate `blob_id` uploads are idempotent (server returns `200` without overwriting).
- The server never inspects, decodes, indexes, or logs `payload`.

### 3.2 Inner Blob (Client-Only)

After decryption and UTF-8 deserialization:

```json
{
  "id": "string",
  "record_type": "highlight|stream|settings|...",
  "last_modified": 1716480000000,
  "data": { ... }
}
```

**Fields:**

- `id` (string): Semantic record identifier. Format unchanged from v1.
- `record_type` (string): Domain type enum. Extensible; new types may be added in future sprints.
- `last_modified` (int64, Unix ms): Conflict-resolution timestamp. Must be monotonic per-record from the client's local clock (not wall-clock; see §5.3).
- `data` (object): Domain-specific payload. Schema defined per `record_type`.

### 3.3 Encrypted Payload Construction

```
inner_json = serde_json::to_string(InnerBlob)
nonce      = crypto::rand_bytes(12)          // never reused with same key
aad        = "relay-sync-v2"                  // protocol version as AAD
ciphertext = AES_256_GCM_encrypt(
                key = derived_key,
                nonce = nonce,
                plaintext = inner_json.as_bytes(),
                aad = aad.as_bytes()
             )
payload    = base64(nonce || ciphertext || auth_tag)
```

**Notes:**
- `derived_key` is the existing Argon2id-derived key from the user's sync password. No change.
- AAD (`"relay-sync-v2"`) is added to prevent v1 ciphertexts from being misinterpreted as v2, and to bind the ciphertext to the protocol version.

---

## 4. API Changes

### 4.1 Upload Blob

**Endpoint:** `POST /v2/sync/blobs`

**Headers:**
```
Authorization: Bearer <jwt>
Content-Type: application/json
X-Relay-Protocol-Version: 2
```

**Request Body:**
```json
{
  "blob_id": "uuid-v4",
  "payload": "base64-string"
}
```

**Response:**
- `201 Created` — new blob stored.
- `200 OK` — blob_id already known (idempotent).
- `400 Bad Request` — malformed base64 or missing fields.
- `413 Payload Too Large` — payload exceeds 1 MB per blob (configurable).

### 4.2 List Blobs (Sync Pull)

**Endpoint:** `GET /v2/sync/blobs?since=<cursor>`

**Headers:**
```
Authorization: Bearer <jwt>
X-Relay-Protocol-Version: 2
```

**Response:**
```json
{
  "blobs": [
    { "blob_id": "uuid", "payload": "base64" },
    ...
  ],
  "cursor": "opaque-cursor-string",
  "has_more": false
}
```

**Behavior:**
- Returns **all** unacknowledged blobs for the authenticated user, paginated.
- The server does **not** filter, sort, or interpret payload content.
- Ordering is by `received_at` (server ingestion time), which is visible to the server but not to the client. This is acceptable because `received_at` is not user-provided metadata.

### 4.3 Acknowledge Blobs

**Endpoint:** `POST /v2/sync/ack`

**Body:**
```json
{
  "blob_ids": ["uuid-1", "uuid-2"]
}
```

After acknowledgment, the server may garbage-collect blobs per its retention policy.

---

## 5. Client Logic

### 5.1 Upload Path

```rust
fn prepare_upload(record: &Record) -> OpaqueBlob {
    let inner = InnerBlob {
        id: record.id.clone(),
        record_type: record.record_type.clone(),
        last_modified: record.last_modified,
        data: serde_json::to_value(&record.data).unwrap(),
    };

    let inner_json = serde_json::to_string(&inner).unwrap();
    let nonce = crypto::random_nonce();          // 12 bytes
    let aad = b"relay-sync-v2";
    let ciphertext = aes_256_gcm_encrypt(&self.key, &nonce, inner_json.as_bytes(), aad);

    OpaqueBlob {
        blob_id: Uuid::new_v4().to_string(),    // NOT derived from record.id
        payload: base64_encode(&nonce, &ciphertext),
    }
}
```

### 5.2 Download + Decrypt Path

```rust
fn ingest_blob(&mut self, blob: OpaqueBlob) -> Result<Record, SyncError> {
    let (nonce, ciphertext) = base64_decode(&blob.payload)?;
    let aad = b"relay-sync-v2";
    let inner_json = aes_256_gcm_decrypt(&self.key, &nonce, &ciphertext, aad)?;
    let inner: InnerBlob = serde_json::from_str(&inner_json)?;

    // LWW conflict resolution
    if let Some(existing) = self.local_store.get(&inner.id) {
        if existing.last_modified >= inner.last_modified {
            return Ok(existing.clone()); // local wins
        }
    }

    self.local_store.upsert(inner.into_record())?;
    Ok(inner.into_record())
}
```

### 5.3 Clock & Monotonicity

- **`last_modified` is a local monotonic counter**, not wall-clock time. It is the output of a `u64` counter incremented on every local mutation, with an initial seed from `SystemTime::now().duration_since(UNIX_EPOCH)`.
- This prevents clock-skew attacks and ensures deterministic LWW on a single device.
- The counter is persisted in local SQLite and restored on app startup.
- Cross-device: if two devices both use monotonic counters starting from different seeds, collisions are possible. LWW resolves ties deterministically by falling back to lexical comparison of `blob_id` (never by content hash, which would leak data).

### 5.4 Sync Status Reporting

Because the server cannot count "pending highlights," the client must derive sync status locally:

- **Upload queue:** Client counts unsent `OpaqueBlob`s in its local outbound queue. UI shows "Syncing N items..."
- **Download queue:** Client counts received but not-yet-processed `blob_id`s. UI shows "Receiving updates..."
- **Per-type breakdown:** Only available after decryption. UI may show a generic spinner until the first batch is decrypted, then switch to detailed counts.

---

## 6. Server Logic

### 6.1 Schema (Simplified)

```sql
CREATE TABLE user_blobs (
    user_id       TEXT NOT NULL,
    blob_id       TEXT NOT NULL,
    payload       BLOB NOT NULL,   -- opaque; never inspected
    received_at   INTEGER NOT NULL, -- server ingestion time (Unix ms)
    acknowledged  INTEGER DEFAULT 0,
    PRIMARY KEY (user_id, blob_id)
);

CREATE INDEX idx_user_blobs_receive ON user_blobs(user_id, received_at);
-- NO index on payload, no index on semantic fields (there are none)
```

### 6.2 Garbage Collection

- **Policy:** Blobs are retained for 90 days after `acknowledged = 1`.
- **Unacknowledged blobs:** Retained indefinitely (or until account deletion).
- **Implementation:** Daily cron job deletes `WHERE acknowledged = 1 AND received_at < now() - 90 days`.

### 6.3 Back-Pressure

- Maximum unacknowledged blobs per user: 10,000 (configurable).
- If exceeded, server returns `429 Too Many Requests` on upload + a `Retry-After` header. Client must trigger a pull/ack cycle before uploading more.

---

## 7. Migration & Compatibility

### 7.1 Protocol Version Negotiation

Clients send `X-Relay-Protocol-Version: 2`. The server behavior:

| Client Version | Upload | Pull |
|----------------|--------|------|
| Missing / 1 | Accept `EncryptedBlob` (v1). Store in `user_blobs_legacy`. | Return v1 blobs from `user_blobs_legacy`. |
| 2 | Accept `OpaqueBlob`. Store in `user_blobs`. | Return v2 blobs from `user_blobs`. Do NOT include v1 blobs. |

> **Rationale:** v2 clients do not need to see v1 blobs from their own device because v2 clients write v2 blobs. However, during the transition, a v2 client may need to see v1 blobs from a **peer device** that has not upgraded. Therefore, the **actual** pull endpoint must support a `?include_legacy=true` query parameter, default `false`, which v2 clients set to `true` until Sprint 17.

### 7.2 Deprecation Schedule

| Sprint | Milestone |
|--------|-----------|
| **14** | v2 implemented. Both v1 and v2 fully supported. |
| **15** | v1 marked deprecated. Server logs warnings for v1 uploads. Client shows "update available" nudge if peer is on v1. |
| **16** | v1 upload rate-limited (e.g., max 1 upload/minute). |
| **17** | v1 upload rejected with `410 Gone`. v1 read-only endpoint remains for stragglers. |
| **18+** | v1 endpoints may be removed after 90-day retention window expires. |

### 7.3 v2 Client Ingestion of v1 Blobs (Compatibility Shim)

When `?include_legacy=true` is used, the server returns a hybrid list:

```json
{
  "blobs": [
    { "blob_id": "uuid", "payload": "base64-opaque" },
    { "legacy_id": "hl_abc", "legacy_record_type": "highlight", "legacy_last_modified": 123, "legacy_ciphertext": "base64" }
  ]
}
```

The v2 client:
1. Detects the presence of `legacy_*` fields.
2. Skips the outer decrypt step.
3. Directly decrypts `legacy_ciphertext` using v1 AAD (none).
4. Constructs an `InnerBlob` from the plaintext metadata + decrypted data.
5. Stores it as if it were a v2 blob.
6. After successful ingestion, the client writes a **new v2 tombstone blob** with the same `id` and a fresh `last_modified` to force the legacy peer to upgrade on next sync.

---

## 8. Threat Model Assumptions

| Assumption | Justification |
|------------|--------------|
| The user's sync password is strong and unique. | Out of scope; enforced by UI policy + zxcvbn. |
| The client device is not compromised at the time of encryption. | If compromised, all local data is exposed anyway; sync confidentiality is the least concern. |
| TLS between client and server is intact. | Standard assumption; if broken, attacker sees payload size and timing, but not plaintext. |
| The server operator is honest-but-curious. | OpaqueBlob defends against this; if operator is actively malicious, availability attacks are still possible (out of scope). |

---

## 9. Audit & Test Requirements

### 9.1 Unit Tests (Client)

- Encrypt → decrypt round-trip: 100 randomized `InnerBlob` shapes.
- `blob_id` distribution: verify UUIDv4, no correlation with `id` or timestamp (statistical test).
- AAD mismatch: attempt to decrypt v2 payload with v1 AAD (`None`) → must fail authentication.
- Legacy shim: ingest a synthetic v1 blob, verify it becomes a v2 record in local DB.

### 9.2 Integration Tests

- Offline capture → online sync with mismatched timestamps (existing test pattern, extended to v2).
- Multi-device: v1 desktop + v2 mobile sync to same account; verify LWW correctness.
- Server back-pressure: upload 10,001 blobs, verify `429` + client retry behavior.

### 9.3 Penetration Test Targets (Sprint 15)

- Payload size side-channel: vary `data` size and measure TLS packet sizes. Ensure padding or chunking prevents record-size inference.
- Timing side-channel: measure decryption + LWW comparison time with varying `last_modified` values. Ensure constant-time comparison or randomized delay.

---

## 10. Glossary

| Term | Definition |
|------|------------|
| AAD | Additional Authenticated Data (AEAD context) |
| AEAD | Authenticated Encryption with Associated Data (AES-GCM) |
| LWW | Last-Write-Wins |
| Nonce | Number-used-once (IV for AES-GCM) |
| OpaqueBlob | The outer transport structure in v2 |

---

## 11. Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-05-23 | Agent 5 | Initial PRD draft for Sprint 13 |
