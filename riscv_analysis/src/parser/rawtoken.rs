use uuid::Uuid;

use crate::passes::DiagnosticLocation;

use super::{HasRawText, Range};

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

impl HasRawText for RawToken {
    fn raw_text(&self) -> &str {
        &self.text
    }
}
impl DiagnosticLocation for RawToken {
    fn file(&self) -> Uuid {
        self.file
    }
    fn range(&self) -> super::Range {
        self.pos.clone()
    }
}
