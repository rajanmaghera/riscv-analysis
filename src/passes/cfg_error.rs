use std::collections::HashSet;

use crate::parser::{LabelString, ParserNode, With};

#[derive(Debug)]

// TODO try best to move as many to linterrors, rather than cfg errors
pub enum CFGError {
    LabelsNotDefined(HashSet<With<LabelString>>),
    DuplicateLabel(With<LabelString>),
    MultipleLabelsForReturn(ParserNode, HashSet<With<LabelString>>),
    ReturnFromProgramStart(ParserNode),
    NoLabelForReturn(ParserNode),
    MultipleReturnsForLabel(HashSet<With<LabelString>>, HashSet<ParserNode>),
    UnexpectedError,
    AssertionError,
}
