use uuid::Uuid;

use super::Range;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct RawToken {
    pub text: String,
    pub pos: Range,
    pub file: Uuid,
}
