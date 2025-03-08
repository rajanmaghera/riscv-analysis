use uuid::Uuid;

use crate::passes::DiagnosticLocation;

use super::Range;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct RawToken {
    text: String,
    pos: Range,
    file: Uuid,
}

impl RawToken {
    pub fn new<S: Into<String>>(text: S, pos: Range, file: Uuid) -> RawToken {
        RawToken {
            text: text.into(),
            pos,
            file,
        }
    }
}

impl DiagnosticLocation for RawToken {
    fn file(&self) -> Uuid {
        self.file
    }
    fn range(&self) -> super::Range {
        self.pos.clone()
    }
    fn raw_text(&self) -> String {
        self.text.clone()
    }
}
