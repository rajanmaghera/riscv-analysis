use std::ops::Deref;

use uuid::Uuid;

use super::FullLexer;

/// Text content that is ready to be lexed.
///
/// This struct represents text content that is ready to be lexed. This text content can come from any source and its source is represented by a uuid.
pub struct LexingString<'a>(&'a str, Uuid);

impl<'a> LexingString<'a> {
    /// Create a new lexing string.
    fn new(text: &str, uuid: Uuid) -> Self {
        LexingString(text, uuid)
    }
}

impl<'a> Deref for LexingString<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> From<LexingString<'a>> for FullLexer<'a> {
    fn from(value: LexingString<'a>) -> Self {
        FullLexer::new(&value.0, value.1)
    }
}
