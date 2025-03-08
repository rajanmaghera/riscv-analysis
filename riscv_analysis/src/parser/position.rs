use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Copy, Clone, Eq, Serialize, Deserialize, Default)]
pub struct Position {
    line: usize,
    column: usize,
    raw_index: usize,
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.raw_index.cmp(&other.raw_index)
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Position {
    /// Create a new position from the zero-indexed `line` and `column` numbers.
    #[must_use]
    pub fn new(line: usize, column: usize, raw_index: usize) -> Self {
        Position {
            line,
            column,
            raw_index,
        }
    }

    /// Get the line number in zero-based index.
    #[must_use]
    pub fn zero_idx_line(&self) -> usize {
        self.line
    }

    /// Get the line number in one-based index.
    #[must_use]
    pub fn one_idx_line(&self) -> usize {
        self.line + 1
    }

    /// Get the column number in zero-based index.
    #[must_use]
    pub fn zero_idx_column(&self) -> usize {
        self.column
    }

    /// Get the column number in one-based index.
    #[must_use]
    pub fn one_idx_column(&self) -> usize {
        self.column + 1
    }

    /// Get the raw index of the position.
    #[must_use]
    pub fn raw_index(&self) -> usize {
        self.raw_index
    }

    /// Increment the column and `raw_index` number by one.
    pub fn increment_column(&mut self) {
        self.column += 1;
        self.raw_index += 1;
    }

    /// Decrement the column and `raw_index` until at the beginning of
    /// the column.
    ///
    /// In other terms, decrement until column = 0.
    pub fn decrement_to_beginning_of_line(&mut self) {
        self.raw_index -= self.column;
        self.column = 0;
    }
}
