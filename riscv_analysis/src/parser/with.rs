use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Range, Token, TokenType};

#[derive(Clone)]
pub struct With<T> {
    pub token: TokenType,
    pub text: String,
    pub pos: Range,
    pub file: Uuid,
    pub data: T,
}

impl<T> With<T> {
    pub fn info(&self) -> Token {
        Token::new(
            self.token.clone(),
            self.text.to_string(),
            self.pos.clone(),
            self.file,
        )
    }

    pub fn new(data: T, info: Token) -> Self {
        With {
            token: info.token,
            text: info.text,
            pos: info.pos,
            file: info.file,
            data,
        }
    }
}

// Forwarding implementations

impl<T> Serialize for With<T>
where
    T: Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.data.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for With<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(With {
            token: TokenType::default(),
            pos: Range::default(),
            file: Uuid::nil(),
            text: "".to_owned(),
            data: T::deserialize(deserializer)?,
        })
    }
}

impl<T> std::fmt::Debug for With<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.data, f)
    }
}

impl<T> std::fmt::Display for With<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.data, f)
    }
}

impl<T> Hash for With<T>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl<T> PartialOrd for With<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &With<T>) -> Option<std::cmp::Ordering> {
        self.data.partial_cmp(&other.data)
    }
}

impl<T> Ord for With<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.data.cmp(&other.data)
    }
}

impl<T> PartialEq<With<T>> for With<T>
where
    T: PartialEq<T>,
{
    fn eq(&self, other: &With<T>) -> bool {
        self.data == other.data
    }
}

impl<T> PartialEq<T> for With<T>
where
    T: PartialEq<T>,
{
    fn eq(&self, other: &T) -> bool {
        self.data == *other
    }
}

impl<T> Eq for With<T> where T: Eq {}
