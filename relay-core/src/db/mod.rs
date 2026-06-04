pub mod account;
pub mod analytics;
pub mod creator;
pub mod deferred_enrichment;
pub mod founding_curator;
pub mod migrations;
pub mod search;
pub mod store;
pub mod stream;
pub mod subscriptions;
pub mod tiers;
pub mod vector;

pub use crate::types::{
    FeedHighlight, ListedHighlight, SearchResult, StreamHighlight, StreamInfo, UserProfile,
};

use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;

use r2d2::ManageConnection;

// ── Schema ──────────────────────────────────────────────────────────────────

/// Shared CREATE TABLE block for the pool manager and `open_test_db()`.
pub(crate) const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS highlights (
    id TEXT PRIMARY KEY,
    text TEXT NOT NULL,
    source_url TEXT,
    source_title TEXT,
    source_author TEXT,
    summary TEXT NOT NULL,
    tags TEXT NOT NULL,
    connection_suggestion TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_modified TEXT NOT NULL DEFAULT (datetime('now')),
    sync_status TEXT NOT NULL DEFAULT 'local'
);

CREATE VIRTUAL TABLE IF NOT EXISTS highlights_fts USING fts5(
    id,
    text,
    summary,
    tags_text
);

CREATE TABLE IF NOT EXISTS device_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    platform TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_seen_at TEXT NOT NULL DEFAULT (datetime('now')),
    is_active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS streams (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT DEFAULT '',
    is_public INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_modified TEXT NOT NULL DEFAULT (datetime('now')),
    sync_status TEXT NOT NULL DEFAULT 'local'
);

CREATE TABLE IF NOT EXISTS stream_highlights (
    stream_id TEXT NOT NULL REFERENCES streams(id) ON DELETE CASCADE,
    highlight_id TEXT NOT NULL REFERENCES highlights(id) ON DELETE CASCADE,
    added_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_modified TEXT NOT NULL DEFAULT (datetime('now')),
    sync_status TEXT NOT NULL DEFAULT 'local',
    PRIMARY KEY (stream_id, highlight_id)
);

CREATE TABLE IF NOT EXISTS analytics_events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    stream_id TEXT,
    curator_user_id TEXT,
    visitor_anonymous_id TEXT,
    channel TEXT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_analytics_event_type ON analytics_events(event_type);
CREATE INDEX IF NOT EXISTS idx_analytics_timestamp ON analytics_events(timestamp);

CREATE TABLE IF NOT EXISTS subscriptions (
    user_id TEXT NOT NULL,
    stream_id TEXT NOT NULL,
    subscribed_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_modified TEXT NOT NULL DEFAULT (datetime('now')),
    sync_status TEXT NOT NULL DEFAULT 'local',
    PRIMARY KEY (user_id, stream_id)
);

CREATE TABLE IF NOT EXISTS user_profile (
    user_id TEXT PRIMARY KEY,
    email TEXT,
    display_name TEXT,
    tier TEXT NOT NULL DEFAULT 'free',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_modified TEXT NOT NULL DEFAULT (datetime('now')),
    sync_status TEXT NOT NULL DEFAULT 'local'
);

