use std::fmt::Display;
use uuid::Uuid;

use crate::passes::DiagnosticLocation;

use super::{HasRawText, Range, RawToken, TokenType};

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    token_type: TokenType,
    raw_token: RawToken,
}

impl Token {
    pub fn new<S: Into<String>>(token: TokenType, text: S, pos: Range, file: Uuid) -> Self {
        // TODO: assert token text and positions match
        Token {
            token_type: token,
            raw_token: RawToken::new(text, pos, file),
        }
    }

    pub fn new_without_text(token: TokenType, pos: Range, file: Uuid) -> Self {
        Token {
            token_type: token,
            raw_token: RawToken::new("".to_owned(), pos, file),
        }
    }

    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn raw_token(&self) -> &RawToken {
        &self.raw_token
    }
}

impl From<Token> for RawToken {
    fn from(token: Token) -> RawToken {
        token.raw_token
    }
}

impl PartialEq<TokenType> for Token {
    fn eq(&self, other: &TokenType) -> bool {
        self.token_type == *other
    }
}
impl Default for Token {
    fn default() -> Self {
        Token {
            token_type: TokenType::Newline,
            raw_token: RawToken::default(),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(&self.token_type, f)
    }
}

impl DiagnosticLocation for Token {
    fn file(&self) -> Uuid {
        self.raw_token.file()
    }
    fn range(&self) -> super::Range {
        self.raw_token.range()
    }
}

impl HasRawText for Token {
    fn raw_text(&self) -> &str {
        &self.raw_token.raw_text()
    }
}
