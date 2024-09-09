use crate::{
    cfg::{Cfg, Segment},
    passes::{LintError, LintPass},
};

/// A lint to ensure that instructions only exist in the text
/// segment.
///
/// Instructions will only be assembled if they appear in
/// the text segment. Instructions in other locations is
/// behaviour that we do not handle.
pub struct InstructionInTextCheck;
impl LintPass for InstructionInTextCheck {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
        for node in &cfg.clone() {
            if node.node().is_instruction() && node.segment() != Segment::Text {
                errors.push(LintError::InvalidSegment(node.node().clone()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{arith, directive, iarith};

    #[test]
    fn default_segment_is_text() {
        let nodes = &[iarith!(Addi X1 X0 0)];
        let errors = InstructionInTextCheck::run_single_pass_along_nodes(nodes);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn explicit_text_segment_is_allowed() {
        let nodes = &[directive!(Text, TextSection), iarith!(Addi X1 X0 0)];
        let errors = InstructionInTextCheck::run_single_pass_along_nodes(nodes);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn can_get_error_for_data_segment() {
        let nodes = &[
            iarith!(Addi X1 X0 0),
            arith!(Add X1 X0 X20),
            directive!(Data, DataSection),
            iarith!(Addi X1 X0 0),
            arith!(Sub X1 X0 X20),
            directive!(Text, TextSection),
            iarith!(Andi X1 X0 0),
        ];
        let errors = InstructionInTextCheck::run_single_pass_along_nodes(nodes);
        assert_eq!(errors.len(), 2);
        assert!(matches!(errors[0], LintError::InvalidSegment(_)));
        assert!(matches!(errors[1], LintError::InvalidSegment(_)));
    }

    #[test]
    fn can_get_error_if_data_segment_is_first() {
        let nodes = &[
            directive!(Data, DataSection),
            iarith!(Addi X1 X0 0),
            arith!(Add X1 X0 X20),
            iarith!(Addi X1 X0 0),
            directive!(Text, TextSection),
            iarith!(Addi X1 X0 0),
            arith!(Sub X1 X0 X20),
            directive!(Text, TextSection),
            directive!(Data, DataSection),
            directive!(Text, TextSection),
            iarith!(Andi X1 X0 0),
        ];
        let errors = InstructionInTextCheck::run_single_pass_along_nodes(nodes);
        assert_eq!(errors.len(), 3);
        assert!(matches!(errors[0], LintError::InvalidSegment(_)));
        assert!(matches!(errors[1], LintError::InvalidSegment(_)));
        assert!(matches!(errors[2], LintError::InvalidSegment(_)));
    }
}
