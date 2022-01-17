use std::collections::BTreeMap;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A representation for a OSM tags key
#[derive(Clone)]
pub enum TagKey {
    Static(&'static str),
    String(String),
}

impl TagKey {
    pub const fn from(string: &'static str) -> Self {
        TagKey::Static(string)
    }
    pub fn as_str(&self) -> &str {
        match self {
            Self::Static(v) => v,
            Self::String(v) => v.as_str(),
        }
    }
}

impl From<&'static str> for TagKey {
    fn from(string: &'static str) -> Self {
        TagKey::from(string)
    }
}

impl std::ops::Add for TagKey {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let val = format!("{}:{}", self.as_str(), other.as_str());
        TagKey::String(val)
    }
}

impl std::ops::Add<&'static str> for TagKey {
    type Output = Self;
    fn add(self, other: &'static str) -> Self {
        self.add(TagKey::from(other))
    }
}

/// A map from string keys to string values. This makes copies of strings for
/// convenience; don't use in performance sensitive contexts.
// BTreeMap chosen for deterministic serialization.
// We often need to compare output directly, so cannot tolerate reordering
// TODO: fix this in the serialization by having the keys sorted.
#[derive(Clone, Debug, Deserialize, Default, Serialize)]
pub struct Tags(BTreeMap<String, String>);

impl Tags {
    pub fn new(map: BTreeMap<String, String>) -> Tags {
        Tags(map)
    }

    /// Expose inner map
    pub fn map(&self) -> &BTreeMap<String, String> {
        &self.0
    }
}

impl FromStr for Tags {
    type Err = String;

    /// Parse tags from an '=' separated list
    ///
    /// ```
    /// use std::str::FromStr;
    /// use osm2lanes::Tags;
    /// use osm2lanes::TagsRead;
    /// let tags = Tags::from_str("foo=bar\nabra=cadabra").unwrap();
    /// assert_eq!(tags.get("foo"), Some("bar"));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut map = BTreeMap::new();
        for line in s.lines() {
            let (key, val) = line.split_once("=").ok_or("tag must be = separated")?;
            map.insert(key.to_owned(), val.to_owned());
        }
        Ok(Self(map))
    }
}

impl ToString for Tags {
    /// Return tags as an '=' separated list
    ///
    /// ```
    /// use std::str::FromStr;
    /// use std::string::ToString;
    /// use osm2lanes::Tags;
    /// use osm2lanes::TagsRead;
    /// let tags = Tags::from_str("foo=bar\nabra=cadabra").unwrap();
    /// assert_eq!(tags.to_string(), "abra=cadabra\nfoo=bar");
    /// ```
    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|(k, v)| format!("{}={}", k.as_str(), v.as_str()))
            .collect::<Vec<String>>()
            .as_slice()
            .join("\n")
    }
}

// TODO, shouldn't TagKey be passed by reference?
pub trait TagsRead {
    fn get<T: Into<TagKey>>(&self, k: T) -> Option<&str>;
    fn is<T: Into<TagKey>>(&self, k: T, v: &str) -> bool;
    fn is_any<T: Into<TagKey>>(&self, k: T, values: &[&str]) -> bool;
    fn subset<T>(&self, keys: &[T]) -> Self
    where
        T: Clone,
        T: Into<TagKey>;
}

impl TagsRead for Tags {
    fn get<T: Into<TagKey>>(&self, k: T) -> Option<&str> {
        self.0.get(k.into().as_str()).map(|v| v.as_str())
    }

    fn is<T: Into<TagKey>>(&self, k: T, v: &str) -> bool {
        self.get(k) == Some(v)
    }

    fn is_any<T: Into<TagKey>>(&self, k: T, values: &[&str]) -> bool {
        if let Some(v) = self.get(k) {
            values.contains(&v)
        } else {
            false
        }
    }

    // TODO, find a way to do this without so many clones
    fn subset<T>(&self, keys: &[T]) -> Self
    where
        T: Clone,
        T: Into<TagKey>,
    {
        let mut map = Self::default();
        for key in keys {
            let tag_key: TagKey = key.clone().into();
            if let Some(val) = self.get(tag_key.clone()) {
                assert!(map
                    .0
                    .insert(tag_key.as_str().to_owned(), val.to_owned())
                    .is_none());
            }
        }
        map
    }
}

pub trait TagsWrite {
    /// Returns the old value of this key, if it was already present.
    fn insert<K: Into<TagKey>, V: Into<String>>(&mut self, k: K, v: V) -> Option<String>;
}

impl TagsWrite for Tags {
    fn insert<K: Into<TagKey>, V: Into<String>>(&mut self, k: K, v: V) -> Option<String> {
        self.0.insert(k.into().as_str().to_owned(), v.into())
    }
}