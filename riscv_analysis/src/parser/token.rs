use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::passes::DiagnosticLocation;

#[derive(Debug, PartialEq, Copy, Clone, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Position {
    /// Line, zero indexed
    pub line: usize,
    /// Column, zero indexed.
    ///
    /// Column could point to a non-existant value. The behaviour of the position is to add a column to the range when advancing.
    pub column: usize,
    /// Raw index of character.
    pub raw_index: usize,
}

impl Position {
    pub fn new(line: usize, column: usize, raw_index: usize) -> Position {
        Position {
            line,
            column,
            raw_index,
        }
    }
}

impl Add<usize> for Position {
    type Output = Position;

    fn add(self, rhs: usize) -> Self::Output {
        Position {
            line: self.line,
            column: self.column + rhs,
            raw_index: self.raw_index,
        }
    }
}

impl Sub for Position {
    type Output = usize;

    fn sub(self, rhs: Self) -> Self::Output {
        self.column - rhs.column
    }
}

impl Position {
    pub fn add_line(&mut self) {
        self.line += 1;
        self.column = 0;
        self.raw_index += 1;
    }

    pub fn add_char(&mut self) {
        *self = *self + 1usize;
        self.raw_index += 1;
    }
}

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd, Ord, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Range {
    /// Start position of range, inclusive
    pub start: Position,
    /// End position of range, exclusive
    pub end: Position,
    /// Source of range
    pub source: Uuid,
}

impl Range {
    pub fn new(start: Position, end: Position, source: Uuid) -> Range {
        Range { start, end, source }
    }
}

#[cfg(test)]
impl Default for Range {
    fn default() -> Self {
        Self {
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
            source: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[non_exhaustive]
pub struct Token<'a> {
    pub token: TokenType<'a>,
    pub range: Range,
}

impl<'a> Token<'a> {
    pub fn new(token: TokenType<'a>, range: Range) -> Token {
        Token { token, range }
    }
}

#[cfg(test)]
impl<'a> Default for Token<'a> {
    fn default() -> Self {
        Self {
            token: Default::default(),
            range: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SourceText<'a> {
    pub text: &'a str,
    pub range: Range,
}

impl<'a> SourceText<'a> {
    pub fn new(text: &'a str, range: Range) -> SourceText {
        SourceText { text, range }
    }

    pub fn into_token(self, token: TokenType) -> Token {
        Token::new(token, self.range)
    }
}

pub trait WithRange {
    fn with_range(&self, range: Range) -> SourceText;
}

pub trait WithRangeFromSized: WithRange {
    fn with_range_size(&self, start: Position, file: Uuid) -> SourceText;
}

impl<'a> WithRange for &'a str {
    fn with_range(&self, range: Range) -> SourceText {
        SourceText::new(self, range)
    }
}

impl<'a> WithRangeFromSized for &'a str {
    fn with_range_size(&self, start: Position, file: Uuid) -> SourceText {
        SourceText::new(self, Range::new(start, start + self.len(), file))
    }
}

/// TokenType type for the parser
///
/// This is the token type for the parser. It is used to
/// determine what the token is, and what to do with it.
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize, Default)]
pub enum TokenType<'a> {
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
    Label(&'a str),
    /// Symbol: text not matching any special token types
    ///
    /// This is used to mark a symbol. A symbol is a
    /// generic token that can be converted into a
    /// more specific type. The types include
    /// instructions, registers, numbers, and special CSR numbers/regs.
    Symbol(&'a str),
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
    Directive(&'a str),
    /// String: text enclosed in double quotes
    String(&'a str),
}

impl<'a> TokenType<'a> {
    #[must_use]
    pub fn as_original_string(&self) -> String {
        match self {
            TokenType::LParen => "(".to_owned(),
            TokenType::RParen => ")".to_owned(),
            TokenType::Newline => "\n".to_owned(),
            TokenType::Label(l) => format!("{l}:"),
            TokenType::Symbol(s) => s.to_string(),
            TokenType::Directive(d) => format!(".{d}"),
            TokenType::String(s) => format!("\"{s}\""),
        }
    }
}

impl<'a, T> With<'a, T> {
    pub fn as_token(&self) -> &Token {
        &self.token
    }
}

impl<'a, T> PartialOrd for With<'a, T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &With<T>) -> Option<std::cmp::Ordering> {
        self.data.partial_cmp(&other.data)
    }
}

impl<'a, T> Ord for With<'a, T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.data.cmp(&other.data)
    }
}

#[derive(Clone)]
pub struct With<'a, T> {
    pub data: T,
    pub token: Token<'a>,
}

