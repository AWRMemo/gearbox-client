# Sprint 5: Sync Infrastructure & Multi-Device Data Mobility
**Technical Specification · No Half-Measures**
**Version 1.0 · May 2026**

---

## 1. Executive Summary

Sprint 5 delivers the sync protocol, encrypted transport, server, and identity system that unlocks the PRD's two-sided Stream subscription growth loop. Without sync, Relay is a single-device highlight annotator. With sync, it becomes a multi-device personal knowledge pipeline with viral distribution.

This is the highest-risk, highest-reward sprint in the entire product roadmap. There are no shortcuts: every component (encryption, protocol, server, auth, conflict UI) must be production-hardened before launch.

---

## 2. Current State vs Target State

| Layer | Current (End of Sprint 4) | Target (End of Sprint 5) |
|---|---|---|
| **Identity** | Per-device `device_id.txt` (random UUID) | User-authenticated identity (email + password) with Argon2id-derived encryption key |
| **Sync** | None | LWW protocol: push/pull/merge background engine |
| **Encryption** | None (SQLite plaintext) | AES-256-GCM. Server stores only ciphertext |
| **Server** | None | `relay-sync-server`: REST API for auth + encrypted blob storage |
| **Schema** | No `last_modified` or `sync_status` columns | Every mutable table has `last_modified` and `sync_status` |
| **Conflict UI** | None | User-visible conflict log; resolution action per conflict |
| **Mobile** | No mobile code | Flutter vs KMP spike executed; framework decision locked |

---

## 3. Architecture

### 3.1 Component Diagram

```
┌─────────────────────────────────────────────────┐
│  DESKTOP CLIENT (src-tauri/)                     │
│                                                  │
│  ┌──────────────┐  ┌──────────────┐              │
│  │ sync/engine  │  │ sync/encrypt │              │
│  │ push/pull    │  │ AES-256-GCM  │              │
│  │ merge        │  │ Argon2id KDF │              │
│  └──────┬───────┘  └──────┬───────┘              │
│         │                  │                      │
│  ┌──────┴──────────────────┴───────┐              │
│  │        SQLite (relay.db)        │              │
│  │  + last_modified, sync_status   │              │
│  │  + sync_metadata                │              │
│  │  + sync_conflicts               │              │
│  └─────────────────────────────────┘              │
│         │                                         │
│  ┌──────┴───────┐  ┌──────────────────┐          │
│  │ Tauri        │  │ React Frontend   │          │
│  │ commands     │  │ Settings Auth UI │          │
│  │ invoke       │  │ Conflict Panel   │          │
│  └──────────────┘  │ Sync Status UI   │          │
│                    └──────────────────┘          │
└─────────────────────────────────────────────────┘
         │ HTTPS
         ▼
┌─────────────────────────────────────────────────┐
│  SYNC SERVER (relay-sync-server/)                │
│                                                  │
│  ┌──────────────┐  ┌────────────────────────┐   │
│  │ Auth service │  │ Encrypted Blob Store    │   │
│  │ JWT          │  │ user_id → [blob_id,...] │   │
│  │ password hash│  │ Stores ciphertext ONLY  │   │
│  └──────────────┘  └────────────────────────┘   │
│                                                  │
│  Endpoints:                                      │
│  POST /v1/auth/register                          │
│  POST /v1/auth/login                             │
│  GET  /v1/sync/pull?since={iso8601}              │
│  POST /v1/sync/push                              │
└─────────────────────────────────────────────────┘
```

### 3.2 Key Architectural Decisions (Locked)

1. **Server stores only ciphertext.** Plaintext never exists on the server. If the server is compromised, attackers get encrypted blobs and no keys.
2. **LWW (Last-Writer-Wins), not CRDT.** Higher `last_modified` timestamp wins. Conflict log is user-visible. No merging of highlights.
3. **Desktop client owns sync engine.** The server is a dumb blob store. All push/pull/merge logic lives in the Rust client.
4. **`src-tauri/src/sync/` is a new top-level module.** No business logic in command handlers (per AGENTS.md). Commands delegate to `sync/`.
5. **Sync server is a separate repository/crate.** Open-source client, proprietary server (per PRD §10).
6. **Encryption key is derived from the user's password via Argon2id.** If the user changes their password, they must re-upload all data from the device that has the plaintext. This limitation is documented and acceptable for MVP.

