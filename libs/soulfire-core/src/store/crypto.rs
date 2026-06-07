//! Key derivation and the unlock verifier (`SEC-2`, `SEC-4`, `SEC-6`).
//!
//! The master password is stretched through Argon2id into a 32-byte key that is
//! used directly as the SQLCipher raw key. The KDF salt and parameters, plus a
//! one-way verifier of the derived key, live in a small **plaintext sidecar**
//! file next to the encrypted database — they must be readable *before* the
//! database can be opened (you need them to derive the key). The salt is not
//! secret; the verifier is a one-way hash that lets a wrong password be rejected
//! without touching the encrypted data (`SEC-6`). Both KDF params and the
//! verifier scheme are versioned so they can be strengthened by a future
//! migration (`SEC` design notes, `PKG-4`).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::error::{CoreError, CoreResult};

/// Length of the derived key / SQLCipher raw key, in bytes.
pub const KEY_LEN: usize = 32;
/// Length of the KDF salt, in bytes.
pub const SALT_LEN: usize = 16;

/// Current sidecar meta format version (`PKG-4`).
pub const META_FORMAT_VERSION: u32 = 1;
/// Current KDF parameter-set version (`SEC` design notes).
pub const KDF_VERSION: u32 = 1;

/// Domain-separation tag mixed into the verifier so it can never collide with any
/// other use of the derived key.
const VERIFIER_DOMAIN: &[u8] = b"soulfire-unlock-verifier-v1";

/// A derived 32-byte key, zeroized on drop (`SEC-9`).
pub type DerivedKey = Zeroizing<[u8; KEY_LEN]>;

/// Argon2id key-derivation parameters (`SEC-2`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KdfParams {
    pub version: u32,
    /// Salt, hex-encoded (not secret).
    pub salt_hex: String,
    /// Memory cost in KiB.
    pub m_cost: u32,
    /// Iteration (time) cost.
    pub t_cost: u32,
    /// Degree of parallelism.
    pub p_cost: u32,
}

impl KdfParams {
    /// Generate fresh parameters with a random salt and strong defaults.
    pub fn generate() -> CoreResult<KdfParams> {
        let mut salt = [0u8; SALT_LEN];
        getrandom::fill(&mut salt)
            .map_err(|e| CoreError::Crypto(format!("salt generation failed: {e}")))?;
        Ok(KdfParams {
            version: KDF_VERSION,
            salt_hex: hex::encode(salt),
            // OWASP-strong defaults: 64 MiB memory, 3 iterations, 1 lane.
            m_cost: 65_536,
            t_cost: 3,
            p_cost: 1,
        })
    }

    fn salt(&self) -> CoreResult<Vec<u8>> {
        hex::decode(&self.salt_hex)
            .map_err(|e| CoreError::Crypto(format!("invalid salt encoding: {e}")))
    }
}

/// Derive the 32-byte key from `password` and `params` using Argon2id (`SEC-2`).
pub fn derive_key(password: &str, params: &KdfParams) -> CoreResult<DerivedKey> {
    use argon2::{Algorithm, Argon2, Params, Version};

    let salt = params.salt()?;
    let argon_params = Params::new(params.m_cost, params.t_cost, params.p_cost, Some(KEY_LEN))
        .map_err(|e| CoreError::Crypto(format!("invalid argon2 params: {e}")))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);

    let mut key: DerivedKey = Zeroizing::new([0u8; KEY_LEN]);
    argon2
        .hash_password_into(password.as_bytes(), &salt, key.as_mut())
        .map_err(|e| CoreError::Crypto(format!("key derivation failed: {e}")))?;
    Ok(key)
}

/// Compute the one-way verifier of a derived key (`SEC-6`).
pub fn compute_verifier(key: &[u8; KEY_LEN]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(VERIFIER_DOMAIN);
    hasher.update(key);
    hex::encode(hasher.finalize())
}

