use uuid::Uuid;

use crate::passes::DiagnosticLocation;

use super::Range;

#[derive(Debug, PartialEq, Clone)]
pub struct RawToken {
    text: String,
    pos: Range,
    file: Uuid,
}

impl RawToken {
    pub fn new<S: Into<String>>(text: S, pos: Range, file: Uuid) -> RawToken {
        let text = text.into();

        // These assert statements ensure that every raw token:
        // - only span one line,
        // - have a valid range (end > start), and
        // - the text matches the size of the range
        //
        // RawTokens are meant to match the actual input source file; e.g. comments include
        // the '#' and strings include the quotes " " and escape characters. The Token struct is
        // meant to represent the underlying data however is needed; e.g. strings get converted to the real
        // string representation.
        debug_assert_eq!(pos.start().zero_idx_line(), pos.end().zero_idx_line());
        debug_assert!(pos.end().zero_idx_column() > pos.start().zero_idx_column());
        debug_assert!(pos.end().raw_index() > pos.start().raw_index());
        debug_assert_eq!(
            pos.end().zero_idx_column() - pos.start().zero_idx_column(),
            pos.end().raw_index() - pos.start().raw_index()
        );
        debug_assert_eq!(pos.end().raw_index() - pos.start().raw_index(), text.len());

        RawToken { text, pos, file }
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