---

## 4. Sync Protocol (Detailed)

### 4.1 Schema Migration

Every mutable table gains:

```sql
ALTER TABLE highlights ADD COLUMN last_modified TEXT NOT NULL DEFAULT (datetime('now'));
ALTER TABLE highlights ADD COLUMN sync_status TEXT NOT NULL DEFAULT 'local';

ALTER TABLE streams ADD COLUMN last_modified TEXT NOT NULL DEFAULT (datetime('now'));
ALTER TABLE streams ADD COLUMN sync_status TEXT NOT NULL DEFAULT 'local';

ALTER TABLE stream_highlights ADD COLUMN last_modified TEXT NOT NULL DEFAULT (datetime('now'));
ALTER TABLE stream_highlights ADD COLUMN sync_status TEXT NOT NULL DEFAULT 'local';

ALTER TABLE subscriptions ADD COLUMN last_modified TEXT NOT NULL DEFAULT (datetime('now'));
ALTER TABLE subscriptions ADD COLUMN sync_status TEXT NOT NULL DEFAULT 'local';

ALTER TABLE user_profile ADD COLUMN last_modified TEXT NOT NULL DEFAULT (datetime('now'));
ALTER TABLE user_profile ADD COLUMN sync_status TEXT NOT NULL DEFAULT 'local';
```

New tables:

```sql
CREATE TABLE IF NOT EXISTS sync_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
-- Keys: 'last_sync_timestamp', 'device_name', 'sync_server_url'

CREATE TABLE IF NOT EXISTS sync_conflicts (
    id TEXT PRIMARY KEY,
    record_type TEXT NOT NULL,
    record_id TEXT NOT NULL,
    local_version TEXT,    -- JSON of the local version that lost
    remote_version TEXT,   -- JSON of the remote version that won
    resolved_at TEXT,
    resolution TEXT,       -- 'accept_remote', 'keep_local', 'manual_merge'
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sync_credentials (
    user_email TEXT PRIMARY KEY,
    password_hash TEXT NOT NULL,
    encryption_key_salt TEXT NOT NULL,
    server_url TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 4.2 Encryption Design

**Key Derivation Flow:**
```
User Password (input)
    │
    ├─► Argon2id (salt_1) → password_hash   → stored on server for auth
    │
    └─► Argon2id (salt_2) → encryption_key  → NEVER leaves device
