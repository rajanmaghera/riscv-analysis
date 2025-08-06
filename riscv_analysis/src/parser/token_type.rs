use std::fmt::Display;

use serde::{Deserialize, Serialize};

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
    /// String: text enclosed in double quotes
    String(String),
    // Char: Single character enclosed in single quotes
    Char(char),
    /// Comment: text starting with # up until the first newline.
    /// A comment is a line of text that is ignored by
    /// the assembler, but they are useful for human readers.
    /// They may be used to annotate the assembler in the future.
    Comment(String),
    /// High specifier
    PercentHigh,
    /// Low specifier
    PercentLow,
    /// Plus specifier
    Plus(u32),
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
            TokenType::String(s) => format!("\"{s}\""),
            TokenType::Char(c) => format!("'{c}'"),
            TokenType::Comment(c) => format!("#{c}:"),
            TokenType::PercentHigh => "%hi".to_owned(),
            TokenType::PercentLow => "%lo".to_owned(),
            TokenType::Plus(n) => format!("+{}", n),
        }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TokenType::Label(s) => writeln!(f, "LABEL({s})"),
            TokenType::Symbol(s) => write!(f, "SYMBOL({s})"),
            TokenType::String(s) => write!(f, "STRING({s})"),
            TokenType::Char(c) => write!(f, "CHAR({c})"),
            TokenType::Comment(s) => write!(f, "COMMENT{s}"),
            TokenType::Newline => write!(f, "NEWLINE"),
            TokenType::LParen => write!(f, "LPAREN"),
            TokenType::RParen => write!(f, "RPAREN"),
            TokenType::PercentHigh => write!(f, "PERCENT_HIGH"),
            TokenType::PercentLow => write!(f, "PERCENT_LOW"),
            TokenType::Plus(n) => writeln!(f, "PLUS({n})"),
        }
    }
}
