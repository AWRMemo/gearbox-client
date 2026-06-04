//! FRB FFI surface — annotated bridge over relay_core.

use super::types::*;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use flutter_rust_bridge::frb;
use relay_core::ai::fallback::FallbackService;
use relay_core::ai::service::AIService;
use relay_core::config;
use relay_core::db;
use relay_core::sync::conflict::{list_conflicts, resolve_conflict_with_action};
use relay_core::sync::engine::SyncEngine;
use relay_core::sync::server::SyncServerClient;
use relay_core::types::*;
use std::sync::Mutex;

#[derive(Clone)]
struct MobileAuthState {
    jwt: String,
    encryption_key: [u8; 32],
    server_url: String,
}

static AUTH_STATE: Mutex<Option<MobileAuthState>> = Mutex::new(None);

fn get_mobile_auth() -> Result<MobileAuthState, String> {
    let guard = AUTH_STATE
        .lock()
        .map_err(|e| format!("Auth lock poisoned: {e}"))?;
    guard.clone().ok_or_else(|| "Not authenticated".to_string())
}

#[frb(sync)]
pub fn init_core(data_dir: String) -> Result<(), String> {
    let path = std::path::PathBuf::from(data_dir);
    db::set_data_dir(path.clone());
    config::init(&path)?;
    Ok(())
}

#[frb(sync)]
pub fn store_highlight(
    text: String,
    summary: String,
    tags: Vec<String>,
    source_url: Option<String>,
    source_title: Option<String>,
    source_author: Option<String>,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let highlight = Highlight {
        id: id.clone(),
        text,
        source_url,
        source_title,
        source_author,
    };
    let enrichment = EnrichmentOutput {
        summary,
        tags,
        connection_suggestion: None,
    };
    db::store::store_highlight(&highlight, &enrichment, None)?;
    Ok(id)
}

#[frb(sync)]
pub fn enrich_and_store(text: String) -> Result<EnrichResult, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let highlight = Highlight {
        id: id.clone(),
        text: text.clone(),
        source_url: None,
        source_title: None,
        source_author: None,
    };
    let service = FallbackService;
    let output = service
        .enrich(&highlight, &[])
        .map_err(|e| format!("Fallback enrichment failed: {e}"))?;

    db::store::store_highlight(&highlight, &output, None)?;

    Ok(EnrichResult {
        id,
        summary: output.summary,
        tags: output.tags,
        suggestion_highlight_id: output
            .connection_suggestion
            .as_ref()
            .map(|c| c.source_highlight_id.clone()),
        suggestion_bridging_sentence: output
            .connection_suggestion
            .as_ref()
            .map(|c| c.bridging_sentence.clone()),
        source_url: None,
        source_title: None,
        source_author: None,
    })
}

#[frb(sync)]
pub fn search_highlights(query: String, limit: i64) -> Result<Vec<SearchResultResponse>, String> {
    let results = db::search::search_highlights(&query, limit as usize)?;
    Ok(results
        .into_iter()
        .map(|r| SearchResultResponse {
            id: r.id,
            summary: r.summary,
            tags: r.tags,
            text: r.text,
            score: r.score,
        })
        .collect())
}

#[frb(sync)]
pub fn list_stored_highlights(
    limit: i64,
    offset: i64,
) -> Result<Vec<ListedHighlightResponse>, String> {
    let results = db::store::list_highlights(limit as usize, offset as usize)?;
    Ok(results
        .into_iter()
        .map(|r| ListedHighlightResponse {
            id: r.id,
            text: r.text,
            summary: r.summary,
            tags: r.tags,
            created_at: r.created_at,
            source_url: r.source_url,
            source_title: r.source_title,
            source_author: r.source_author,
            connection_suggestion: r.connection_suggestion,
        })
        .collect())
}