```

**Encryption Per Blob (AES-256-GCM):**
```
plaintext:  JSON serialization of a single record
nonce:      96-bit random (crypto random)
ciphertext: AES-256-GCM(encryption_key, nonce, plaintext)
blob:       base64(nonce || ciphertext || auth_tag)
```

**What Gets Encrypted Per Record:**
- `highlights`: id, text, source_url, source_title, source_author, summary, tags, connection_suggestion, created_at, last_modified
- `streams`: id, title, description, is_public, created_at, last_modified
- `subscriptions`: user_id, stream_id, subscribed_at, last_modified
- `user_profile`: display_name, email, tier, last_modified

### 4.3 Push Algorithm (Client → Server)

```rust
fn sync_push(conn: &Connection, server_url: &str, auth_token: &str) -> Result<usize, String> {
    // 1. Collect local changes
    let local_rows = conn.prepare(
        "SELECT id, typeof, content_json, last_modified
         FROM highlights WHERE sync_status = 'local'
         UNION ALL
         SELECT id, 'stream', ...
         FROM streams WHERE sync_status = 'local'
         UNION ALL
         SELECT id, 'subscription', ...
         FROM subscriptions WHERE sync_status = 'local'
         WHERE last_modified > COALESCE(
             (SELECT value FROM sync_metadata WHERE key = 'last_sync_timestamp'),
             '1970-01-01T00:00:00Z'
         )"
    )?.query_map([], ...)?;

    // 2. Mark as syncing
    conn.execute("UPDATE highlights SET sync_status = 'syncing' WHERE sync_status = 'local'")?;
    // ... (same for all tables)

    // 3. Encrypt each row
    let blobs: Vec<EncryptedBlob> = local_rows
        .map(|row| encrypt_row(row, &encryption_key))
        .collect::<Result<Vec<_>, _>>()?;

    // 4. POST to server
    let response = reqwest::blocking::Client::new()
        .post(format!("{}/v1/sync/push", server_url))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&blobs)
        .send()?;

    // 5. On success, mark as synced
    conn.execute("UPDATE highlights SET sync_status = 'synced' WHERE sync_status = 'syncing'")?;
    // ...

    Ok(blobs.len())
}
```

### 4.4 Pull Algorithm (Server → Client)

```rust
fn sync_pull(conn: &Connection, server_url: &str, auth_token: &str) -> Result<usize, String> {
    let last_sync = get_metadata(conn, "last_sync_timestamp")
        .unwrap_or("1970-01-01T00:00:00Z");

    // 1. GET from server
    let response = reqwest::blocking::Client::new()
        .get(format!("{}/v1/sync/pull?since={}", server_url, last_sync))
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()?;

    let blobs: Vec<EncryptedBlob> = response.json()?;

    // 2. Decrypt and apply each blob
    for blob in &blobs {
        let plaintext = decrypt_blob(blob, &encryption_key)?;
        let remote: SyncRecord = serde_json::from_str(&plaintext)?;

        if !record_exists(conn, &remote) {
            insert_from_sync(conn, &remote)?;
        } else {
            let local_lm = get_last_modified(conn, &remote)?;
            // LWW: higher timestamp wins
            if remote.last_modified > local_lm {
                // Remote is newer: overwrite local
                update_from_sync(conn, &remote)?;
            } else if remote.last_modified < local_lm {
                // Local is newer: log conflict and keep local
                log_conflict(conn, &remote, ConflictResolution::KeepLocal)?;
            }
            // Equal timestamps: skip (already synced)
        }
    }

    // 3. Update last sync timestamp
    let now = Utc::now().to_rfc3339();
    set_metadata(conn, "last_sync_timestamp", &now)?;

    Ok(blobs.len())
}
```

### 4.5 Conflict Resolution

**LWW Rule:** The record with the higher `last_modified` timestamp wins. If timestamps are equal, the local version is preserved (no change).

**Conflict Logging:** When a remote version loses (its timestamp is lower than local), the losing version's content is stored in the `sync_conflicts` table. The user can:

1. **Accept remote** (discard local): overwrite local with remote content.
2. **Keep local** (discard remote): delete the conflict entry.
3. **Manual merge** (future): open a diff view and merge field-by-field.

For Sprint 5, only options 1 and 2 are implemented with a simple list UI.

---

## 5. Sync Server API

### 5.1 Endpoints

| Method | Path | Auth | Description |
|---|---|---|---|
| `POST` | `/v1/auth/register` | None | Create account. Body: `{ email, password_hash }`. Returns JWT. |
| `POST` | `/v1/auth/login` | None | Authenticate. Body: `{ email, password_hash }`. Returns JWT. |
| `POST` | `/v1/sync/push` | JWT | Upload encrypted blobs. Body: `[{ id, record_type, ciphertext, last_modified }]`. Returns `{ accepted: N }`. |
| `GET` | `/v1/sync/pull?since={iso8601}` | JWT | Download blobs since timestamp. Returns `[{ id, record_type, ciphertext, last_modified }]`. |

### 5.2 Data Model (Server)

```
Table: users
  id        UUID PRIMARY KEY
  email     TEXT UNIQUE NOT NULL
  password  TEXT NOT NULL        -- Argon2id hash
  created_at TIMESTAMP

Table: encrypted_blobs
  id             UUID PRIMARY KEY
  user_id        UUID NOT NULL REFERENCES users(id)
  record_id      TEXT NOT NULL   -- client-side highlight/stream ID
  record_type    TEXT NOT NULL   -- 'highlight' | 'stream' | 'subscription' | 'profile'
  ciphertext     TEXT NOT NULL   -- base64(nonce || ciphertext || auth_tag)
  last_modified  TEXT NOT NULL   -- RFC 3339 timestamp
  uploaded_at    TIMESTAMP DEFAULT NOW()
