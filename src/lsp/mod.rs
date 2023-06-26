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
            InvalidUseAfterCall(_, _label) => Some(vec![
            //     DiagnosticRelatedInformation {
            //     location: lsp_types::Location {
            //         uri: lsp_types::Url::parse("file:///"),
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
