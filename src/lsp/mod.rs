// Type conversions for LSP

use std::convert::From;

use crate::parser::Range as MyRange;
use crate::passes::LintError::*;
use crate::passes::{LintError, WarningLevel};
use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

impl From<&MyRange> for Range {
    fn from(r: &MyRange) -> Self {
        lsp_types::Range {
            start: Position {
                line: r.start.line.try_into().unwrap_or(0),
                character: r.start.column.try_into().unwrap_or(0),
            },
            end: Position {
                line: r.end.line.try_into().unwrap_or(0),
                character: r.end.column.try_into().unwrap_or(0),
            },
        }
    }
}

impl From<WarningLevel> for DiagnosticSeverity {
    fn from(w: WarningLevel) -> Self {
        match w {
            WarningLevel::Warning => DiagnosticSeverity::WARNING,
            WarningLevel::Error => DiagnosticSeverity::ERROR,
        }
    }
}

impl From<&LintError> for Diagnostic {
    fn from(e: &LintError) -> Self {
        let range = e.range();
        let related = match &e {
            InvalidUseAfterCall(_, label) => Some(vec![
                // TODO determine name/URI of function
            //     DiagnosticRelatedInformation {
            //     location: lsp_types::Location {
            //         // TODO this is a hack, we need to get the file path from the parser
            //         uri: lsp_types::Url::parse("file:///").unwrap(),
            //         range: label.pos.borrow().into(),
            //     },
            //     message: format!(
            //         "The function call to [{}] invalidates any temporary registers afterwards.",
            //         label
            //             .entry
            //             .labels
            //             .iter()
            //             .map(|x| x.data.0.clone())
            //             .collect::<Vec<_>>()
            //             .join(", ")
            //     ),
            // }
            ]),
            _ => None,
        };

        let warning_level: WarningLevel = e.into();
        Diagnostic {
            range: (&range).into(),
            severity: Some(warning_level.into()),
            code: None,
            code_description: None,
            source: None,
            message: e.long_description(),
            related_information: related,
            tags: None,
            data: None,
        }
    }
}

// TODO COMPLETION ITEMS
// X0 - X32
//
// write enums for registers from X0 to X32

/* TODO extended completion and hover information

I would like to implement completion based on the type of instruction.
For example, if I write "add", the next completion item should be a register.

Labels should be differentiated from function labels.

Each instruction should have a hover item that shows the syntax of the instruction,
and "what" the instruction is doing to registers.

Aliases and long names should be in descriptions, so they pick up on completion.
For example, modulus, remainder, mod and rem should all be completion items for
the "rem" instruction.

Each instruction should have a hover item that shows function that it is part of,
as well as their inputs and outputs.

Stack store/restores could have a hover item that displays the always known item

 */