```

### 5.3 Security Rules

- **No plaintext storage.** Server never sees `text`, `summary`, `tags`, or any user content.
- **Server never stores encryption keys.** The encryption key is derived client-side and never transmitted.
- **Rate limiting:** 100 blobs per push, 1000 blobs fetchable per pull.
- **Blob size cap:** 64 KB per encrypted blob (reject larger payloads).
- **JWT expiry:** 7 days. Refresh tokens stored client-side.

---

## 6. Auth & Identity

### 6.1 Account Creation Flow

1. User clicks "Create Account" in Settings.
2. Frontend collects email + password + confirm password.
3. Password goes through client-side Argon2id with two salts:
   - `salt_auth`: used for server authentication hash
   - `salt_encrypt`: used for AES-256-GCM key derivation
4. `POST /v1/auth/register` with `{ email, password_hash: hash_auth }`.
5. On success, store locally:
   - `sync_credentials.user_email`
   - `sync_credentials.encryption_key_salt` (salt_encrypt)
   - `sync_credentials.server_url`
6. The encryption key is derived on-demand (never stored persistently in plaintext; derived each session).

### 6.2 Login Flow

1. User enters email + password.
2. Derive `hash_auth` from password + stored `salt_auth`.
3. `POST /v1/auth/login` → receive JWT.
4. Derive `encryption_key` from password + stored `salt_encrypt`.
5. JWT and encryption key held in memory (dropped on app quit).
6. Background sync begins automatically.

### 6.3 Tauri Integration

- Auth state stored in `std::sync::RwLock<Option<AuthState>>` managed by Tauri.
- Commands:
  - `create_account(email, password) -> Result<(), String>`
  - `log_in(email, password) -> Result<(), String>`
  - `log_out() -> Result<(), String>`
  - `get_auth_status() -> Result<AuthStatus, String>` (signed_in, email, server_url)
  - `sync_now() -> Result<SyncReport, String>` (push count, pull count, conflicts)

---

## 7. Conflict Resolution UI

### 7.1 Component: `SyncConflictPanel`

- Lists all unresolved conflicts from `sync_conflicts` table.
- Each conflict shows: record type, record ID, local version snippet, remote version snippet, timestamps.
- Actions: "Keep Local" or "Accept Remote."
- After resolution: mark conflict as resolved, apply chosen version to the live record.
- Under 200 lines.

### 7.2 Component: `SyncStatusBar`

- Visible in Settings panel.
- Shows:
  - Signed in as `{email}` (or "Not signed in").
  - Last synced: `{timestamp}` or "Never".
  - Sync status: "Synced", "Syncing...", "Offline", "Conflicts (N)".
  - "Sync now" button.
- Under 200 lines.

---

## 8. Mobile Framework Spike

### 8.1 Objective

Per PRD §12: build a single-screen capture app in both **Flutter** and **Kotlin Multiplatform (KMP)**. Measure and decide.

### 8.2 Measurements

| Metric | Target | Flutter | KMP |
|---|---|---|---|
| APK size (no model) | <20 MB | TBD | TBD |
| Cold start to capture-ready | <2 s | TBD | TBD |
| Cactus SDK integration time | <4 h | TBD | TBD |
| Model load time (EmbeddingGemma + Qwen 3.5) | <3 s | TBD | TBD |

### 8.3 Tiebreaker

**Flutter unless KMP shows >20% advantage on start time or app size.** This decision is locked by the end of Sprint 5.

### 8.4 Rust Core Library Extraction

Regardless of framework choice, the Rust core (AI pipeline, DB, sync engine) must be compiled as a shared library for mobile. This extraction begins in Sprint 5 but may not complete until Sprint 6.

---

## 9. File Structure

### 9.1 New Rust Modules

```
src-tauri/src/
  sync/
    mod.rs           -- Module declarations
    engine.rs        -- SyncEngine struct: push, pull, merge, background loop
    encrypt.rs       -- AES-256-GCM, Argon2id, key derivation
    server.rs        -- HTTP client for sync server API
    conflict.rs      -- Conflict detection, logging, resolution
  commands/
    auth.rs          -- create_account, log_in, log_out, get_auth_status
    sync.rs          -- sync_now, get_sync_status, get_conflicts, resolve_conflict
  db/
    migrations.rs    -- Schema migration runner (version 2 adds sync columns)
