use std::collections::HashSet;

use crate::parser::{LabelString, Node, With};

#[derive(Debug)]

// TODO try best to move as many to linterrors, rather than cfg errors
pub enum CFGError {
    LabelsNotDefined(HashSet<With<LabelString>>),
    DuplicateLabel(With<LabelString>),
    MultipleLabelsForReturn(Node, HashSet<With<LabelString>>),
    ReturnFromProgramStart(Node),
    NoLabelForReturn(Node),
    MultipleReturnsForLabel(HashSet<With<LabelString>>, HashSet<Node>),
    UnexpectedError,
    AssertionError,
}