CREATE TABLE IF NOT EXISTS sync_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sync_conflicts (
    id TEXT PRIMARY KEY,
    record_type TEXT NOT NULL,
    record_id TEXT NOT NULL,
    local_version TEXT,
    remote_version TEXT,
    resolved_at TEXT,
    resolution TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sync_credentials (
    user_email TEXT PRIMARY KEY,
    password_hash TEXT NOT NULL,
    salt_auth TEXT NOT NULL,
    encryption_key_salt TEXT NOT NULL,
    server_url TEXT NOT NULL,
    protocol_version INTEGER NOT NULL DEFAULT 2,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS current_session (
    token TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    email TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS enrichment_quality_log (
    capture_id TEXT PRIMARY KEY,
    parse_success INTEGER NOT NULL,
    tag_count INTEGER NOT NULL,
    summary_length INTEGER NOT NULL,
    model_name TEXT NOT NULL DEFAULT '',
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_quality_timestamp ON enrichment_quality_log(timestamp);

CREATE TABLE IF NOT EXISTS review_log (
    highlight_id TEXT PRIMARY KEY,
    ease_factor REAL NOT NULL DEFAULT 2.5,
    interval_days INTEGER NOT NULL DEFAULT 1,
    next_review_at TEXT NOT NULL DEFAULT (datetime('now')),
    review_count INTEGER NOT NULL DEFAULT 0,
    last_grade INTEGER,
    reviewed_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (highlight_id) REFERENCES highlights(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS creator_profiles (
    user_id TEXT PRIMARY KEY,
    stripe_connect_account_id TEXT,
    is_verified INTEGER NOT NULL DEFAULT 0,
    display_name TEXT,
    bio TEXT,
    platform_fee_percent INTEGER NOT NULL DEFAULT 10,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_modified TEXT NOT NULL DEFAULT (datetime('now')),
    sync_status TEXT NOT NULL DEFAULT 'local'
);

CREATE TABLE IF NOT EXISTS monetized_streams (
    stream_id TEXT PRIMARY KEY REFERENCES streams(id),
    creator_id TEXT NOT NULL REFERENCES creator_profiles(user_id),
    monthly_price_cents INTEGER NOT NULL DEFAULT 200,
    subscriber_count INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_modified TEXT NOT NULL DEFAULT (datetime('now')),
    sync_status TEXT NOT NULL DEFAULT 'local'
);

CREATE INDEX IF NOT EXISTS idx_creator_verified ON creator_profiles(is_verified);

"#;

// ── r2d2 connection manager ──────────────────────────────────────────────────

/// [`ManageConnection`] implementation backed by `rusqlite`.
/// Creates each connection with WAL mode, FK checks, and the shared schema.
pub struct SqliteConnectionManager {
    path: String,
}

impl SqliteConnectionManager {
    pub fn new(path: &str) -> Self {
        SqliteConnectionManager {
            path: path.to_string(),
        }
    }
}

impl ManageConnection for SqliteConnectionManager {
    type Connection = rusqlite::Connection;
    type Error = rusqlite::Error;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let conn = rusqlite::Connection::open(&self.path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;",
        )?;
        conn.execute_batch(SCHEMA_SQL)?;
        Ok(conn)
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        conn.execute_batch("SELECT 1;")?;
        Ok(())
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        conn.execute("SELECT 1", []).is_err()
    }
}

// ── Global pool ─────────────────────────────────────────────────────────────

/// Legacy directory tracker — retained so that `src-tauri` tests can still
/// manipulate `DB_DIR` directly.  The runtime path is derived from this mutex.
pub static DB_DIR: Mutex<Option<&'static str>> = Mutex::new(None);

static DB_POOL: LazyLock<Mutex<Option<r2d2::Pool<SqliteConnectionManager>>>> =
    LazyLock::new(|| Mutex::new(None));

/// Set the database directory path.
/// Called once during app initialization. Safe to call multiple times (overwrites).
pub fn set_data_dir(path: PathBuf) {
    let dir = path.to_string_lossy().to_string();
    let leaked: &'static str = Box::leak(dir.into_boxed_str());
    let mut guard = DB_DIR.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(leaked);

    let db_path = format!("{}/relay.db", leaked);
    let manager = SqliteConnectionManager::new(&db_path);
    let pool = r2d2::Pool::builder()
        .max_size(5)
        .build(manager)
        .expect("Failed to build SQLite connection pool");

    let mut pool_guard = DB_POOL.lock().unwrap_or_else(|e| e.into_inner());
    *pool_guard = Some(pool);

    vector::init_vector_store(&path).expect("Failed to initialise LanceDB vector store");
}

fn get_data_dir() -> &'static str {
    let guard = DB_DIR.lock().unwrap_or_else(|e| e.into_inner());
    guard.expect("db::set_data_dir was not called before database access")
}

pub fn ensure_data_dir() -> Result<(), String> {
    let dir = get_data_dir();
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create data directory: {e}"))
}

pub fn pool_ready() -> bool {
    let guard = DB_POOL.lock().unwrap_or_else(|e| e.into_inner());
    guard.is_some()
}

/// Open a pooled connection to the SQLite database.
///
/// Callers do **not** need to know about `r2d2`; `PooledConnection`
/// derefs to `rusqlite::Connection`, so existing `conn.execute(...)` code
/// continues to compile unchanged.
pub fn open_db() -> Result<r2d2::PooledConnection<SqliteConnectionManager>, String> {
    let guard = DB_POOL.lock().unwrap_or_else(|e| e.into_inner());
    let pool = guard
        .as_ref()
        .expect("db::set_data_dir was not called before database access");
    pool.get()
        .map_err(|e| format!("Failed to get database connection from pool: {e}"))
}

/// Run `PRAGMA wal_checkpoint(TRUNCATE)` via the pool.
/// Exposed so the app can periodically reclaim WAL disk space.
pub fn checkpoint_wal() -> Result<(), String> {
    let conn = open_db()?;
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|e| format!("Failed to checkpoint WAL: {e}"))
}

// ── Test helpers ────────────────────────────────────────────────────────────

#[cfg(any(test, feature = "test-utils"))]
pub fn init_test_pool(data_dir: &std::path::Path) {
    let dir = data_dir.to_string_lossy().to_string();
    let leaked: &'static str = Box::leak(dir.into_boxed_str());
    let mut guard = DB_DIR.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(leaked);

    let db_path = format!("{}/relay.db", leaked);
    let manager = SqliteConnectionManager::new(&db_path);
    let pool = r2d2::Pool::builder()
        .max_size(5)
        .build(manager)
        .expect("Failed to build SQLite connection pool");

    let mut pool_guard = DB_POOL.lock().unwrap_or_else(|e| e.into_inner());
    *pool_guard = Some(pool);
}

#[cfg(any(test, feature = "test-utils"))]
pub fn open_test_db() -> Result<rusqlite::Connection, String> {
    let conn =
        rusqlite::Connection::open_in_memory().map_err(|e| format!("Failed to open DB: {e}"))?;
    conn.execute_batch(SCHEMA_SQL)
        .map_err(|e| format!("Failed to create test tables: {e}"))?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EnrichmentOutput, Highlight};

    fn make_highlight(id: &str, text: &str) -> Highlight {
        Highlight {
            id: id.to_string(),
            text: text.to_string(),
            source_url: None,
            source_title: None,
            source_author: None,
        }
    }

    fn make_enrichment() -> EnrichmentOutput {
        EnrichmentOutput {
            summary: "Test summary.".to_string(),
            tags: vec!["test".to_string(), "highlight".to_string()],
            connection_suggestion: None,
        }
    }

    #[test]
    fn test_store_and_search_roundtrip() {
        let conn = open_test_db().unwrap();

        let highlight = make_highlight("test-1", "The quick brown fox jumps over the lazy dog.");
        let enrichment = make_enrichment();

        let tags_json = serde_json::to_string(&enrichment.tags).unwrap();
        let tags_text = enrichment.tags.join(" ");

        conn.execute(
            "INSERT INTO highlights (id, text, summary, tags) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![highlight.id, highlight.text, enrichment.summary, tags_json],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO highlights_fts (id, text, summary, tags_text) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![highlight.id, highlight.text, enrichment.summary, tags_text],
        )
        .unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT h.id
                 FROM highlights_fts
                 JOIN highlights h ON h.id = highlights_fts.id
                 WHERE highlights_fts MATCH 'fox'
                 LIMIT 10",
            )
            .unwrap();

        let rows: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0], "test-1");
    }

    #[test]
    fn test_store_empty_tags() {
        let conn = open_test_db().unwrap();
        let highlight = make_highlight("test-2", "Some text.");
        let enrichment = EnrichmentOutput {
            summary: "Some text.".to_string(),
            tags: vec![],
            connection_suggestion: None,
        };

        let tags_json = serde_json::to_string(&enrichment.tags).unwrap();
        let tags_text = enrichment.tags.join(" ");

        conn.execute(
            "INSERT INTO highlights (id, text, summary, tags) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![highlight.id, highlight.text, enrichment.summary, tags_json],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO highlights_fts (id, text, summary, tags_text) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![highlight.id, highlight.text, enrichment.summary, tags_text],
        )
        .unwrap();

        let mut stmt = conn
            .prepare("SELECT tags FROM highlights WHERE id = ?1")
            .unwrap();
        let tags: String = stmt
            .query_row(rusqlite::params!["test-2"], |row| row.get(0))
            .unwrap();
        assert_eq!(tags, "[]");
    }
}
