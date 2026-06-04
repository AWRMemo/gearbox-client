use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use base64::Engine as _;
use ring::rand::{SecureRandom, SystemRandom};

/// Derive a 32-byte key from `password` and `salt` using Argon2id (default params).
pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    use argon2::{Algorithm, Argon2, Params, Version};
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::default());
    let mut out = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut out)
        .map_err(|e| format!("Key derivation failed: {e}"))?;
    Ok(out)
}

/// Encrypt `plaintext` with AES-256-GCM.
/// Returns base64(nonce || ciphertext).
pub fn encrypt(plaintext: &str, key: &[u8; 32]) -> Result<String, String> {
    let nonce_bytes = generate_nonce()?;
    let key_ref = Key::<Aes256Gcm>::from_slice(key.as_slice());
    let cipher = Aes256Gcm::new(key_ref);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {e}"))?;

    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(base64::engine::general_purpose::STANDARD.encode(&combined))
}

/// Decrypt a base64(nonce || ciphertext) blob.
pub fn decrypt(ciphertext_b64: &str, key: &[u8; 32]) -> Result<String, String> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(ciphertext_b64)
        .map_err(|e| format!("Base64 decode failed: {e}"))?;
    if decoded.len() < 12 {
        return Err("Ciphertext too short".to_string());
    }
    let (nonce_bytes, ciphertext_bytes) = decoded.split_at(12);
    let key_ref = Key::<Aes256Gcm>::from_slice(key.as_slice());
    let cipher = Aes256Gcm::new(key_ref);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext_bytes)
        .map_err(|e| format!("Decryption failed: {e}"))?;
    String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8: {e}"))
}

/// Generate a 16-byte random salt using `ring`. Returns an error if RNG fails.
pub fn generate_salt() -> Result<[u8; 16], String> {
    let rng = SystemRandom::new();
    let mut salt = [0u8; 16];
    rng.fill(&mut salt)
        .map_err(|e| format!("RNG fill failed: {e}"))?;
    Ok(salt)
}

/// Generate a 12-byte random nonce using `ring`. Returns an error if RNG fails.
pub fn generate_nonce() -> Result<[u8; 12], String> {
    let rng = SystemRandom::new();
    let mut nonce = [0u8; 12];
    rng.fill(&mut nonce)
        .map_err(|e| format!("RNG fill failed: {e}"))?;
    Ok(nonce)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let original = "Hello, secure sync world!";
        let encrypted = encrypt(original, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_key_derivation_is_deterministic() {
        let password = "my_secret_password";
        let salt = b"fixed_salt_here!!";
        let key1 = derive_key(password, salt).unwrap();
        let key2 = derive_key(password, salt).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_decrypt_fails_with_wrong_key() {
        let key = [42u8; 32];
        let wrong_key = [0u8; 32];
        let original = "Sensitive data";
        let encrypted = encrypt(original, &key).unwrap();
        let result = decrypt(&encrypted, &wrong_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_fails_with_tampered_ciphertext() {
        let key = [42u8; 32];
        let original = "Tamper test";
        let encrypted = encrypt(original, &key).unwrap();
        let mut bytes = base64::engine::general_purpose::STANDARD
            .decode(&encrypted)
            .unwrap();
        let last = bytes.len() - 1;
        bytes[last] = bytes[last].wrapping_add(1);
        let tampered_b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        assert!(decrypt(&tampered_b64, &key).is_err());
    }
}