```

### 9.2 New Frontend Components

```
src/
  hooks/
    useAuth.ts          -- sign up, log in, log out, auth status
  components/
    SyncConflictPanel.tsx   -- Conflict list + resolution UI
    SyncStatusBar.tsx       -- Settings sub-panel for sync status
    AuthForm.tsx            -- Sign-up / log-in form in Settings
```

### 9.3 Sync Server (Separate Repository)

```
relay-sync-server/
  src/
    main.rs           -- Axum HTTP server
    auth.rs           -- Register, login, JWT middleware
    sync.rs           -- Push/pull handlers
    db.rs             -- SQLite for user table + encrypted blob table
  Cargo.toml
  .env.example
```

---

## 10. Exit Gates

| Gate | Requirement |
|---|---|
| `cargo test --workspace` | ≥110 tests (current 91 + ~20 sync/encrypt/auth/server tests) |
| `cargo clippy --all-targets -- -D warnings` | Clean |
| `pnpm test` | ≥85 tests (current 70 + ~15 auth/sync-UI tests) |
| **Sync protocol offline→online test** | Intentional timestamp mismatch → conflict logged → resolution works |
| **Encryption verification** | Wireshark/packet capture shows no plaintext in transit |
| **Schema migration test** | DB created in Sprint 4 schema → app launched with Sprint 5 code → migration adds sync columns without data loss |
| **Manual QA** | Create account on Device A → capture highlights → sync → Device B signs in → same highlights appear → subscribe to Stream → appears on both devices → conflict UI test |
| **Mobile spike complete** | Flutter vs KMP measurements recorded; framework decision locked |

---

## 11. Timeline (Sprints 5–8)

| Sprint | Focus | Key Deliverables |
|---|---|---|
| **5** | Sync engine + encryption | Rust sync module; AES-256-GCM + Argon2id; schema migration; sync server MVP |
| **6** | Auth + conflict UI | Account creation/login; JWT flow; SyncConflictPanel; SyncStatusBar |
| **7** | Mobile spike + clipboard watcher | Flutter vs KMP measurement; clipboard background watcher on desktop |
| **8** | Polish + launch prep | Integration testing; privacy policy; open-source repo public; beta test with 10 users |

---

## 12. Risk Register

| Risk | Severity | Mitigation |
|---|---|---|
| Encryption key loss = unrecoverable data | High | Document recovery limitations; Phase 2: add recovery phrase |
| Sync server unavailable = sync blocked | Medium | Sync is background-only, non-blocking; app functions fully offline |
| Schema migration breaks existing DB | High | Migration is idempotent (`ALTER TABLE ... ADD COLUMN IF NOT EXISTS` via `user_version` pragma). Full test on Sprint 4 schema snapshot. |
| Argon2id performance on weak devices | Medium | Test on 4GB RAM target. If too slow, fall back to bcrypt + PBKDF2. |
| Solo developer bottleneck on Rust engine | High | Frontend auth/conflict UI can be built in parallel with stubbed Rust APIs. Sync server can be built by a second agent. |
| Mobile spike inconclusive | Medium | Decision is locked by end of Sprint 7. If both fail, defer mobile to Phase 3. |

---

## 13. Dependencies

| Crate | Purpose | License |
|---|---|---|
| `aes-gcm = "0.10"` | AES-256-GCM encryption | MIT / Apache 2.0 |
| `argon2 = "0.5"` | Password key derivation | MIT / Apache 2.0 |
| `ring = "0.17"` | Cryptographic primitives | ISC (OpenSSL-compatible) |
| `reqwest = { features = ["json"] }` | HTTP client for sync server | MIT / Apache 2.0 |
| `axum = "0.7"` | Sync server HTTP framework | MIT |
| `jsonwebtoken = "9"` | JWT token handling | MIT |
| `chrono = { features = ["serde"] }` | RFC 3339 timestamps | MIT / Apache 2.0 |
| `base64 = "0.22"` | Encrypted blob encoding | MIT / Apache 2.0 |

---

*This specification is the irrevocable source of truth for Sprint 5. All prior discussions about sync scope, mobile timing, or encryption design are superseded by this document.*
