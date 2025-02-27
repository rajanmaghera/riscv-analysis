use uuid::Uuid;

use crate::passes::DiagnosticLocation;

use super::Token;

impl DiagnosticLocation for Token {
    fn file(&self) -> Uuid {
        self.file
    }
    fn range(&self) -> super::Range {
        self.pos.clone()
    }
}
