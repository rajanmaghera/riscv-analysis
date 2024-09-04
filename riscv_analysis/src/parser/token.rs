use std::fmt::Display;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::passes::DiagnosticLocation;

#[derive(Debug, PartialEq, Copy, Clone, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub raw_index: usize,
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Ord, Eq, Default, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Info {
    pub token: Token,
    pub pos: Range,
    pub file: Uuid,
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
pub enum Token {
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
}

impl Token {
    #[must_use]
    pub fn as_original_string(&self) -> String {
        match self {
            Token::LParen => "(".to_owned(),
            Token::RParen => ")".to_owned(),
            Token::Newline => "\n".to_owned(),
            Token::Label(l) => format!("{l}:"),
            Token::Symbol(s) => s.clone(),
            Token::Directive(d) => format!(".{d}"),
            Token::String(s) => format!("\"{s}\""),
        }
    }
}

impl PartialEq<Token> for Info {
    fn eq(&self, other: &Token) -> bool {
        self.token == *other
    }
}
impl<T> With<T> {
    pub fn info(&self) -> Info {
        Info {
            file: self.file,
            token: self.token.clone(),
            pos: self.pos.clone(),
        }
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

impl Default for Info {
    fn default() -> Self {
        Info {
            token: Token::Newline,
            file: Uuid::nil(),
            pos: Range {
                start: Position {
                    line: 0,
                    column: 0,
                    raw_index: 0,
                },
                end: Position {
                    line: 0,
                    column: 0,
                    raw_index: 0,
                },
            },
        }
    }
}

#[derive(Clone)]
pub struct With<T> {
    pub token: Token,
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
            token: Token::default(),
            pos: Range::default(),
            file: Uuid::nil(),
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

impl Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.token)
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::Label(s) => writeln!(f, "LABEL({s})"),
            Token::Symbol(s) => write!(f, "SYMBOL({s})"),
            Token::Directive(s) => write!(f, "DIRECTIVE({s})"),
            Token::String(s) => write!(f, "STRING({s})"),
            Token::Newline => write!(f, "NEWLINE"),
            Token::LParen => write!(f, "LPAREN"),
            Token::RParen => write!(f, "RPAREN"),
        }
    }
}

pub struct VecTokenDisplayWrapper<'a>(&'a Vec<Info>);
impl<'a> Display for VecTokenDisplayWrapper<'a> {
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

impl ToDisplayForTokenVec for Vec<Info> {
    fn to_display(&self) -> VecTokenDisplayWrapper {
        VecTokenDisplayWrapper(self)
    }
}

// implement display for Range
impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}:{} - {}:{}",
            self.start.line, self.start.column, self.end.line, self.end.column
        )
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
    pub fn new(data: T, info: Info) -> Self {
        With {
            token: info.token,
            pos: info.pos,
            file: info.file,
            data,
        }
    }
}

impl<T> TryFrom<Info> for With<T>
where
    T: TryFrom<Info>,
{
    type Error = T::Error;

    fn try_from(value: Info) -> Result<Self, Self::Error> {
        Ok(With {
            pos: value.pos.clone(),
            token: value.token.clone(),
            file: value.file,
            data: T::try_from(value)?,
        })
    }
}

impl TryFrom<Info> for String {
    type Error = String;

    fn try_from(value: Info) -> Result<Self, Self::Error> {
        match value.token {
            Token::Symbol(s) | Token::String(s) => Ok(s),
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
