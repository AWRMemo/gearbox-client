use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::sync::opaque_blob::OpaqueBlob;

/// An encrypted sync payload (v1).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncryptedBlob {
    pub id: String,
    pub record_type: String,
    pub ciphertext: String,
    pub last_modified: String,
}

/// Trait abstracting push/pull to the sync server.
/// Implemented by the real HTTP client and by test mocks.
pub trait SyncClient {
    fn push(&self, token: &str, blobs: &[EncryptedBlob]) -> Result<usize, String>;
    fn pull(&self, token: &str, since: &str) -> Result<Vec<EncryptedBlob>, String>;

    fn push_v2(&self, _token: &str, _blobs: &[OpaqueBlob]) -> Result<usize, String> {
        Err("v2 push not supported by this client".to_string())
    }
    fn pull_v2(&self, _token: &str, _since: &str) -> Result<Vec<OpaqueBlob>, String> {
        Err("v2 pull not supported by this client".to_string())
    }
}

/// Trait for server authentication operations used by command handlers.
/// Abstracts over the real HTTP client and test mocks.
pub trait AuthService {
    fn register(&self, email: &str, password_hash: &str) -> Result<String, String>;
    fn login(&self, email: &str, password_hash: &str) -> Result<String, String>;
}

/// Minimal JSON response from server auth endpoints.
#[derive(Deserialize)]
struct TokenRes {
    token: String,
}

/// Minimal sync client using `ureq` (blocking, no `reqwest` dependency in core).
pub struct SyncServerClient {
    base_url: String,
}

impl SyncServerClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    fn ensure_https_or_localhost(url: &str) -> Result<(), String> {
        let lower = url.to_lowercase();
        if lower.starts_with("https://")
            || lower.starts_with("http://localhost")
            || lower.starts_with("http://127.0.0.1")
        {
            Ok(())
        } else {
            Err("Sync server URL must use HTTPS for production. Set a secure server_url or use localhost for testing.".to_string())
        }
    }

    /// Register a new account and return a JWT string.
    pub fn register(&self, _email: &str, _password_hash: &str) -> Result<String, String> {
        Self::ensure_https_or_localhost(&self.base_url)?;
        let url = format!("{}/v1/auth/register", self.base_url);
        let body = serde_json::json!({
            "email": _email,
            "password_hash": _password_hash,
        });
        let resp = ureq::post(&url)
            .send_json(&body)
            .map_err(|e| format!("Register HTTP error: {e}"))?;
        if resp.status() < 200 || resp.status() >= 300 {
            return Err(format!("Register failed: HTTP {}", resp.status()));
        }
        let token = resp
            .into_json::<TokenRes>()
            .map_err(|e| format!("Failed to parse register response: {e}"))?
            .token;
        Ok(token)
    }

    /// Log in and return a JWT string.
    pub fn login(&self, _email: &str, _password_hash: &str) -> Result<String, String> {
        Self::ensure_https_or_localhost(&self.base_url)?;
        let url = format!("{}/v1/auth/login", self.base_url);
        let body = serde_json::json!({
            "email": _email,
            "password_hash": _password_hash,
        });
        let resp = ureq::post(&url)
            .send_json(&body)
            .map_err(|e| format!("Login HTTP error: {e}"))?;
        if resp.status() < 200 || resp.status() >= 300 {
            return Err(format!("Login failed: HTTP {}", resp.status()));
        }
        let token = resp
            .into_json::<TokenRes>()
            .map_err(|e| format!("Failed to parse login response: {e}"))?
            .token;
        Ok(token)
    }

    /// Push encrypted blobs to the server (v1). Returns number accepted.
    pub fn push(&self, token: &str, blobs: &[EncryptedBlob]) -> Result<usize, String> {
        let url = format!("{}/v1/sync/push", self.base_url);
        let resp = ureq::post(&url)
            .set("Authorization", &format!("Bearer {token}"))
            .send_json(blobs)
            .map_err(|e| format!("Push HTTP error: {e}"))?;
        if resp.status() < 200 || resp.status() >= 300 {
            return Err(format!("Push failed: HTTP {}", resp.status()));
        }
        let accepted: usize = resp
            .into_json()
            .map_err(|e| format!("Failed to parse push response: {e}"))?;
        Ok(accepted)
    }

    /// Pull blobs modified since `since` (ISO-8601 timestamp) (v1).
    pub fn pull(&self, token: &str, since: &str) -> Result<Vec<EncryptedBlob>, String> {
        let url = format!("{}/v1/sync/pull?since={}", self.base_url, since);
        let resp = ureq::get(&url)
            .set("Authorization", &format!("Bearer {token}"))
            .call()
            .map_err(|e| format!("Pull HTTP error: {e}"))?;
        if resp.status() < 200 || resp.status() >= 300 {
            return Err(format!("Pull failed: HTTP {}", resp.status()));
        }
        let blobs: Vec<EncryptedBlob> = resp
            .into_json()
            .map_err(|e| format!("Failed to parse pull response: {e}"))?;
        Ok(blobs)
    }

    /// Push opaque blobs to the server (v2). Returns number accepted.
    pub fn push_v2(&self, token: &str, blobs: &[OpaqueBlob]) -> Result<usize, String> {
        let url = format!("{}/v2/sync/blobs", self.base_url);
        let resp = ureq::post(&url)
            .set("Authorization", &format!("Bearer {token}"))
            .set("X-Relay-Protocol-Version", "2")
            .send_json(blobs)
            .map_err(|e| format!("Push v2 HTTP error: {e}"))?;
        if resp.status() < 200 || resp.status() >= 300 {
            return Err(format!("Push v2 failed: HTTP {}", resp.status()));
        }
        let accepted: usize = resp
            .into_json()
            .map_err(|e| format!("Failed to parse push v2 response: {e}"))?;
        Ok(accepted)
    }

    /// Pull opaque blobs modified since `since` (ISO-8601 timestamp) (v2).
    pub fn pull_v2(&self, token: &str, since: &str) -> Result<Vec<OpaqueBlob>, String> {
        let url = format!("{}/v2/sync/blobs?since={}", self.base_url, since);
        let resp = ureq::get(&url)
            .set("Authorization", &format!("Bearer {token}"))
            .set("X-Relay-Protocol-Version", "2")
            .call()
            .map_err(|e| format!("Pull v2 HTTP error: {e}"))?;
        if resp.status() < 200 || resp.status() >= 300 {
            return Err(format!("Pull v2 failed: HTTP {}", resp.status()));
        }
        let blobs: Vec<OpaqueBlob> = resp
            .into_json()
            .map_err(|e| format!("Failed to parse pull v2 response: {e}"))?;
        Ok(blobs)
    }
}

