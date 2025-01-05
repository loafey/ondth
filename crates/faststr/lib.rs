//! A crate for a "faster" String.
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::Deref,
    sync::Arc,
};

/// A faster immutable String type. Simply wraps a [String] in an [Arc][std::sync::Arc].
/// to speed up cloning.
/// Also implemenents [Deserialize][serde::Deserialize] and [Serialize][serde::Serialize]
/// for any seralization needs.
#[derive(Clone, Default)]
pub struct FastStr {
    /// The inner string.
    inner: Arc<String>,
}

impl Deref for FastStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Serialize for FastStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for FastStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::from(String::deserialize(deserializer)?))
    }
}

impl PartialEq<&str> for FastStr {
    fn eq(&self, other: &&str) -> bool {
        self.inner.as_str() == *other
    }
}
impl PartialEq<String> for FastStr {
    fn eq(&self, other: &String) -> bool {
        self.inner.as_str() == other
    }
}
impl PartialEq<Self> for FastStr {
    fn eq(&self, other: &Self) -> bool {
        *self.inner == *other.inner
    }
}
impl Eq for FastStr {}
impl PartialOrd for FastStr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for FastStr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}
impl Hash for FastStr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(self.inner.as_bytes());
    }
}

impl Debug for FastStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &*self.inner)
    }
}

impl Display for FastStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &*self.inner)
    }
}

impl<'a> From<&'a str> for FastStr {
    fn from(value: &'a str) -> Self {
        Self {
            inner: Arc::new(value.to_string()),
        }
    }
}

impl From<String> for FastStr {
    fn from(value: String) -> Self {
        Self {
            inner: Arc::new(value),
        }
    }
}

impl AsRef<str> for FastStr {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disp_format() {
        let og_string = (0..255u8).map(|a| a as char).collect::<String>();
        let fstring = FastStr::from(og_string.clone());
        assert_eq!(format!("{og_string:?}"), format!("{fstring:?}"))
    }

    #[test]
    fn dbg_format() {
        let og_string = (0..255u8).map(|a| a as char).collect::<String>();
        let fstring = FastStr::from(og_string.clone());
        assert_eq!(format!("{og_string:?}"), format!("{fstring:?}"))
    }

    #[test]
    fn string() {
        let test = FastStr::from("jacoboi".to_string());
        drop(test);
    }

    #[test]
    fn static_str() {
        let test = FastStr::from("jacoboi");
        drop(test);
    }
}
