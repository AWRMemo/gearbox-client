# SEC-12-SYNC-METADATA: Sync Metadata Leakage

**Status:** `FIXED` — OpaqueBlob v2 integrated into `SyncEngine` (Sprint 15, 2026-05-24). v2 protocol now active; v1 deprecated.
**Severity:** Critical  
**Discovered:** Sprint 12 Security Audit (Agent 9)  
**Classification:** Metadata leakage in transit to sync server

---

## 1. Problem Statement

### 1.1 Current Behavior (v1 Protocol)

The `EncryptedBlob` structure sent in every sync request exposes three plaintext metadata fields **outside** the AES-256-GCM ciphertext envelope:

```rust
pub struct EncryptedBlob {
    pub id: String,              // e.g. "hl_abc123" — semantic record identity
    pub record_type: String,     // e.g. "highlight" | "stream" | "settings"
    pub last_modified: i64,      // Unix timestamp (ms)
    pub ciphertext: Vec<u8>,     // AES-256-GCM(AAD=none)
}
```

These fields are visible to:
- The sync server (`relay-sync-server/`) in plaintext
- Any intermediary TLS-terminating proxy or load balancer
- Server logs, metrics, tracing, and backup dumps

### 1.2 Exact Leakage

| Field | What It Leaks | Risk |
|-------|---------------|------|
| `id` | **Record identity** — deterministic per-record UUID or slug. Correlating `id` across syncs reveals which records changed, how often, and in what order. | Pattern-of-life inference; linkability across devices even if content is encrypted. |
| `record_type` | **Data type** — distinguishes "highlight" (personal reading) from "stream" (public curation) from "settings". Reveals user behavior categories. | Behavioral profiling; an adversary learns when the user is curating vs. capturing privately. |
| `last_modified` | **Temporal patterns** — exact timestamps of every edit, creation, deletion. Granular to millisecond. | Sleep/wake inference; workplace schedule deduction; co-location timing attacks. |

### 1.3 Why This Matters for a Privacy-First App

Gearbox Relay's privacy guarantee (PRD §10.2) states:

> "The sync server cannot read, index, or infer the contents of user data. The server performs only storage and delivery."

The current v1 protocol violates this guarantee by telemetry-grade metadata exposure. An operator of the sync server (or a compromised server) can:

1. Build a **temporal graph** of user activity (`last_modified` sequences)
2. **Correlate** `id` across devices to link phone, tablet, and desktop without ever decrypting content
3. **Categorize** behavior by `record_type` to infer user habits (e.g., "this user publishes streams every Tuesday at 9pm")

This is unacceptable for a product whose core value proposition is **zero-knowledge synchronization**.

---

## 2. Proposed Design — `OpaqueBlob` Protocol (v2)

### 2.1 Transport Structure

Replace `EncryptedBlob` with an **outer** transport envelope that contains zero semantic metadata:

```json
{
  "blob_id": "random_uuid",               // purely for transport deduplication
  "payload": "base64_encrypted_opaque_blob"
}
```

- `blob_id`: A random UUIDv4 generated per-sync-operation. It has **no correlation** with the semantic `id` inside the blob. Used only so the server can dedupe duplicate uploads on retry.
- `payload`: The entire inner JSON object, serialized and encrypted as a single AES-256-GCM blob.

### 2.2 Inner Opaque Blob (Client-Decrypt-Only)

After decryption, the client sees:

```json
{
  "id": "hl_abc123",
  "record_type": "highlight",
  "last_modified": 1716480000000,
  "data": {
    "text": "...",
    "source_url": "...",
    "tags": ["ai", "privacy"]
  }
}
```

All fields — `id`, `record_type`, `last_modified`, and the actual `data` — are inside the ciphertext envelope.

### 2.3 Server-Side Changes

- **Storage:** The server stores `(user_id, blob_id, payload_b64, received_at)`.
- **No indexing** by record type, date, or id. The server cannot answer queries like "give me all highlights from this user."
- **Conflict resolution:** On sync pull, the server returns **all unacknowledged blobs** for the user. The client decrypts each, compares `last_modified` inside, and applies LWW (last-write-wins) locally.
- **Garbage collection:** The server may retain blobs for 90 days (configurable) and then purge based on `received_at` only. No semantic knowledge required.

### 2.4 Client-Side Changes

- **Encryption path:** Before upload, serialize `{id, record_type, last_modified, data}` → JSON string → AES-256-GCM encrypt → base64 → set as `payload`.
- **Decryption path:** On download, base64-decode `payload` → AES-256-GCM decrypt → JSON parse → apply local LWW logic.
- **Sync status UI:** "3 pending highlights" becomes "3 pending items" (opaque count) or simply a spinner until decryption completes.

### 2.5 Payload Size Considerations

The overhead of JSON-serializing metadata inside the blob is minimal:

- Typical highlight: ~500 bytes of `data`
- Inner blob with metadata: ~550–600 bytes
- Encrypted payload (AES-GCM overhead + base64): ~800–900 bytes vs. ~750–850 today

**Conclusion:** Size increase is <10% and acceptable for the privacy gain.

---

## 3. Impact Analysis

### 3.1 What Breaks