impl SyncClient for SyncServerClient {
    fn push(&self, token: &str, blobs: &[EncryptedBlob]) -> Result<usize, String> {
        self.push(token, blobs)
    }
    fn pull(&self, token: &str, since: &str) -> Result<Vec<EncryptedBlob>, String> {
        self.pull(token, since)
    }
    fn push_v2(&self, token: &str, blobs: &[OpaqueBlob]) -> Result<usize, String> {
        self.push_v2(token, blobs)
    }
    fn pull_v2(&self, token: &str, since: &str) -> Result<Vec<OpaqueBlob>, String> {
        self.pull_v2(token, since)
    }
}

impl AuthService for SyncServerClient {
    fn register(&self, email: &str, password_hash: &str) -> Result<String, String> {
        self.register(email, password_hash)
    }
    fn login(&self, email: &str, password_hash: &str) -> Result<String, String> {
        self.login(email, password_hash)
    }
}

impl AuthService for MockSyncServerClient {
    fn register(&self, _email: &str, _password_hash: &str) -> Result<String, String> {
        Ok("mock-jwt".to_string())
    }
    fn login(&self, _email: &str, _password_hash: &str) -> Result<String, String> {
        Ok("mock-jwt".to_string())
    }
}

/// Test-only mock server client backed by an in-memory store.
/// The v1 store keeps the *last pushed* blob per record id/type.
/// The v2 store keeps all opaque blobs keyed by blob_id.
#[derive(Default, Clone)]
pub struct MockSyncServerClient {
    store: Arc<Mutex<Vec<EncryptedBlob>>>,
    v2_store: Arc<Mutex<Vec<OpaqueBlob>>>,
}

impl MockSyncServerClient {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all stored data (useful for test isolation).
    pub fn clear(&self) {
        self.store.lock().unwrap().clear();
        self.v2_store.lock().unwrap().clear();
    }

    /// Test helper: inject pre-built v2 blobs directly into the v2 store.
    pub fn inject_v2_blobs(&self, blobs: Vec<OpaqueBlob>) {
        let mut store = self.v2_store.lock().unwrap();
        for blob in blobs {
            let idx = store.iter().position(|b| b.blob_id == blob.blob_id);
            if let Some(i) = idx {
                store[i] = blob;
            } else {
                store.push(blob);
            }
        }
    }
}

impl SyncClient for MockSyncServerClient {
    fn push(&self, _token: &str, blobs: &[EncryptedBlob]) -> Result<usize, String> {
        let mut store = self.store.lock().unwrap();
        for blob in blobs {
            let idx = store
                .iter()
                .position(|b| b.id == blob.id && b.record_type == blob.record_type);
            if let Some(i) = idx {
                store[i] = blob.clone();
            } else {
                store.push(blob.clone());
            }
        }
        Ok(blobs.len())
    }

    fn pull(&self, _token: &str, since: &str) -> Result<Vec<EncryptedBlob>, String> {
        let store = self.store.lock().unwrap();
        let mut out: Vec<EncryptedBlob> = store
            .iter()
            .filter(|b| b.last_modified.as_str() > since)
            .cloned()
            .collect();
        out.sort_by(|a, b| a.last_modified.cmp(&b.last_modified));
        Ok(out)
    }

    fn push_v2(&self, _token: &str, blobs: &[OpaqueBlob]) -> Result<usize, String> {
        let mut store = self.v2_store.lock().unwrap();
        for blob in blobs {
            let idx = store.iter().position(|b| b.blob_id == blob.blob_id);
            if let Some(i) = idx {
                store[i] = blob.clone();
            } else {
                store.push(blob.clone());
            }
        }
        Ok(blobs.len())
    }

    fn pull_v2(&self, _token: &str, _since: &str) -> Result<Vec<OpaqueBlob>, String> {
        // The outer OpaqueBlob has no last_modified, so the mock cannot
        // meaningfully filter by `since` without decrypting. Tests rely on
        // client-side LWW to ignore stale data, so we return the whole store.
        let store = self.v2_store.lock().unwrap();
        Ok(store.clone())
    }
}
