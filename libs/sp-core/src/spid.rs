use std::{
    fmt::{Debug, Display, Formatter},
    hash::Hash,
    str::FromStr,
};

#[derive(Debug, thiserror::Error)]
pub enum SpIdError {
    #[error("invalid id")]
    InvalidId,
}

pub trait HasPrefix {
    const PREFIX: &'static str;
}

pub struct SpId<T> {
    id: uuid::Uuid,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: HasPrefix> Default for SpId<T> {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: HasPrefix> SpId<T> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: HasPrefix> FromStr for SpId<T> {
    type Err = SpIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw = s
            .strip_prefix(&format!("{}_", T::PREFIX))
            .ok_or(SpIdError::InvalidId)?;
        uuid::Uuid::parse_str(raw)
            .map(|id| Self {
                id,
                _phantom: std::marker::PhantomData,
            })
            .map_err(|_| SpIdError::InvalidId)
    }
}

impl<T: HasPrefix> Display for SpId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}", T::PREFIX, self.id)
    }
}

// #[derive(
//     Debug,
//     Clone,
//     Copy,
//     PartialEq,
//     Eq,
//     Hash,
//     PartialOrd,
//     Ord,
//     serde_with::SerializeDisplay,
//     serde_with::DeserializeFromStr,
// )]

impl<T: HasPrefix> Debug for SpId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({}_{})",
            std::any::type_name::<Self>(),
            T::PREFIX,
            self.id
        )
    }
}

impl<T> Clone for SpId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for SpId<T> {}

impl<T> PartialEq for SpId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl<T> Eq for SpId<T> {}

impl<T: HasPrefix> Hash for SpId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        format!("{}_", T::PREFIX).hash(state);
        self.id.hash(state);
    }
}

impl<T: HasPrefix> PartialOrd for SpId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: HasPrefix> Ord for SpId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl<T: HasPrefix> serde::Serialize for SpId<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl<'de, T: HasPrefix> serde::Deserialize<'de> for SpId<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[macro_export]
macro_rules! id_type {
    ($name:ident, $prefix:literal) => {
        $crate::paste::paste! {
            pub struct [<$name Spec>];
            impl $crate::spid::HasPrefix for [<$name Spec>] {
                const PREFIX: &'static str = $prefix;
            }
            pub type $name = $crate::spid::SpId<[<$name Spec>]>;
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPrefix;
    impl HasPrefix for TestPrefix {
        const PREFIX: &'static str = "test";
    }
    type MyId = SpId<TestPrefix>;

    #[test]
    fn test_spid_roundtrip() {
        let id = MyId::default();
        let id_str = id.to_string();
        let id_parsed = id_str.parse::<MyId>().unwrap();
        assert_eq!(id, id_parsed);
    }

    #[test]
    fn test_spid_serde() {
        let id = MyId::default();
        let id_str = serde_json::to_string(&id).unwrap();
        let id_parsed = serde_json::from_str(&id_str).unwrap();
        assert_eq!(id, id_parsed);
    }

    #[test]
    fn test_spid_different_prefix() {
        struct OtherPrefix;
        impl HasPrefix for OtherPrefix {
            const PREFIX: &'static str = "other";
        }
        type OtherId = SpId<OtherPrefix>;

        let id = OtherId::default();
        let id_str = id.to_string();
        assert!(id_str.starts_with("other"));
    }

    #[test]
    fn test_spid_invalid_parse() {
        let invalid_id_str = "invalid";
        let parsed = invalid_id_str.parse::<MyId>();
        assert!(parsed.is_err());
    }

    #[test]
    fn test_spid_invalid_serde() {
        let invalid_id_str = "\"invalid\"";
        let parsed: Result<MyId, _> = serde_json::from_str(invalid_id_str);
        assert!(parsed.is_err());
    }
}