| Area | v1 Behavior | v2 Change | Effort |
|------|-------------|-----------|--------|
| **Sync server DB schema** | Table has indexed columns `id`, `record_type`, `last_modified` | Flat table `(user_id, blob_id, payload, received_at)`; drop all semantic indexes | 2–3 days |
| **Conflict resolution UI** | Server resolves LWW based on `last_modified` in query | Client-side-only LWW after decrypt; server is dumb pipe | 3–4 days |
| **Sync status display** | "Pending highlights: 3" (server counts by `record_type`) | "Pending items: 3" (server counts raw blob rows) or spinner | 1 day |
| **Server metrics / dashboards** | Broken down by `record_type` | All become opaque; aggregate-only | 2 days |
| **Mobile+Desktop sync modules** | Direct deserialization into `EncryptedBlob` | Two-step: outer envelope → decrypt → inner struct | 3–4 days |
| **Integration tests** | Mock server exposes `id`/`record_type` | Mock server returns random `blob_id`s; client asserts after decrypt | 2–3 days |

### 3.2 Migration Path

1. **v1 retained:** The server continues to accept `EncryptedBlob` (v1) from existing clients indefinitely during the transition.
2. **v2 introduced:** New client versions send `OpaqueBlob`. Server stores it in a new table or schema variant.
3. **Dual-read:** Server sync-pull endpoint returns **both** v1 blobs (for old clients) and v2 blobs (for new clients), keyed by client version header (`X-Relay-Protocol-Version: 2`).
4. **Deprecation:** v1 deprecated in Sprint 15 (warning logs). v1 removed in Sprint 17 (reject uploads, still serve reads for stragglers).

### 3.3 Risk: Partial Migration

If a user has one device on v1 and one on v2, the v2 device will receive v1 blobs from the server. The v2 client must be able to **downgrade** gracefully: detect v1 structure, parse plaintext metadata fields, and treat them as if they were inside the opaque blob. This compatibility shim exists only during the deprecation window.

---

## 4. Security Review Checklist

| # | Requirement | Status in PRD | Verified By |
|---|-------------|---------------|-------------|
| 1 | AES-256-GCM + Argon2id key derivation **unchanged** | Specified | External reviewer (Sprint 14 Week 1) |
| 2 | IV/nonce uniqueness: generate a new 96-bit nonce per encryption operation, never reuse `(key, nonce)` | Specified | External reviewer |
| 3 | `blob_id` is random UUIDv4, not derived from `id`, timestamp, or counter | Specified | External reviewer |
| 4 | Server never logs or persists `payload` in plaintext; only `blob_id` and `received_at` appear in logs | Policy | External reviewer |
| 5 | Client-side LWW logic is constant-time with respect to `last_modified` comparison (to prevent timing side-channels) | To be implemented | Penetration test (Sprint 15) |
| 6 | v1→v2 downgrade parser strips all plaintext metadata immediately after ingestion, never stores it locally | To be implemented | Code review + static analysis |

---

## 5. Implementation Estimate

| Phase | Sprint | Scope | Agents | Dev-Days | Budget |
|-------|--------|-------|--------|----------|--------|
| **Design** | Sprint 13 | PRD section + security checklist + migration path | 1 (Agent 5) | 2 | — |
| **External Review** | Sprint 14 Week 1 | Independent security auditor reviews PRD + checklist | External | 1 week | **$5,000** |
| **Implementation** | Sprint 14 | Server schema, client encrypt/decrypt, LWW move, tests | 4 | ~20 | — |
| **Migration Testing** | Sprint 15 | Dual-protocol integration, downgrade shim, stress test | 2 | ~10 | — |
| **Deprecation** | Sprint 15 | v1 marked deprecated; warning telemetry | — | — | — |
| **Removal** | Sprint 17 | v1 upload rejected; read-only for stragglers | 1 | 3 | — |

**Total estimated cost (excluding external review):** ~35 dev-days  
**Total calendar time:** 3 sprints (Sprint 14–17)

---

## 6. Alternatives Considered

| Alternative | Rationale for Rejection |
|-------------|------------------------|
| **Encrypt individual metadata fields** (e.g., `encrypted_id`, `encrypted_last_modified`) | Still leaks field count and approximate field sizes. Does not prevent correlation attacks on blob count or timing. |
| **Homomorphic encryption for server-side LWW** | Overkill. Adds 100× latency, complex key management, and no trusted implementation in Rust/Go ecosystems. |
| **CRDT-based sync (Yjs)** | Explicitly forbidden in AGENTS.md. Also does not solve metadata leakage — CRDTs expose even more metadata (clock vectors, peer IDs). |
| **Keep v1 forever, add "privacy mode" toggle** | Adds complexity, splits user base, and violates the "privacy by default" principle in PRD §10. |

---

## 7. Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-23 | Defer implementation to Sprint 14 | Requires PRD rewrite + external review first (AGENTS.md boundary). |
| 2026-05-23 | Select `OpaqueBlob` (full metadata inside ciphertext) over per-field encryption | Eliminates all semantic leakage; simplest mental model for auditors. |
| 2026-05-23 | Retain v1 server support during deprecation window | Avoids bricking users who do not auto-update immediately. |

---

## 8. References

- `AGENTS.md` — Sprint 12 Gaps & Sprint 13 Decisions
- `docs/prd/sync-v2-opaque-blob.md` — Full protocol design (this document's normative spec)
- `relay-core/src/sync/` — Current v1 implementation (do not modify until Sprint 14)
- `relay-sync-server/` — Proprietary server (snapshot excluded from public repo)
