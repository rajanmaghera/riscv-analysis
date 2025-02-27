use super::{HasRawText, RawToken};

impl HasRawText for RawToken {
    fn raw_text(&self) -> &str {
        &self.text
    }
}