#[frb(sync)]
pub fn delete_highlight(id: String) -> Result<(), String> {
    let conn = db::open_db()?;
    conn.execute("DELETE FROM highlights WHERE id = ?1", [&id])
        .map_err(|e| format!("Failed to delete highlight: {e}"))?;
    conn.execute("DELETE FROM highlights_fts WHERE id = ?1", [&id])
        .map_err(|e| format!("Failed to delete FTS entry: {e}"))?;
    Ok(())
}

#[frb(sync)]
pub fn log_event(
    event_type: String,
    stream_id: Option<String>,
    channel: Option<String>,
) -> Result<(), String> {
    let user_id = config::get_device_id()?;
    db::analytics::log_event(
        &event_type,
        stream_id.as_deref(),
        Some(user_id),
        None,
        channel.as_deref(),
    )
}

#[frb(sync)]
pub fn register_device_token(token: String, platform: String) -> Result<(), String> {
    let conn = db::open_db()?;
    conn.execute(
        "INSERT INTO device_tokens (token, platform, created_at, last_seen_at) VALUES (?1, ?2, datetime('now'), datetime('now'))
         ON CONFLICT(token) DO UPDATE SET last_seen_at = datetime('now')",
        [&token, &platform],
    )
    .map_err(|e| format!("Failed to store token: {e}"))?;
    Ok(())
}

#[frb(sync)]
pub fn create_stream(title: String, description: String) -> Result<String, String> {
    let user_id = config::get_device_id()?;
    let info = db::stream::create_stream(user_id, &title, &description)?;
    Ok(info.id)
}

#[frb(sync)]
pub fn add_highlight_to_stream(stream_id: String, highlight_id: String) -> Result<(), String> {
    db::stream::add_highlight_to_stream(&stream_id, &highlight_id)
}

#[frb(sync)]
pub fn get_stream(stream_id: String) -> Result<StreamInfoResponse, String> {
    let info = db::stream::get_stream_by_id(&stream_id)?;
    Ok(StreamInfoResponse {
        id: info.id,
        user_id: info.user_id,
        title: info.title,
        description: info.description,
    })
}

#[frb(sync)]
pub fn list_my_streams() -> Result<Vec<StreamInfoResponse>, String> {
    let user_id = config::get_device_id()?;
    let streams = db::stream::list_user_streams(user_id)?;
    Ok(streams
        .into_iter()
        .map(|s| StreamInfoResponse {
            id: s.id,
            user_id: s.user_id,
            title: s.title,
            description: s.description,
        })
        .collect())
}

#[frb(sync)]
pub fn subscribe_to_stream(stream_id: String) -> Result<(), String> {
    let user_id = config::get_device_id()?;
    db::subscriptions::subscribe(user_id, &stream_id)
}

#[frb(sync)]
pub fn get_subscriber_feed(limit: i64, offset: i64) -> Result<Vec<FeedHighlightResponse>, String> {
    let user_id = config::get_device_id()?;
    let feed = db::subscriptions::get_subscriber_feed(user_id, limit as usize, offset as usize)?;
    Ok(feed
        .into_iter()
        .map(|f| FeedHighlightResponse {
            id: f.id,
            text: f.text,
            summary: f.summary,
            tags: f.tags,
            stream_title: f.stream_title,
        })
        .collect())
}

#[frb(sync)]
pub fn create_account(email: String, password: String) -> Result<AuthResultResponse, String> {
    let conn = db::open_db()?;

    // check for existing user with same email
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM users WHERE email = ?1",
            rusqlite::params![&email],
            |row| row.get(0),
        )
        .ok();
    if existing.is_some() {
        return Err("Account already exists for this email.".to_string());
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Password hashing failed: {e}"))?
        .to_string();

    let user_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO users (id, email, password_hash) VALUES (?1, ?2, ?3)",
        rusqlite::params![&user_id, &email, &password_hash],
    )
    .map_err(|e| format!("Failed to create user: {e}"))?;

    // start session
    conn.execute(
        "INSERT INTO current_session (token, user_id, email) VALUES (?1, ?2, ?3)",
        rusqlite::params!["", &user_id, &email],
    )
    .map_err(|e| format!("Failed to start session: {e}"))?;

    Ok(AuthResultResponse {
        token: String::new(),
        user_id,
    })
}

