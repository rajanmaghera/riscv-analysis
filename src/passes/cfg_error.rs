use std::{collections::HashSet, fmt::Display};

use crate::parser::{LabelString, ParserNode, With};

use super::{DiagnosticLocation, DiagnosticMessage, WarningLevel};

#[derive(Debug, Clone)]
// TODO CFGErrors that do not require the whole thing to be re-run

/// `CFGError` is an error that occurs while generating an annotated CFG.
///
/// These errors are non-recoverable and will cause the program to exit at
/// the point of error. As much effort should be done to avoid these errors
/// and to use `LintErrors`, as those are recoverable.
pub enum CFGError {
    /// This error occurs when a label is used but not defined.
    LabelsNotDefined(HashSet<With<LabelString>>),
    /// This error occurs when a label is defined more than once.
    DuplicateLabel(With<LabelString>),
    /// This error occurs when a return statement is used but can be reached by
    /// multiple labels.
    MultipleLabelsForReturn(ParserNode, HashSet<With<LabelString>>),
    /// This error occurs when a return statement is used but can be reached by
    /// no labels.
    NoLabelForReturn(ParserNode),
    /// Unexpected error
    UnexpectedError,
    /// Assertion error
    AssertionError,
}

trait SetListString {
    fn as_str_list(&self) -> String;
}

impl<T> SetListString for HashSet<T>
where
    T: Display + Ord,
{
    fn as_str_list(&self) -> String {
        let mut vec = self.iter().collect::<Vec<_>>();
        vec.sort();
        vec.iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Display for CFGError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CFGError::LabelsNotDefined(labels) => {
                write!(f, "Labels not defined: {}", labels.as_str_list())
            }
            CFGError::DuplicateLabel(label) => {
                write!(f, "Duplicate label: {label}")
            }
            CFGError::MultipleLabelsForReturn(_, labels) => {
                write!(f, "Multiple labels for return: {}", labels.as_str_list())
            }
            CFGError::NoLabelForReturn(_) => {
                write!(f, "No label for return")
            }
            CFGError::UnexpectedError => write!(f, "Unexpected error"),
            CFGError::AssertionError => write!(f, "Assertion error"),
        }
    }
}

impl From<&CFGError> for WarningLevel {
    fn from(value: &CFGError) -> Self {
        match value {
            CFGError::LabelsNotDefined(_)
            | CFGError::DuplicateLabel(_)
            | CFGError::MultipleLabelsForReturn(_, _)
            | CFGError::NoLabelForReturn(_)
            | CFGError::UnexpectedError
            | CFGError::AssertionError => WarningLevel::Error,
        }
    }
}

impl DiagnosticLocation for CFGError {
    fn file(&self) -> uuid::Uuid {
        match self {
            CFGError::MultipleLabelsForReturn(node, _) => node.file(),
            CFGError::NoLabelForReturn(node) => node.file(),
            CFGError::LabelsNotDefined(labels) => labels.iter().next().unwrap().file(),
            CFGError::DuplicateLabel(label) => label.file(),
            CFGError::UnexpectedError => uuid::Uuid::nil(),
            CFGError::AssertionError => uuid::Uuid::nil(),
        }
    }

    fn range(&self) -> crate::parser::Range {
        match self {
            CFGError::MultipleLabelsForReturn(node, _) => node.range(),
            CFGError::NoLabelForReturn(node) => node.range(),
            CFGError::LabelsNotDefined(labels) => labels.iter().next().unwrap().range(),
            CFGError::DuplicateLabel(label) => label.range(),
            CFGError::UnexpectedError => crate::parser::Range::default(),
            CFGError::AssertionError => crate::parser::Range::default(),
        }
    }
}

impl DiagnosticMessage for CFGError {
    fn related(&self) -> Option<Vec<super::RelatedDiagnosticItem>> {
        None
    }

    fn level(&self) -> WarningLevel {
        self.into()
    }
    fn title(&self) -> String {
        self.to_string()
    }
    fn description(&self) -> String {
        self.long_description()
    }
    fn long_description(&self) -> String {
        match self {
            CFGError::DuplicateLabel(label) => format!(
                "The label {label} is defined more than once. Labels must be unique."
            ),
            CFGError::LabelsNotDefined(labels) => format!(
                "The labels {} are used but not defined. Labels must be defined within your file.",
                labels.as_str_list()
            ),
            CFGError::MultipleLabelsForReturn(_, labels) => format!(
                "The return statement can be reached by multiple function labels: {}.\n\n\
                Every return statement should only be reachable by one label. This also ensures\
                that every instruction is reachable by only one label and is ever only part of a single function.\n\n\
                Your code might contain instructions that allow two functions to reach this return statement.\
                You might also jump from one function to another. You can fix this by ensuring all code for a function\
                is only reachable by one label. For example, replace this return statement with two or more return statements\
                for each function.",
                labels.as_str_list()
            ),
            CFGError::NoLabelForReturn(_) => "The return statement can be reached by no function labels.\n\n\
                Every return statement should be reachable by one label. This also ensures\
                that every instruction is reachable by only one label and is ever only part of a single function.\n\n\
                This return statement might be placed in code that isn't in a function. For example, you might have a
                return statement that is in the 'main' segment of your code. To fix this, remove the return statement or
                place it in a function.\n\n\
                A label is considered a function if it has been called by a [jal] instruction. This code might also be\
                missing from your file or imports.
                ".to_string(),
            CFGError::UnexpectedError => "An unexpected error occurred. Please file a bug.".to_string(),
            CFGError::AssertionError => "An unexpected assertion error occurred. Please file a bug.".to_string(),
        }
    }
}
