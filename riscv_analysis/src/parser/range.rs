use serde::{Deserialize, Serialize};

use super::Position;

#[derive(Debug, PartialEq, Clone, PartialOrd, Ord, Eq, Default, Serialize, Deserialize)]
pub struct Range {
    start: Position,
    end: Position,
}

impl Range {
    #[must_use] pub fn new(start: Position, end: Position) -> Range {
        Range { start, end }
    }

    #[must_use] pub fn start(&self) -> &Position {
        &self.start
    }

    #[must_use] pub fn end(&self) -> &Position {
        &self.end
    }
}

impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}:{} - {}:{}",
            self.start.one_idx_line(),
            self.start.one_idx_column(),
            self.end.one_idx_line(),
            self.end.one_idx_column()
        )
    }
}
