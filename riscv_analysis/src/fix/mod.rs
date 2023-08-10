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
}

impl Manipulation {
    #[must_use]
    pub fn line(&self) -> usize {
        match self {
            Manipulation::Insert(_, pos, _, _) => pos.line,
        }
    }

    #[must_use]
    pub fn column(&self) -> usize {
        match self {
            Manipulation::Insert(_, pos, _, _) => pos.column,
        }
    }

    #[must_use]
    pub fn file(&self) -> uuid::Uuid {
        match self {
            Manipulation::Insert(file, _, _, _) => *file,
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
    let exit_text = format!(
        "\n# restore from stack\n{}addi sp, sp, {}\n\n",
        regs.iter()
            .enumerate()
            .map(|(i, reg)| format!("lw {}, {}(sp)\n", reg, i * 4))
            .collect::<String>(),
        count * 4
    );

    let offset = count + 4;

    // determine what the lines to insert the text into

    // Depending on where the entry and exit points are,
    // the number of lines to offset each insertion by will be different
    // We simply have to determine this by using sound logic

    // if the entry is the same file as the exit and it comes after the exit, then we use an offset
    let entry_offset = if entry.node().file() == exit.node().file()
        && entry.node().range().start.line > exit.node().range().start.line
    {
        offset
    } else {
        0
    };

    // if the exit is the same file as the entry and it comes after the exit, then we use an offset
    let exit_offset = if exit.node().file() == entry.node().file()
        && exit.node().range().start.line > entry.node().range().start.line
    {
        offset
    } else {
        0
    };

    vec![
        Manipulation::Insert(
            entry.node().file(),
            Position {
                line: entry.node().range().start.line + entry_offset,
                column: entry.node().range().start.column,
            },
            entry_text,
            offset,
        ),
        Manipulation::Insert(
            exit.node().file(),
            Position {
                line: exit.node().range().start.line + exit_offset,
                column: exit.node().range().start.column,
            },
            exit_text,
            offset,
        ),
    ]
}
