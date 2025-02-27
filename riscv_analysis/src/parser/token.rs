use std::fmt::Display;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::passes::DiagnosticLocation;

use super::{Range, TokenType};

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token: TokenType,
    pub text: String,
    pub pos: Range,
    pub file: Uuid,
}

impl Token {
    pub fn new(token: TokenType, text: String, pos: Range, file: Uuid) -> Self {
        // TODO: assert token text and positions match
        Token {
            token,
            text,
            pos,
            file,
        }
    }

    pub fn new_without_text(token: TokenType, pos: Range, file: Uuid) -> Self {
        Token {
            token,
            text: "".to_owned(),
            pos,
            file,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct RawToken {
    pub text: String,
    pub pos: Range,
    pub file: Uuid,
}

impl PartialEq<TokenType> for Token {
    fn eq(&self, other: &TokenType) -> bool {
        self.token == *other
    }
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

impl Default for Token {
    fn default() -> Self {
        Token {
            token: TokenType::Newline,
            text: "".to_owned(),
            file: Uuid::nil(),
            pos: Range::default(),
        }
    }
}

#[derive(Clone)]
pub struct With<T> {
    pub token: TokenType,
    text: String,
    pub pos: Range,
    pub file: Uuid,
    pub data: T,
}

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
        write!(f, "{:?}", self.data)
    }
}

impl<T> std::fmt::Display for With<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.data)
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

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.token)
    }
}

pub struct VecTokenDisplayWrapper<'a>(&'a Vec<Token>);
impl Display for VecTokenDisplayWrapper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for t in self.0 {
            write!(f, "{t}")?;
        }
        Ok(())
    }
}

pub trait ToDisplayForTokenVec {
    fn to_display(&self) -> VecTokenDisplayWrapper;
}

impl ToDisplayForTokenVec for Vec<Token> {
    fn to_display(&self) -> VecTokenDisplayWrapper {
        VecTokenDisplayWrapper(self)
    }
}

// implement display for Range

impl<T> DiagnosticLocation for With<T> {
    fn range(&self) -> Range {
        self.pos.clone()
    }
    fn file(&self) -> Uuid {
        self.file
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

impl<T> Eq for With<T> where T: Eq {}

impl<T> With<T>
where
    T: PartialEq<T>,
{
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

impl<T> TryFrom<Token> for With<T>
where
    T: TryFrom<Token>,
{
    type Error = T::Error;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        Ok(With {
            pos: value.pos.clone(),
            token: value.token.clone(),
            file: value.file,
            text: value.text.clone(),
            data: T::try_from(value)?,
        })
    }
}

impl TryFrom<Token> for String {
    type Error = String;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.token {
            TokenType::Symbol(s) | TokenType::String(s) => Ok(s),
            _ => Err(format!("Expected symbol or string, got {:?}", value.token)),
        }
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
