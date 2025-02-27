use std::fmt::Display;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::passes::DiagnosticLocation;

use super::Range;

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

/// Token type for the parser
///
/// This is the token type for the parser. It is used to
/// determine what the token is, and what to do with it.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default)]
pub enum TokenType {
    /// Left Parenthesis '('
    LParen,
    /// Right Parenthesis ')'
    RParen,
    /// Newline '\n'
    #[default]
    Newline,
    /// Label: text ending in ':'
    ///
    /// This is used to mark a label entry point in the code.
    /// It is used to mark the start of a function, or a jump
    /// target.
    Label(String),
    /// Symbol: text not matching any special token types
    ///
    /// This is used to mark a symbol. A symbol is a
    /// generic token that can be converted into a
    /// more specific type. The types include
    /// instructions, registers, numbers, and special CSR numbers/regs.
    Symbol(String),
    /// Directive: text starting with '.'
    ///
    /// This is used to mark a directive. A directive is a
    /// command to the assembler to do something. For example,
    /// the `.text` directive tells the assembler to start
    /// assembling code into the text section.
    ///
    /// The most important directive is `.include`. This
    /// directive tells the assembler to include the file
    /// specified in the directive. This case has to be handled
    /// specially, as the file is not parsed, but rather
    /// included as is.
    Directive(String),
    /// String: text enclosed in double quotes
    String(String),
    // Char: Single character enclosed in single quotes
    Char(char),
    /// Comment: text starting with # up until the first newline.
    /// A comment is a line of text that is ignored by
    /// the assembler, but they are useful for human readers.
    /// They may be used to annotate the assembler in the future.
    Comment(String),
}

impl TokenType {
    #[must_use]
    pub fn as_original_string(&self) -> String {
        match self {
            TokenType::LParen => "(".to_owned(),
            TokenType::RParen => ")".to_owned(),
            TokenType::Newline => "\n".to_owned(),
            TokenType::Label(l) => format!("{l}:"),
            TokenType::Symbol(s) => s.clone(),
            TokenType::Directive(d) => format!(".{d}"),
            TokenType::String(s) => format!("\"{s}\""),
            TokenType::Char(c) => format!("'{c}'"),
            TokenType::Comment(c) => format!("#{c}:"),
        }
    }
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

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TokenType::Label(s) => writeln!(f, "LABEL({s})"),
            TokenType::Symbol(s) => write!(f, "SYMBOL({s})"),
            TokenType::Directive(s) => write!(f, "DIRECTIVE({s})"),
            TokenType::String(s) => write!(f, "STRING({s})"),
            TokenType::Char(c) => write!(f, "CHAR({c})"),
            TokenType::Comment(s) => write!(f, "COMMENT{s}"),
            TokenType::Newline => write!(f, "NEWLINE"),
            TokenType::LParen => write!(f, "LPAREN"),
            TokenType::RParen => write!(f, "RPAREN"),
        }
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