pub trait WithToken: Sized {
    fn with_token<'a>(self, token: Token<'a>) -> With<'a, Self> {
        With::new(self, token)
    }
}

#[cfg(test)]
pub trait WithTokenTestDefault: Sized {
    fn with_test_token<'a>(self) -> With<'a, Self> {
        With::new(self, Token::default())
    }
}

#[cfg(test)]
impl<T> WithTokenTestDefault for T {}

impl<T> WithToken for T {}

impl<'a, T> With<'a, T> {
    fn new(data: T, token: Token) -> Self {
        With { data, token }
    }
}

impl<'a, T> Serialize for With<'a, T>
where
    T: Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.data.serialize(serializer)
    }
}

#[cfg(test)]
impl<'a, 'de, T> Deserialize<'de> for With<'a, T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(T::deserialize(deserializer)?.with_test_token())
    }
}

impl<'a, T> std::fmt::Debug for With<'a, T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}

impl<'a, T> std::fmt::Display for With<'a, T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.data)
    }
}

impl<'a, T> Hash for With<'a, T>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.token)
    }
}

impl<'a> Display for TokenType<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TokenType::Label(s) => writeln!(f, "LABEL({s})"),
            TokenType::Symbol(s) => write!(f, "SYMBOL({s})"),
            TokenType::Directive(s) => write!(f, "DIRECTIVE({s})"),
            TokenType::String(s) => write!(f, "STRING({s})"),
            TokenType::Newline => write!(f, "NEWLINE"),
            TokenType::LParen => write!(f, "LPAREN"),
            TokenType::RParen => write!(f, "RPAREN"),
        }
    }
}

pub struct VecTokenTypeDisplayWrapper<'a>(&'a Vec<Token<'a>>);
impl<'a> Display for VecTokenTypeDisplayWrapper<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for t in self.0 {
            write!(f, "{t}")?;
        }
        Ok(())
    }
}

pub trait ToDisplayForTokenTypeVec {
    fn to_display(&self) -> VecTokenTypeDisplayWrapper;
}

impl<'a> ToDisplayForTokenTypeVec for Vec<Token<'a>> {
    fn to_display(&self) -> VecTokenTypeDisplayWrapper {
        VecTokenTypeDisplayWrapper(self)
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

impl<'a, T> DiagnosticLocation for With<'a, T> {
    fn range(&self) -> Range {
        self.token.range
    }
    fn file(&self) -> Uuid {
        self.token.range.source
    }
}

impl<'a, T> PartialEq<With<'a, T>> for With<'a, T>
where
    T: PartialEq<T>,
{
    fn eq(&self, other: &With<T>) -> bool {
        self.data == other.data
    }
}

impl<'a, T> Eq for With<'a, T> where T: Eq {}

impl<'a> TryFrom<Token<'a>> for &'a str {
    type Error = String;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.token {
            TokenType::Symbol(s) | TokenType::String(s) => Ok(s),
            _ => Err(format!("Expected symbol or string, got {:?}", value.token)),
        }
    }
}

impl<'a, T> PartialEq<T> for With<'a, T>
where
    T: PartialEq<T>,
{
    fn eq(&self, other: &T) -> bool {
        self.data == *other
    }
}

trait TokenTypeExpression {
    fn debug_tokens(&self);
}

impl<'a> TokenTypeExpression for Vec<TokenType<'a>> {
    fn debug_tokens(&self) {
        print!("TokenTypes: ");
        for item in self {
            print!("[{item}]");
        }
        println!();
    }
}
