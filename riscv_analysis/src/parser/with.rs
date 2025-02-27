use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::passes::DiagnosticLocation;

use super::{HasRawText, Range, Token, TokenType};

#[derive(Clone)]
pub struct With<T> {
    token: TokenType,
    text: String,
    pos: Range,
    file: Uuid,
    underlying_data: T,
}

impl<T> With<T> {
    pub fn new(data: T, info: Token) -> Self {
        With {
            token: info.token_type().clone(),
            text: info.raw_text().to_owned(),
            pos: info.range(),
            file: info.file(),
            underlying_data: data,
        }
    }

    pub fn get(&self) -> &T {
        &self.underlying_data
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.underlying_data
    }
}

impl<T> From<With<T>> for Token {
    fn from(with: With<T>) -> Token {
        Token::new(with.token, with.text, with.pos, with.file)
    }
}

impl<T> DiagnosticLocation for With<T> {
    fn range(&self) -> Range {
        self.pos.clone()
    }
    fn file(&self) -> Uuid {
        self.file
    }
}

impl<T> HasRawText for With<T> {
    fn raw_text(&self) -> &str {
        self.text.as_str()
    }
}

// Forwarding implementations

impl<T> Serialize for With<T>
where
    T: Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.underlying_data.serialize(serializer)
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
            underlying_data: T::deserialize(deserializer)?,
        })
    }
}

impl<T> std::fmt::Debug for With<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.underlying_data, f)
    }
}

impl<T> std::fmt::Display for With<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.underlying_data, f)
    }
}

impl<T> Hash for With<T>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.underlying_data.hash(state);
    }
}

impl<T> PartialOrd for With<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &With<T>) -> Option<std::cmp::Ordering> {
        self.underlying_data.partial_cmp(&other.underlying_data)
    }
}

impl<T> Ord for With<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.underlying_data.cmp(&other.underlying_data)
    }
}

impl<T> PartialEq<With<T>> for With<T>
where
    T: PartialEq<T>,
{
    fn eq(&self, other: &With<T>) -> bool {
        self.underlying_data.eq(&other.underlying_data)
    }
}

impl<T> PartialEq<T> for With<T>
where
    T: PartialEq<T>,
{
    fn eq(&self, other: &T) -> bool {
        self.underlying_data.eq(other)
    }
}

impl<T> Eq for With<T> where T: Eq {}
