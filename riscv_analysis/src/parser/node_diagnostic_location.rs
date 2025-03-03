use uuid::Uuid;

use crate::passes::DiagnosticLocation;

use super::ParserNode;

impl DiagnosticLocation for ParserNode {
    fn file(&self) -> Uuid {
        self.token().file()
    }
    fn range(&self) -> super::Range {
        self.token().range()
    }

    fn raw_text(&self) -> String {
        self.token().raw_text()
    }
}