#[frb(sync)]
pub fn log_in(email: String, password: String) -> Result<AuthResultResponse, String> {
    let conn = db::open_db()?;

    let (user_id, stored_hash): (String, String) = conn
        .query_row(
            "SELECT id, password_hash FROM users WHERE email = ?1",
            rusqlite::params![&email],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("No account found for {email}: {e}"))?;

    let parsed_hash =
        PasswordHash::new(&stored_hash).map_err(|e| format!("Invalid stored hash: {e}"))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| "Invalid password.".to_string())?;

    // start session
    conn.execute(
        "INSERT INTO current_session (token, user_id, email) VALUES (?1, ?2, ?3)",
        rusqlite::params!["", &user_id, &email],
    )
    .map_err(|e| format!("Failed to start session: {e}"))?;

    Ok(AuthResultResponse {
        token: String::new(),
        user_id,
    })
}

#[frb(sync)]
pub fn log_out() -> Result<(), String> {
    let conn = db::open_db()?;
    conn.execute("DELETE FROM current_session", [])
        .map_err(|e| format!("Failed to log out: {e}"))?;
    Ok(())
}

#[frb(sync)]
pub fn get_auth_status() -> Result<AuthStatusResponse, String> {
    let conn = db::open_db()?;
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT token, email FROM current_session LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();
    match row {
        Some((_token, email)) => Ok(AuthStatusResponse {
            logged_in: true,
            email: Some(email),
        }),
        None => Ok(AuthStatusResponse {
            logged_in: false,
            email: None,
        }),
    }
}

#[frb(sync)]
pub fn sync_now() -> Result<SyncReportResponse, String> {
    let auth = get_mobile_auth()?;
    let client = std::sync::Arc::new(SyncServerClient::new(auth.server_url));
    let engine = SyncEngine::new(client, auth.jwt, auth.encryption_key);
    let report = engine.sync_now().map_err(|e| e.to_string())?;
    Ok(SyncReportResponse {
        pushed: report.pushed as i64,
        pulled: report.pulled as i64,
        conflicts: report.conflicts as i64,
    })
}

#[frb(sync)]
pub fn get_sync_status() -> Result<SyncStatusResponse, String> {
    let conn = db::open_db()?;

    let last_sync: Option<String> = conn
        .query_row(
            "SELECT value FROM sync_metadata WHERE key = 'last_sync_timestamp'",
            [],
            |row| row.get(0),
        )
        .ok();

    let pending_highlights: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM highlights WHERE sync_status = 'local'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);
    let pending_streams: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM streams WHERE sync_status = 'local'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);
    let pending_sh: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM stream_highlights WHERE sync_status = 'local'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);
    let pending_subs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM subscriptions WHERE sync_status = 'local'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);
    let pending_profile: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM user_profile WHERE sync_status = 'local'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);

    let pending_count =
        pending_highlights + pending_streams + pending_sh + pending_subs + pending_profile;

    let status = if last_sync.is_some() {
        "active".to_string()
    } else {
        "never".to_string()
    };

    Ok(SyncStatusResponse {
        status,
        pending_count,
    })
}

#[frb(sync)]
pub fn get_conflicts() -> Result<Vec<ConflictResponse>, String> {
    let conn = db::open_db()?;
    let conflicts = list_conflicts(&conn)?;
    Ok(conflicts
        .into_iter()
        .map(|c| ConflictResponse {
            id: c.id,
            record_type: c.record_type,
            record_id: c.record_id,
            local_version: c.local_version,
            remote_version: c.remote_version,
            resolved_at: c.resolved_at,
            resolution: c.resolution,
            created_at: c.created_at,
        })
        .collect())
}

#[frb(sync)]
pub fn resolve_conflict(id: String, resolution: String) -> Result<(), String> {
    let conn = db::open_db()?;
    resolve_conflict_with_action(&conn, &id, &resolution).map_err(|e| e.to_string())?;
    Ok(())
}

