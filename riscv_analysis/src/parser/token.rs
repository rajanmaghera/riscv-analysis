use std::fmt::Display;
use uuid::Uuid;

use crate::passes::DiagnosticLocation;

use super::{Range, RawToken, TokenType};

#[derive(Debug, PartialEq, Clone)]
pub struct RVToken {
    token_type: TokenType,
    raw_token: RawToken,
}

impl RVToken {
    pub fn new<S: Into<String>>(token: TokenType, text: S, pos: Range, file: Uuid) -> Self {
        // TODO: assert token text and positions match
        RVToken {
            token_type: token,
            raw_token: RawToken::new(text, pos, file),
        }
    }

    #[must_use]
    pub fn new_without_text(token: TokenType, pos: Range, file: Uuid) -> Self {
        RVToken {
            token_type: token,
            raw_token: RawToken::new(String::new(), pos, file),
        }
    }

    #[must_use]
    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    #[must_use]
    pub fn raw_token(&self) -> &RawToken {
        &self.raw_token
    }
}

impl From<RVToken> for RawToken {
    fn from(token: RVToken) -> RawToken {
        token.raw_token
    }
}

impl PartialEq<TokenType> for RVToken {
    fn eq(&self, other: &TokenType) -> bool {
        self.token_type == *other
    }
}

impl Display for RVToken {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(&self.token_type, f)
    }
}

impl DiagnosticLocation for RVToken {
    fn file(&self) -> Uuid {
        self.raw_token.file()
    }
    fn range(&self) -> Range {
        self.raw_token.range()
    }

    fn raw_text(&self) -> String {
        self.raw_token.raw_text().clone()
    }
}
