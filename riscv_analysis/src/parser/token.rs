use std::fmt::Display;
use uuid::Uuid;

use crate::passes::DiagnosticLocation;

use super::{HasRawText, Range, TokenType};

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    token: TokenType,
    text: String,
    pos: Range,
    file: Uuid,
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

    pub fn token_type(&self) -> &TokenType {
        &self.token
    }
}

impl PartialEq<TokenType> for Token {
    fn eq(&self, other: &TokenType) -> bool {
        self.token == *other
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

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(&self.token, f)
    }
}

impl DiagnosticLocation for Token {
    fn file(&self) -> Uuid {
        self.file
    }
    fn range(&self) -> super::Range {
        self.pos.clone()
    }
}

impl HasRawText for Token {
    fn raw_text(&self) -> &str {
        &self.text
    }
}
