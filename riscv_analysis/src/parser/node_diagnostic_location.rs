use uuid::Uuid;

use crate::passes::DiagnosticLocation;

use super::ParserNode;

impl DiagnosticLocation for ParserNode {
    fn file(&self) -> Uuid {
        self.token().file
    }
    fn range(&self) -> super::Range {
        self.token().pos
    }
}
