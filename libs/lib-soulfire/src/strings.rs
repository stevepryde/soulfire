//! Bounded-string newtypes and the `string_type!` macro.
//!
//! Reproduces Soulfire-OG's `string_type!` pattern: a tuple newtype over `String`
//! that enforces `(min, max)` length bounds (`DATA` "length-bounded strings").
//! `FromStr` trims and rejects out-of-range input (used on explicit save, `CHAR-5`/
//! `WORLD`); `coerce` clamps instead (used on AI-generated updates). Bounds are
//! contract values from `specs/01-data-model.md`.
//!
//! Length is measured in bytes (`str::len`), reproducing Soulfire-OG exactly so
//! prompt/behavior fidelity is preserved (`PROD-7`).

/// Define a bounded-string newtype with `(min, max)` byte-length bounds.
#[macro_export]
macro_rules! string_type {
    ($name:ident, $min_length:literal, $max_length:literal) => {
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            serde::Serialize,
            serde::Deserialize,
        )]
        pub struct $name(String);

        impl $name {
            /// Clamp `value` into range: pad up to the minimum, truncate to the
            /// maximum. Used for AI-generated values that must not be rejected.
            pub fn coerce(value: &str) -> $name {
                #[allow(unused_comparisons)]
                if value.len() < $min_length {
                    $name(".".repeat($min_length))
                } else if value.len() > $max_length {
                    $name(value[..$max_length].to_string())
                } else {
                    $name(value.to_string())
                }
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn into_string(self) -> String {
                self.0
            }

            pub fn min_length() -> usize {
                $min_length
            }

            pub fn max_length() -> usize {
                $max_length
            }
        }

        impl std::ops::Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let s = s.trim();

                #[allow(unused_comparisons)]
                if s.len() < $min_length {
                    return Err(anyhow::anyhow!(
                        "{} cannot be shorter than {} characters",
                        stringify!($name),
                        $min_length
                    ));
                }

                if s.len() > $max_length {
                    return Err(anyhow::anyhow!(
                        "{} cannot be longer than {} characters",
                        stringify!($name),
                        $max_length
                    ));
                }
                Ok(Self(s.to_string()))
            }
        }
    };
}

/// Give a `string_type!` an empty-string `Default` (bypasses validation, matching
/// Soulfire-OG; default values are placeholders filled before save).
#[macro_export]
macro_rules! impl_default_empty {
    ($name:ident) => {
        impl Default for $name {
            fn default() -> Self {
                $name(String::new())
            }
        }
    };
}

// ===== Characters (DATA-1, DATA-2) =====
string_type!(CharacterName, 1, 100);
impl_default_empty!(CharacterName);
string_type!(CharacterSubtitle, 0, 500);
impl_default_empty!(CharacterSubtitle);
string_type!(CharacterDescription, 0, 1000);
impl_default_empty!(CharacterDescription);
string_type!(CharacterPrompt, 0, 16000);
impl_default_empty!(CharacterPrompt);
// The initial-message string (prompt seed or verbatim opening line), DATA-2.
string_type!(InitialMessageText, 0, 16000);
impl_default_empty!(InitialMessageText);
// An AI-authored persona profile / dynamic state blob (DATA-3); generous bound.
string_type!(CharacterContext, 0, 50000);
impl_default_empty!(CharacterContext);

// ===== Chats & messages (DATA-5, DATA-6) =====
string_type!(ChatTitle, 0, 200);
impl_default_empty!(ChatTitle);
string_type!(MessageString, 0, 4096);
impl_default_empty!(MessageString);

// ===== Worlds: blueprints (DATA-8) =====
string_type!(WorldTitle, 1, 200);
impl_default_empty!(WorldTitle);
string_type!(WorldDescription, 0, 1000);
impl_default_empty!(WorldDescription);
string_type!(WorldPrompt, 1, 50000);
impl_default_empty!(WorldPrompt);

// ===== Worlds: adventure live state & memory (DATA-10, DATA-11) =====
string_type!(AdventureState, 0, 50000);
impl_default_empty!(AdventureState);
string_type!(RecentSummary, 0, 50000);
impl_default_empty!(RecentSummary);
string_type!(SignificantEvents, 0, 50000);
impl_default_empty!(SignificantEvents);
string_type!(StorySummary, 0, 50000);
impl_default_empty!(StorySummary);
// One adventure turn-log entry's content (DATA-12).
string_type!(MessageContent, 1, 10000);
impl_default_empty!(MessageContent);

// ===== Player profile (DATA-17) =====
string_type!(PlayerName, 0, 200);
impl_default_empty!(PlayerName);
string_type!(PlayerAttributes, 0, 5000);
impl_default_empty!(PlayerAttributes);
string_type!(PromptExtension, 0, 10000);
impl_default_empty!(PromptExtension);

// ===== App profile (DATA-16) =====
string_type!(DisplayName, 0, 100);
impl_default_empty!(DisplayName);

// ===== Drafts (DATA-26) =====
string_type!(DraftContent, 0, 10000);
impl_default_empty!(DraftContent);

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn from_str_trims_and_accepts_in_range() {
        // DATA: values are trimmed and accepted within bounds.
        let name = CharacterName::from_str("  Lyra  ").unwrap();
        assert_eq!(name.as_str(), "Lyra");
    }

    #[test]
    fn from_str_rejects_too_long() {
        let too_long = "x".repeat(101);
        assert!(CharacterName::from_str(&too_long).is_err());
    }

    #[test]
    fn from_str_rejects_below_min() {
        // CharacterName has min 1, so empty (after trim) is rejected.
        assert!(CharacterName::from_str("   ").is_err());
    }

    #[test]
    fn coerce_truncates_to_max() {
        let too_long = "x".repeat(200);
        let coerced = CharacterName::coerce(&too_long);
        assert_eq!(coerced.as_str().len(), 100);
    }

    #[test]
    fn coerce_pads_below_min() {
        let coerced = WorldTitle::coerce("");
        assert_eq!(coerced.as_str(), "."); // min 1
    }

    #[test]
    fn serializes_transparently_as_string() {
        let title = ChatTitle::from_str("A Quiet Evening").unwrap();
        let json = serde_json::to_string(&title).unwrap();
        assert_eq!(json, "\"A Quiet Evening\"");
    }
}
