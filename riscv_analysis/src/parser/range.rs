use serde::{Deserialize, Serialize};

use super::Position;

#[derive(Debug, PartialEq, Clone, PartialOrd, Ord, Eq, Default, Serialize, Deserialize)]
pub struct Range {
    start: Position,
    end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Range {
        Range { start, end }
    }

    pub fn start(&self) -> &Position {
        &self.start
    }

    pub fn end(&self) -> &Position {
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
