use uuid::Uuid;

use crate::passes::DiagnosticLocation;

use super::{Range, With};

impl<T> DiagnosticLocation for With<T> {
    fn range(&self) -> Range {
        self.pos.clone()
    }
    fn file(&self) -> Uuid {
        self.file
    }
}
