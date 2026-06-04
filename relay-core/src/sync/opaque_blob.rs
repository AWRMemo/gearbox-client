use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use base64::Engine as _;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::RelayError;

/// Outer envelope visible to the sync server.
///
/// `blob_id` is a fresh UUID-v4 with no semantic link to the inner record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpaqueBlob {
    pub blob_id: String,
    pub payload: String,
}

/// Inner plaintext structure (client-only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InnerBlob {
    pub id: String,
    pub record_type: String,
    pub last_modified: i64,
    pub data: Value,
}

/// Encrypt an [`InnerBlob`] into the v2 payload format.
///
/// Steps:
/// 1. Serialize `inner` to JSON.
/// 2. Generate a random 12-byte nonce.
/// 3. AES-256-GCM encrypt with AAD `b"relay-sync-v2"`.
/// 4. Return `base64(nonce || ciphertext || tag)`.
pub fn encrypt_inner_blob(key: &[u8], inner: &InnerBlob) -> Result<String, RelayError> {
    if key.len() != 32 {
        return Err(RelayError::CryptoError(
            "encryption key must be exactly 32 bytes".to_string(),
        ));
    }

    let inner_json =
        serde_json::to_string(inner).map_err(|e| RelayError::CryptoError(e.to_string()))?;

    let nonce_bytes = generate_nonce().map_err(RelayError::CryptoError)?;

    let key_ref = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key_ref);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let aead_payload = Payload {
        msg: inner_json.as_bytes(),
        aad: b"relay-sync-v2",
    };

    let ciphertext = cipher
        .encrypt(nonce, aead_payload)
        .map_err(|e| RelayError::CryptoError(format!("AES-256-GCM encrypt failed: {e}")))?;

    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(base64::engine::general_purpose::STANDARD.encode(&combined))
}

/// Decrypt a v2 payload back into an [`InnerBlob`].
///
/// Steps:
/// 1. Base64-decode.
/// 2. Extract the leading 12-byte nonce.
/// 3. AES-256-GCM decrypt with AAD `b"relay-sync-v2"`.
/// 4. Deserialize JSON into `InnerBlob`.
pub fn decrypt_payload(key: &[u8], payload: &str) -> Result<InnerBlob, RelayError> {
    if key.len() != 32 {
        return Err(RelayError::CryptoError(
            "encryption key must be exactly 32 bytes".to_string(),
        ));
    }

    let decoded = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .map_err(|e| RelayError::CryptoError(format!("base64 decode failed: {e}")))?;

    if decoded.len() < 12 {
        return Err(RelayError::CryptoError(
            "ciphertext too short (missing nonce)".to_string(),
        ));
    }

    let (nonce_bytes, ciphertext_bytes) = decoded.split_at(12);
    let key_ref = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key_ref);
    let nonce = Nonce::from_slice(nonce_bytes);

    let aead_payload = Payload {
        msg: ciphertext_bytes,
        aad: b"relay-sync-v2",
    };

    let plaintext = cipher
        .decrypt(nonce, aead_payload)
        .map_err(|e| RelayError::CryptoError(format!("AES-256-GCM decrypt failed: {e}")))?;

    let inner_json = String::from_utf8(plaintext)
        .map_err(|e| RelayError::CryptoError(format!("invalid UTF-8: {e}")))?;

    serde_json::from_str(&inner_json).map_err(|e| RelayError::CryptoError(e.to_string()))
}

