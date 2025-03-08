use std::{
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::passes::DiagnosticLocation;

use super::{Range, RawToken, Register, Token};

#[derive(Clone)]
pub struct With<T> {
    token: Token,
    underlying_data: T,
}

impl<T> Deref for With<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.underlying_data
    }
}

impl<T> DerefMut for With<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.underlying_data
    }
}

impl<T> With<T> {
    pub fn new(data: T, token: Token) -> Self {
        With {
            token,
            underlying_data: data,
        }
    }

    pub fn get(&self) -> &T {
        &self.underlying_data
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.underlying_data
    }

    pub fn get_cloned(&self) -> T
    where
        T: Clone,
    {
        self.underlying_data.clone()
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn raw_token(&self) -> &RawToken {
        self.token.raw_token()
    }
}

impl<T> DiagnosticLocation for With<T> {
    fn range(&self) -> Range {
        self.token.range()
    }
    fn file(&self) -> Uuid {
        self.token.file()
    }
    fn raw_text(&self) -> String {
        self.token.raw_text()
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
            token: Token::default(),
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

// Blanket implementation for into()

impl From<With<Register>> for Register {
    fn from(with: With<Register>) -> Register {
        *with.get()
    }
}
