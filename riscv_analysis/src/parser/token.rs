use std::fmt::Display;
use uuid::Uuid;

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
