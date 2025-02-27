use super::{HasRawText, Token};

impl HasRawText for Token {
    fn raw_text(&self) -> &str {
        &self.text
    }
}
