use std::collections::HashSet;

use crate::parser::{LabelString, ParserNode, With};

#[derive(Debug)]

/// CFGError is an error that occurs while generating an annotated CFG.
///
/// These errors are non-recoverable and will cause the program to exit at
/// the point of error. As much effort should be done to avoid these errors
/// and to use LintErrors, as those are recoverable.
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
    /// This error occurs when there are multiple returns for a label.
    ///
    /// This error is temporary and will be removed from a future version of
    /// the compiler.
    MultipleReturnsForLabel(HashSet<With<LabelString>>, HashSet<ParserNode>),
    /// Unexpected error
    UnexpectedError,
    /// Assertion error
    AssertionError,
}
