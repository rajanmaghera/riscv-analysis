use super::{HasRawText, ParserNode};

impl HasRawText for ParserNode {
    fn raw_text(&self) -> &str {
        &self.token().raw_text()
    }
}
