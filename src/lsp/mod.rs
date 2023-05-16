// Type conversions for LSP

use std::convert::From;

use crate::parser::token::Range as MyRange;
use crate::passes::PassError::*;
use crate::passes::{PassError, WarningLevel};
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
        };

        let warning_level: WarningLevel = (&e).into();
        Diagnostic {
            range: range.into(),
            severity: Some(warning_level.into()),
            code: None,
            code_description: None,
            source: None,
            message: e.long_description(),
            related_information: None,
            tags: None,
            data: None,
        }
    }
}

// COMPLETION ITEMS
// X0 - X32
//
// write enums for registers from X0 to X32
