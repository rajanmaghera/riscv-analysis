use std::rc::Rc;

use itertools::Itertools;

use crate::{
    cfg::{Cfg, Function},
    parser::{Position, Range},
    passes::DiagnosticLocation,
};

/// SUPPORT FOR STACK FIXES

/// On stack fix with an input function, we will:
/// - insert stack updates to entry
/// - find exit points of code, if there is one, insert stack updates
/// TODO if there are multiple exit points, convert to a single exit point by adding
///   a label in between

pub enum Manipulation {
    /// Insert text at a given position
    ///
    /// (file, position, text, lines)
    Insert(uuid::Uuid, Position, String, usize),
    /// Replace text at a given position
    ///
    /// (file, range, text, lines rem, lines added)
    Replace(uuid::Uuid, Range, String, usize, usize),
}

impl Manipulation {
    #[must_use]
    pub fn line(&self) -> usize {
        match self {
            Manipulation::Insert(_, pos, _, _) => pos.line,
            Manipulation::Replace(_, range, _, _, _) => range.start.line,
        }
    }

    #[must_use]
    pub fn column(&self) -> usize {
        match self {
            Manipulation::Insert(_, pos, _, _) => pos.column,
            Manipulation::Replace(_, range, _, _, _) => range.start.column,
        }
    }

    #[must_use]
    pub fn raw_pos(&self) -> usize {
        match self {
            Manipulation::Insert(_, pos, _, _) => pos.raw_index,
            Manipulation::Replace(_, range, _, _, _) => range.start.raw_index,
        }
    }

    #[must_use]
    pub fn file(&self) -> uuid::Uuid {
        match self {
            Manipulation::Insert(file, _, _, _) | Manipulation::Replace(file, _, _, _, _) => *file,
        }
    }
}

/// Return the ranges of the function labels
///
/// This allows LSP servers to determine where we can mark
/// function actions.
pub fn get_function_label_ranges(cfg: &Cfg) -> Vec<Range> {
    cfg.label_function_map
        .keys()
        .map(crate::passes::DiagnosticLocation::range)
        .collect()
}

#[must_use]
pub fn fix_stack(func: &Rc<Function>) -> Vec<Manipulation> {
    let mut changes: Vec<Manipulation> = Vec::new();

    // go to the beginning of the function
    let entry = &func.entry;
    let exit = &func.exit;
    // sorted to make the output nicer
    let regs = func.to_save().into_iter().sorted().collect_vec();
    let count = regs.len();
    let entry_text = format!(
        "\n# save to stack\naddi sp, sp, -{}\n{}\n",
        count * 4,
        regs.iter()
            .enumerate()
            .map(|(i, reg)| format!("sw {}, {}(sp)\n", reg, i * 4))
            .collect::<String>()
    );
    let other_exits = func.other_exits.borrow();

    let (exit_offset, exit_label): (usize, String) = if other_exits.len() > 0 {
        // get a new safe label name for the function
        let name = func.get_empty_label();

        for ex in other_exits.iter() {
            // replace other exits with jump to exit label
            changes.push(Manipulation::Replace(
                ex.node().file(),
                ex.node().range(),
                format!("j {}", name),
                0,
                0,
            ));
        }

        // prepend exit text with new label
        (2, format!("\n{}:\n", name))
    } else {
        (0, "".into())
    };

    let exit_text = format!(
        "\n{}# restore from stack\n{}addi sp, sp, {}\n\n",
        exit_label,
        regs.iter()
            .enumerate()
            .map(|(i, reg)| format!("lw {}, {}(sp)\n", reg, i * 4))
            .collect::<String>(),
        count * 4
    );

    // Move range to beginning of line
    let mut entry_range = entry.node().range().start;
    entry_range.raw_index -= entry_range.column;
    entry_range.column = 0;

    let mut exit_range = exit.node().range().start;
    exit_range.raw_index -= exit_range.column;
    exit_range.column = 0;

    changes.push(Manipulation::Insert(
        entry.node().file(),
        entry_range,
        entry_text,
        count + 4,
    ));
    changes.push(Manipulation::Insert(
        exit.node().file(),
        exit_range,
        exit_text,
        count + 4 + exit_offset,
    ));

    changes
}