#[frb(sync)]
pub fn export_local_data() -> Result<String, String> {
    Ok(String::new())
}

#[frb(sync)]
pub fn clear_local_data() -> Result<(), String> {
    Ok(())
}

#[frb(sync)]
pub fn get_telemetry_opt_out() -> Result<bool, String> {
    Ok(relay_core::telemetry::is_opted_out())
}

#[frb(sync)]
pub fn set_telemetry_opt_out(opt_out: bool) -> Result<(), String> {
    relay_core::telemetry::set_opt_out(opt_out)
}

#[frb(sync)]
pub fn get_auto_capture_enabled() -> Result<bool, String> {
    Ok(false)
}

#[frb(sync)]
pub fn set_auto_capture_enabled(_enabled: bool) -> Result<(), String> {
    Ok(())
}

#[frb(sync)]
pub fn get_model_status() -> Result<ModelStatusResponse, String> {
    Ok(ModelStatusResponse {
        loaded: false,
        model_name: "Fallback".to_string(),
        backend: "deterministic".to_string(),
    })
}

#[frb(sync)]
pub fn extract_source_metadata(text: String) -> Result<SourceMetaResponse, String> {
    let lines: Vec<&str> = text.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
            let source_url = Some(trimmed.to_string());
            let source_title = if i > 0 {
                let prev = lines[i - 1].trim();
                if !prev.is_empty() && !prev.starts_with("http") {
                    Some(prev.to_string())
                } else {
                    None
                }
            } else {
                None
            };
            return Ok(SourceMetaResponse {
                source_url,
                source_title,
                source_author: None,
            });
        }
    }
    Ok(SourceMetaResponse {
        source_url: None,
        source_title: None,
        source_author: None,
    })
}

#[derive(Debug, Clone)]
pub struct EnrichResult {
    pub id: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub suggestion_highlight_id: Option<String>,
    pub suggestion_bridging_sentence: Option<String>,
    pub source_url: Option<String>,
    pub source_title: Option<String>,
    pub source_author: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(
            crate::api::simple::greet("Relay".to_string()),
            "Hello, Relay!"
        );
    }

    #[test]
    fn test_extract_source_metadata_finds_url() {
        let text = "Example Article\nhttps://example.com/article".to_string();
        let meta = extract_source_metadata(text).unwrap();
        assert_eq!(
            meta.source_url,
            Some("https://example.com/article".to_string())
        );
        assert_eq!(meta.source_title, Some("Example Article".to_string()));
    }

    #[test]
    fn test_extract_source_metadata_no_url() {
        let text = "Just some plain text without any links".to_string();
        let meta = extract_source_metadata(text).unwrap();
        assert_eq!(meta.source_url, None);
        assert_eq!(meta.source_title, None);
    }

    #[test]
    fn test_extract_source_metadata_url_first_line() {
        let text = "https://example.com/article\nSome body".to_string();
        let meta = extract_source_metadata(text).unwrap();
        assert_eq!(
            meta.source_url,
            Some("https://example.com/article".to_string())
        );
        assert_eq!(meta.source_title, None);
    }

    #[test]
    fn test_enrich_result_structure() {
        let r = EnrichResult {
            id: "test-id".to_string(),
            summary: "A summary".to_string(),
            tags: vec!["tag1".to_string()],
            suggestion_highlight_id: None,
            suggestion_bridging_sentence: None,
            source_url: Some("https://example.com".to_string()),
            source_title: None,
            source_author: None,
        };
        assert_eq!(r.id, "test-id");
        assert_eq!(r.tags.len(), 1);
    }

    #[test]
    fn test_source_meta_response_default() {
        let m = SourceMetaResponse {
            source_url: None,
            source_title: None,
            source_author: None,
        };
        assert!(m.source_url.is_none());
        assert!(m.source_title.is_none());
        assert!(m.source_author.is_none());
    }
}
