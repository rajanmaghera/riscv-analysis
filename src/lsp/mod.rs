// Type conversions for LSP

use std::borrow::Borrow;
use std::convert::From;

use crate::parser::token::Range as MyRange;
use crate::passes::PassError::*;
use crate::passes::{PassError, WarningLevel};
use lsp_types::{Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Position, Range};

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
            WarningLevel::Suggestion => DiagnosticSeverity::INFORMATION,
            WarningLevel::Warning => DiagnosticSeverity::WARNING,
            WarningLevel::Error => DiagnosticSeverity::ERROR,
        }
    }
}

impl From<PassError> for Diagnostic {
    fn from(e: PassError) -> Self {
        let range = match &e {
            SaveToZero(r) => r,
            DeadAssignment(r) => r,
            InvalidUseAfterCall(r, _) => r,
            JumpToFunc(r, _) => r,
            NaturalFuncEntry(r) => r,
        };
        let related = match &e {
            InvalidUseAfterCall(_, label) => Some(vec![DiagnosticRelatedInformation {
                location: lsp_types::Location {
                    uri: lsp_types::Url::parse("file:///").unwrap(),
                    range: label.pos.borrow().into(),
                },
                message: format!(
                    "The function call to [{}] invalidates any temporary registers afterwards.",
                    label.data.0
                ),
            }]),
            _ => None,
        };

        let warning_level: WarningLevel = (&e).into();
        Diagnostic {
            range: range.into(),
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

// COMPLETION ITEMS
// X0 - X32
//
// write enums for registers from X0 to X32
