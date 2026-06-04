use crate::db::open_db;
use crate::sync::encrypt::{derive_key, generate_salt};
use crate::sync::server::AuthService;
use base64::Engine;
use serde::Serialize;
use std::sync::{Arc, RwLock};

/// Global authentication state managed by Tauri.
#[derive(Clone, Debug)]
pub struct AuthState {
    pub email: String,
    pub jwt: String,
    pub encryption_key: [u8; 32],
    pub server_url: String,
}

#[derive(Serialize)]
pub struct AuthStatus {
    pub signed_in: bool,
    pub email: Option<String>,
    pub server_url: Option<String>,
}

/// Helper: get default server URL from DB or fallback.
fn get_default_server_url(conn: &rusqlite::Connection) -> String {
    conn.query_row(
        "SELECT server_url FROM sync_credentials LIMIT 1",
        [],
        |row| row.get::<_, String>(0),
    )
    .unwrap_or_else(|_| "https://relay-sync.gearbox.local/v1".to_string())
}

/// Generate password hash from password + salt using Argon2id.
fn derive_password_hash(password: &str, salt: &[u8]) -> Result<String, String> {
    let key = derive_key(password, salt)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(key))
}

/// Internal implementation accepting any `AuthService` so tests can inject mocks.
fn create_account_impl<S: AuthService>(
    email: String,
    password: String,
    state: &Arc<RwLock<Option<AuthState>>>,
    client: &S,
) -> Result<(), String> {
    let conn = open_db()?;

    let salt_auth = generate_salt()?;
    let salt_encrypt = generate_salt()?;
    let password_hash = derive_password_hash(&password, &salt_auth)?;

    let server_url = get_default_server_url(&conn);
    let jwt = client.register(&email, &password_hash)?;

    conn.execute(
        "INSERT INTO sync_credentials (user_email, password_hash, salt_auth, encryption_key_salt, server_url, protocol_version)
         VALUES (?1, ?2, ?3, ?4, ?5, 2)
         ON CONFLICT(user_email) DO UPDATE SET
             password_hash = excluded.password_hash,
             salt_auth = excluded.salt_auth,
             encryption_key_salt = excluded.encryption_key_salt,
             server_url = excluded.server_url,
             protocol_version = 2",
        rusqlite::params![
            email,
            password_hash,
            base64::engine::general_purpose::STANDARD.encode(salt_auth),
            base64::engine::general_purpose::STANDARD.encode(salt_encrypt),
            server_url,
        ],
    )
    .map_err(|e| format!("Failed to store credentials: {e}"))?;

    // Save JWT back to DB so login can rehydrate later
    conn.execute(
        "INSERT INTO sync_metadata (key, value) VALUES ('jwt', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![jwt],
    )
    .map_err(|e| format!("Failed to store JWT: {e}"))?;

    // Derive encryption key and set in-memory auth state
    let encryption_key = derive_key(&password, &salt_encrypt)?;
    let auth = AuthState {
        email: email.clone(),
        jwt: jwt.clone(),
        encryption_key,
        server_url: server_url.clone(),
    };

    if let Ok(mut guard) = state.write() {
        *guard = Some(auth);
    } else {
        return Err("Failed to write auth state".to_string());
    }

    // Trigger initial sync in background so registration remains fast
    let state_clone = Arc::clone(state);
    std::thread::spawn(move || {
        if let Ok(guard) = state_clone.read() {
            if let Some(ref auth) = *guard {
                let client = Arc::new(crate::sync::server::SyncServerClient::new(
                    auth.server_url.clone(),
                ));
                let engine = crate::sync::engine::SyncEngine::new(
                    client,
                    auth.jwt.clone(),
                    auth.encryption_key,
                );
                if let Err(e) = engine.sync_now() {
                    eprintln!("[auth] Initial sync after registration failed (non-critical): {e}");
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub fn create_account(
    email: String,
    password: String,
    state: tauri::State<Arc<RwLock<Option<AuthState>>>>,
) -> Result<(), String> {
    let server_url = {
        let conn = open_db()?;
        get_default_server_url(&conn)
    };
    let client = crate::sync::server::SyncServerClient::new(server_url);
    create_account_impl(email, password, state.inner(), &client)
}

fn log_in_impl<S: AuthService>(
    email: String,
    password: String,
    state: &Arc<RwLock<Option<AuthState>>>,
    client: &S,
) -> Result<(), String> {
    let conn = open_db()?;

    let row: (String, String, String) = conn
        .query_row(
            "SELECT password_hash, salt_auth, encryption_key_salt FROM sync_credentials WHERE user_email = ?1",
            rusqlite::params![&email],
            |row| {
                let ph: String = row.get(0)?;
                let sa: String = row.get(1)?;
                let se: String = row.get(2)?;
                Ok((ph, sa, se))
            },
        )
        .map_err(|e| format!("No stored credentials for {email}: {e}"))?;

    let (stored_hash_b64, salt_auth_b64, salt_encrypt_b64) = row;
    let salt_auth = base64::engine::general_purpose::STANDARD
        .decode(&salt_auth_b64)
        .map_err(|e| format!("Invalid stored salt_auth: {e}"))?;
    let salt_encrypt = base64::engine::general_purpose::STANDARD
        .decode(&salt_encrypt_b64)
        .map_err(|e| format!("Invalid stored encryption_key_salt: {e}"))?;

    let derived_hash = derive_password_hash(&password, &salt_auth)?;
    if derived_hash != stored_hash_b64 {
        return Err("Invalid password.".to_string());
    }

    let server_url = get_default_server_url(&conn);
    let jwt = client.login(&email, &derived_hash)?;

    let encryption_key = derive_key(&password, &salt_encrypt)?;

    let auth = AuthState {
        email: email.clone(),
        jwt,
        encryption_key,
        server_url: server_url.clone(),
    };

    if let Ok(mut guard) = state.write() {
        *guard = Some(auth);
    } else {
        return Err("Failed to write auth state".to_string());
    }

    conn.execute(
        "INSERT INTO sync_metadata (key, value) VALUES ('last_email', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![email],
    )
    .map_err(|e| format!("Failed to store last email: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn log_in(
    email: String,
    password: String,
    state: tauri::State<Arc<RwLock<Option<AuthState>>>>,
) -> Result<(), String> {
    let server_url = {
        let conn = open_db()?;
        get_default_server_url(&conn)
    };
    let client = crate::sync::server::SyncServerClient::new(server_url);
    log_in_impl(email, password, state.inner(), &client)
}

fn get_auth_status_impl(state: &Arc<RwLock<Option<AuthState>>>) -> Result<AuthStatus, String> {
    {
        if let Ok(guard) = state.read() {
            if let Some(ref auth) = *guard {
                return Ok(AuthStatus {
                    signed_in: true,
                    email: Some(auth.email.clone()),
                    server_url: Some(auth.server_url.clone()),
                });
            }
        }
    }

    // Fallback: is there an email in sync_credentials?
    let conn = open_db()?;
    let email: Option<String> = conn
        .query_row(
            "SELECT user_email FROM sync_credentials LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();
    let server_url: Option<String> = conn
        .query_row(
            "SELECT server_url FROM sync_credentials LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    Ok(AuthStatus {
        signed_in: false,
        email,
        server_url,
    })
}

#[tauri::command]
pub fn get_auth_status(
    state: tauri::State<Arc<RwLock<Option<AuthState>>>>,
) -> Result<AuthStatus, String> {
    get_auth_status_impl(state.inner())
}

#[tauri::command]
pub fn log_out(state: tauri::State<Arc<RwLock<Option<AuthState>>>>) -> Result<(), String> {
    if let Ok(mut guard) = state.write() {
        *guard = None;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use relay_core::db::init_test_pool;
    use std::sync::Mutex;

    #[allow(dead_code)]
    static SERIAL: Mutex<()> = Mutex::new(());

    struct DummyAuthService;

    impl AuthService for DummyAuthService {
        fn register(&self, _email: &str, _password_hash: &str) -> Result<String, String> {
            Ok("dummy-jwt".to_string())
        }
        fn login(&self, _email: &str, _password_hash: &str) -> Result<String, String> {
            Ok("dummy-jwt".to_string())
        }
    }

    fn setup() -> (std::path::PathBuf, Arc<RwLock<Option<AuthState>>>) {
        let _guard = SERIAL.lock().unwrap();
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("relay_auth_test_{}_{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        init_test_pool(&dir);
        (dir, Arc::new(RwLock::new(None)))
    }

    #[test]
    fn test_create_account_derives_key_and_sets_state() {
        let (_dir, state) = setup();
        let result = create_account_impl(
            "test@example.com".to_string(),
            "password123".to_string(),
            &state,
            &DummyAuthService,
        );
        assert!(
            result.is_ok(),
            "create_account_impl should succeed: {:?}",
            result
        );

        let guard = state.read().unwrap();
        assert!(
            guard.is_some(),
            "auth state should be set after registration"
        );
        let auth = guard.as_ref().unwrap();
        assert_eq!(auth.email, "test@example.com");
        assert_eq!(auth.jwt, "dummy-jwt");
        assert!(
            !auth.encryption_key.iter().all(|&b| b == 0),
            "encryption_key must be derived (non-zero)"
        );
    }

    #[test]
    fn test_create_account_stores_credentials_in_db() {
        let (_dir, state) = setup();
        let result = create_account_impl(
            "db-test@example.com".to_string(),
            "password123".to_string(),
            &state,
            &DummyAuthService,
        );
        assert!(result.is_ok());

        let conn = open_db().unwrap();
        let (email, server_url): (String, String) = conn
            .query_row(
                "SELECT user_email, server_url FROM sync_credentials LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(email, "db-test@example.com");
        assert_eq!(&server_url, "https://relay-sync.gearbox.local/v1");
    }

    #[test]
    fn test_log_in_validates_password() {
        let (_dir, state) = setup();
        // Register first (uses create_account_impl which stores credentials but no in-memory state for second call)
        create_account_impl(
            "login-test@example.com".to_string(),
            "correct_password".to_string(),
            &state,
            &DummyAuthService,
        )
        .unwrap();

        // Log in with correct password
        let result = log_in_impl(
            "login-test@example.com".to_string(),
            "correct_password".to_string(),
            &state,
            &DummyAuthService,
        );
        assert!(
            result.is_ok(),
            "log_in with correct password should succeed: {:?}",
            result
        );

        // Verify state written by log_in
        let guard = state.read().unwrap();
        assert!(guard.is_some());
        assert_eq!(guard.as_ref().unwrap().email, "login-test@example.com");
    }

    #[test]
    fn test_log_in_rejects_wrong_password() {
        let (_dir, state) = setup();
        create_account_impl(
            "login-wrong@example.com".to_string(),
            "correct_password".to_string(),
            &state,
            &DummyAuthService,
        )
        .unwrap();

        let result = log_in_impl(
            "login-wrong@example.com".to_string(),
            "wrong_password".to_string(),
            &state,
            &DummyAuthService,
        );
        assert!(result.is_err(), "log_in with wrong password should fail");
        let err = result.unwrap_err();
        assert!(
            err.contains("Invalid password") || err.contains("Invalid"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_get_auth_status_reflects_state() {
        let (_dir, state) = setup();
        let status = get_auth_status_impl(&state).unwrap();
        assert!(!status.signed_in);
        assert!(status.email.is_none());

        create_account_impl(
            "status-test@example.com".to_string(),
            "password123".to_string(),
            &state,
            &DummyAuthService,
        )
        .unwrap();

        let status = get_auth_status_impl(&state).unwrap();
        assert!(status.signed_in);
        assert_eq!(status.email.as_deref(), Some("status-test@example.com"));
    }
}
