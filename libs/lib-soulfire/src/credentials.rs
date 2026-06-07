//! Provider API-key credentials (`DATA-19`, `SEC-9`, `SEC-10`).
//!
//! Stored only inside the encrypted store. The key value is wrapped in
//! [`sp_core::secret::Secret`] so it never appears in `Debug`/log output, and is
//! shown only masked through the UI (`SEC-10`).

use serde::{Deserialize, Serialize};

use sp_core::secret::Secret;

use crate::ai_model::AiVendor;

/// A stored provider credential: the provider and its secret key value
/// (`DATA-19`).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCredential {
    pub provider: AiVendor,
    /// The secret API key. Never logged or shown in full (`SEC-9`, `SEC-10`).
    pub key: Secret<String>,
}

impl ProviderCredential {
    pub fn new(provider: AiVendor, key: impl Into<String>) -> Self {
        ProviderCredential {
            provider,
            key: Secret::new(key.into()),
        }
    }

    /// A masked rendering for display: the last few characters only (`SEC-10`).
    /// Returns a fixed mask when the key is too short to reveal any tail safely.
    pub fn masked(&self) -> String {
        mask_key(self.key.expose())
    }
}

impl std::fmt::Debug for ProviderCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never render the key, even in Debug (SEC-9).
        f.debug_struct("ProviderCredential")
            .field("provider", &self.provider)
            .field("key", &"**redacted**")
            .finish()
    }
}

/// Mask an API key, revealing only its last 4 characters (`SEC-10`).
pub fn mask_key(key: &str) -> String {
    let visible = 4;
    let chars: Vec<char> = key.chars().collect();
    if chars.len() <= visible {
        return "•".repeat(chars.len().max(1));
    }
    let tail: String = chars[chars.len() - visible..].iter().collect();
    format!("{}{}", "•".repeat(8), tail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_reveal_only_tail() {
        let c = ProviderCredential::new(AiVendor::OpenAI, "sk-secret-abcd1234");
        let masked = c.masked();
        assert!(masked.ends_with("1234"));
        assert!(!masked.contains("secret"));
    }

    #[test]
    fn debug_never_leaks_key() {
        let c = ProviderCredential::new(AiVendor::OpenAI, "sk-supersecret");
        let dbg = format!("{c:?}");
        assert!(!dbg.contains("supersecret"));
        assert!(dbg.contains("redacted"));
    }

    #[test]
    fn serializes_key_value_for_encrypted_store() {
        // The whole store is encrypted at rest, so the key serializes plainly
        // here; SEC-1/SEC-9 guarantee it is unreadable on disk.
        let c = ProviderCredential::new(AiVendor::OpenAI, "sk-abc");
        let json = serde_json::to_string(&c).unwrap();
        let back: ProviderCredential = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }
}