/// Constant-time-ish check that `key` matches `verifier_hex`. (Both sides are
/// fixed-length hex of a SHA-256 digest.)
pub fn verify_key(key: &[u8; KEY_LEN], verifier_hex: &str) -> bool {
    let computed = compute_verifier(key);
    // Length-equal hex strings; compare with a simple constant-time fold.
    if computed.len() != verifier_hex.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in computed.bytes().zip(verifier_hex.bytes()) {
        diff |= a ^ b;
    }
    diff == 0
}

/// The plaintext sidecar describing how to derive and verify the store key
/// (`SEC-4`). Contains no secrets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreMeta {
    pub format_version: u32,
    pub kdf: KdfParams,
    /// One-way verifier of the derived key (`SEC-6`).
    pub verifier_hex: String,
}

impl StoreMeta {
    /// Build meta for a freshly chosen password: generate KDF params, derive the
    /// key, and record its verifier. Returns the meta and the derived key.
    pub fn create(password: &str) -> CoreResult<(StoreMeta, DerivedKey)> {
        let kdf = KdfParams::generate()?;
        let key = derive_key(password, &kdf)?;
        let verifier_hex = compute_verifier(&key);
        Ok((
            StoreMeta {
                format_version: META_FORMAT_VERSION,
                kdf,
                verifier_hex,
            },
            key,
        ))
    }

    /// Derive the key from `password` and verify it against this meta. Returns the
    /// key on success, or [`CoreError::WrongPassword`] (`SEC-6`).
    pub fn unlock(&self, password: &str) -> CoreResult<DerivedKey> {
        let key = derive_key(password, &self.kdf)?;
        if verify_key(&key, &self.verifier_hex) {
            Ok(key)
        } else {
            Err(CoreError::WrongPassword)
        }
    }

    /// Re-key to a new password: generate fresh KDF params and verifier (`SEC-8`).
    /// Returns the new meta and the new derived key.
    pub fn rekey(new_password: &str) -> CoreResult<(StoreMeta, DerivedKey)> {
        StoreMeta::create(new_password)
    }

    pub fn to_json(&self) -> CoreResult<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    pub fn from_json(s: &str) -> CoreResult<StoreMeta> {
        serde_json::from_str(s).map_err(Into::into)
    }
}

/// The SQLCipher raw-key pragma value for a derived key: `x'<hex>'`.
pub fn sqlcipher_key_literal(key: &[u8; KEY_LEN]) -> String {
    format!("x'{}'", hex::encode(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_then_unlock_round_trips() {
        let (meta, key) = StoreMeta::create("correct horse battery staple").unwrap();
        let key2 = meta.unlock("correct horse battery staple").unwrap();
        assert_eq!(&*key, &*key2);
    }

    #[test]
    fn wrong_password_is_rejected() {
        let (meta, _key) = StoreMeta::create("right-password").unwrap();
        let err = meta.unlock("wrong-password").unwrap_err();
        assert!(matches!(err, CoreError::WrongPassword));
    }

    #[test]
    fn rekey_changes_salt_and_verifier() {
        let (meta1, _k1) = StoreMeta::create("pw1").unwrap();
        let (meta2, _k2) = StoreMeta::rekey("pw2").unwrap();
        assert_ne!(meta1.kdf.salt_hex, meta2.kdf.salt_hex);
        assert_ne!(meta1.verifier_hex, meta2.verifier_hex);
    }

    #[test]
    fn meta_json_round_trips() {
        let (meta, _key) = StoreMeta::create("pw").unwrap();
        let json = meta.to_json().unwrap();
        let back = StoreMeta::from_json(&json).unwrap();
        assert_eq!(meta, back);
    }

    #[test]
    fn sqlcipher_literal_is_hex_wrapped() {
        let key = [0xABu8; KEY_LEN];
        let lit = sqlcipher_key_literal(&key);
        assert!(lit.starts_with("x'") && lit.ends_with('\''));
        assert_eq!(lit.len(), 2 + KEY_LEN * 2 + 1);
    }
}