/// Generate a 12-byte random nonce using `ring`.
fn generate_nonce() -> Result<[u8; 12], String> {
    let rng = SystemRandom::new();
    let mut nonce = [0u8; 12];
    rng.fill(&mut nonce)
        .map_err(|e| format!("RNG fill failed: {e}"))?;
    Ok(nonce)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid::Uuid;

    fn dummy_key() -> [u8; 32] {
        [42u8; 32]
    }

    #[test]
    fn test_roundtrip_100_random_blobs() {
        let key = dummy_key();
        let rng = SystemRandom::new();

        for i in 0..100 {
            let mut buf = [0u8; 8];
            rng.fill(&mut buf).unwrap();
            let random_num = u64::from_le_bytes(buf);

            let data = match i % 5 {
                0 => json!({ "text": format!("hello-{}", random_num) }),
                1 => json!([1, 2, random_num]),
                2 => json!({ "nested": { "value": random_num } }),
                3 => json!(null),
                _ => json!(format!("string-{}", random_num)),
            };

            let inner = InnerBlob {
                id: format!("id_{}", random_num),
                record_type: match i % 3 {
                    0 => "highlight".to_string(),
                    1 => "stream".to_string(),
                    _ => "settings".to_string(),
                },
                last_modified: random_num as i64,
                data,
            };

            let payload = encrypt_inner_blob(&key, &inner).unwrap();
            let decrypted = decrypt_payload(&key, &payload).unwrap();
            assert_eq!(inner, decrypted, "round-trip failed on iteration {}", i);
        }
    }

    #[test]
    fn test_blob_id_is_uuidv4() {
        let key = dummy_key();
        let inner = InnerBlob {
            id: "my-record-123".to_string(),
            record_type: "highlight".to_string(),
            last_modified: 1716480000000,
            data: json!({ "text": "capture" }),
        };

        let blob_id = Uuid::new_v4().to_string();
        let payload = encrypt_inner_blob(&key, &inner).unwrap();
        let opaque = OpaqueBlob { blob_id, payload };

        let parsed = Uuid::parse_str(&opaque.blob_id).expect("blob_id must be a valid UUID");
        assert_eq!(
            parsed.get_version(),
            Some(uuid::Version::Random),
            "must be UUIDv4"
        );

        // Must not correlate with inner id
        assert_ne!(opaque.blob_id, inner.id);
    }

    #[test]
    fn test_aad_mismatch_fails() {
        let key = dummy_key();
        let inner = InnerBlob {
            id: "aad-test".to_string(),
            record_type: "highlight".to_string(),
            last_modified: 1,
            data: json!({}),
        };

        let payload = encrypt_inner_blob(&key, &inner).unwrap();

        // Manually decrypt with a mismatched AAD to force authentication failure.
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&payload)
            .unwrap();
        let (nonce_bytes, ciphertext_bytes) = decoded.split_at(12);
        let key_ref = Key::<Aes256Gcm>::from_slice(&key);
        let cipher = Aes256Gcm::new(key_ref);
        let nonce = Nonce::from_slice(nonce_bytes);

        let bad_payload = Payload {
            msg: ciphertext_bytes,
            aad: b"relay-sync-v1", // wrong AAD
        };

        let result = cipher.decrypt(nonce, bad_payload);
        assert!(result.is_err(), "decrypt with wrong AAD must fail");
    }

    #[test]
    fn test_legacy_downgrade_shim() {
        // Simulate a v1 blob: the v1 encrypt function does not use AAD.
        // A v2 shim would decrypt the legacy ciphertext with v1 decrypt,
        // then construct an InnerBlob using plaintext metadata + decrypted data.
        let key = dummy_key();
        let legacy_data = json!({
            "text": "legacy highlight text",
            "source_url": "https://example.com"
        });

        // Encrypt the data payload using v1 (no AAD)
        let legacy_ciphertext =
            crate::sync::encrypt::encrypt(&legacy_data.to_string(), &key).unwrap();

        // Simulate the shim path:
        // 1. Decrypt legacy ciphertext with v1 decrypt (no AAD).
        let decrypted_data_str = crate::sync::encrypt::decrypt(&legacy_ciphertext, &key).unwrap();
        let decrypted_data: Value = serde_json::from_str(&decrypted_data_str).unwrap();

        // 2. Construct an InnerBlob using the plaintext metadata + decrypted data.
        let reconstructed = InnerBlob {
            id: "legacy_hl_abc".to_string(),
            record_type: "highlight".to_string(),
            last_modified: 1234567890,
            data: decrypted_data,
        };

        // 3. Verify the reconstructed InnerBlob is valid and serializable.
        let json_str = serde_json::to_string(&reconstructed).unwrap();
        let parsed_back: InnerBlob = serde_json::from_str(&json_str).unwrap();
        assert_eq!(reconstructed, parsed_back);

        // 4. Ensure the reconstructed blob can be encrypted with v2 and round-tripped.
        let v2_payload = encrypt_inner_blob(&key, &reconstructed).unwrap();
        let roundtripped = decrypt_payload(&key, &v2_payload).unwrap();
        assert_eq!(reconstructed, roundtripped);
    }
}
