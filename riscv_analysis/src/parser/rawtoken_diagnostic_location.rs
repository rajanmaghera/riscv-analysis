use uuid::Uuid;

use crate::passes::DiagnosticLocation;

use super::RawToken;

impl DiagnosticLocation for RawToken {
    fn file(&self) -> Uuid {
        self.file
    }
    fn range(&self) -> super::Range {
        self.pos.clone()
    }
}
