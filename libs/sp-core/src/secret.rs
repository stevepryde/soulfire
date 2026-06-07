use std::{cmp::Ordering, fmt::Debug};

use serde::{Deserialize, Serialize};

pub struct Secret<T>(T);

impl<T> Secret<T> {
    pub fn new(secret: T) -> Self {
        Self(secret)
    }

    pub fn expose(&self) -> &T {
        &self.0
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Secret<T> {
    fn from(secret: T) -> Self {
        Self(secret)
    }
}

impl<T> Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(&format!("Secret<{}>", std::any::type_name::<T>()))
            .field(&"**redacted**")
            .finish()
    }
}

impl<T> Clone for Secret<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Copy for Secret<T> where T: Copy {}

impl<T> Default for Secret<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> PartialEq for Secret<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> Eq for Secret<T> where T: Eq {}

impl<T> std::hash::Hash for Secret<T>
where
    T: std::hash::Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> PartialOrd for Secret<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Ord for Secret<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> Serialize for Secret<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'a, T> Deserialize<'a> for Secret<T>
where
    T: Deserialize<'a>,
{
    fn deserialize<D>(deserializer: D) -> Result<Secret<T>, D::Error>
    where
        D: serde::de::Deserializer<'a>,
    {
        let secret = T::deserialize(deserializer)?;
        Ok(Secret(secret))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret() {
        let secret = Secret::new("hello world");
        assert_eq!(secret.expose(), &"hello world");
        assert_eq!(secret.clone().expose(), &"hello world");
        assert_eq!(secret.into_inner(), "hello world");
    }
}
