use crate::cfg::Function;
use crate::parser::{LabelStringToken, RVInstructionNode};
use std::rc::Rc;
use std::{collections::HashSet, fmt::Display};

use super::{DiagnosticLocation, DiagnosticMessage, SeverityLevel};

#[derive(Debug, Clone)]
// TODO CfgErrors that do not require the whole thing to be re-run

/// `CfgError` is an error that occurs while generating an annotated CFG.
///
/// These errors are non-recoverable and will cause the program to exit at
/// the point of error. As much effort should be done to avoid these errors
/// and to use `LintErrors`, as those are recoverable.
pub enum CfgError {
    /// This error occurs when a label is used but not defined.
    LabelsNotDefined(HashSet<LabelStringToken>),
    /// This error occurs when a label is defined more than once.
    DuplicateLabel(LabelStringToken),
    /// Node in two functions
    NodeInTwoFunctions(RVInstructionNode, Rc<Function>, Rc<Function>),
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

impl Display for CfgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CfgError::LabelsNotDefined(labels) => {
                write!(f, "Labels not defined: {}", labels.as_str_list())
            }
            CfgError::DuplicateLabel(label) => {
                write!(f, "Duplicate label: {label}")
            }
            CfgError::NodeInTwoFunctions(_, _, _) => {
                write!(f, "Node in two functions")
            }
        }
    }
}

impl From<&CfgError> for SeverityLevel {
    fn from(value: &CfgError) -> Self {
        match value {
            CfgError::LabelsNotDefined(_)
            | CfgError::DuplicateLabel(_)
            | CfgError::NodeInTwoFunctions(_, _, _) => SeverityLevel::Error,
        }
    }
}

impl DiagnosticLocation for CfgError {
    fn file(&self) -> uuid::Uuid {
        match self {
            CfgError::NodeInTwoFunctions(node, _, _) => node.file(),
            CfgError::LabelsNotDefined(labels) => labels.iter().next().unwrap().file(),
            CfgError::DuplicateLabel(label) => label.file(),
        }
    }

    fn range(&self) -> crate::parser::Range {
        match self {
            CfgError::NodeInTwoFunctions(node, _, _) => node.range(),
            CfgError::LabelsNotDefined(labels) => labels.iter().next().unwrap().range(),
            CfgError::DuplicateLabel(label) => label.range(),
        }
    }

    fn raw_text(&self) -> String {
        match self {
            CfgError::NodeInTwoFunctions(node, _, _) => node.raw_text(),
            CfgError::LabelsNotDefined(labels) => labels.iter().next().unwrap().raw_text(),
            CfgError::DuplicateLabel(label) => label.raw_text(),
        }
    }
}

impl DiagnosticMessage for CfgError {
    fn related(&self) -> Option<Vec<super::RelatedDiagnosticItem>> {
        None
    }

    fn level(&self) -> SeverityLevel {
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
            CfgError::NodeInTwoFunctions(_, f1, f2) => format!(
                "The instruction was found to be part of two functions: one that begins on line {} and one that begins on line {}. Each instruction must be part of only one function",
                f1.entry().range().start().one_idx_line(),
                f2.entry().range().start().one_idx_line()
            ),
            CfgError::DuplicateLabel(label) => format!(
                "The label {label} is defined more than once. Labels must be unique."
            ),
            CfgError::LabelsNotDefined(labels) => format!(
                "The labels {} are used but not defined. Labels must be defined within your file.",
                labels.as_str_list()
            ),
        }
    }
}
