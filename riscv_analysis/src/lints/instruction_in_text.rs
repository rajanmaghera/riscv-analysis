use crate::{
    cfg::{Cfg, Segment},
    parser::InstructionProperties,
    passes::{DiagnosticManager, LintError, LintPass},
};

/// A lint to ensure that instructions only exist in the text
/// segment.
///
/// Instructions will only be assembled if they appear in
/// the text segment. Instructions in other locations is
/// behaviour that we do not handle.
#[non_exhaustive]
pub struct InstructionInTextPass;
impl InstructionInTextPass {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for InstructionInTextPass {
    fn default() -> Self {
        Self::new()
    }
}

impl LintPass for InstructionInTextPass {
    fn get_pass_name(&self) -> &'static str {
        "instruction-in-text"
    }
    fn run(&self, cfg: &Cfg, errors: &mut DiagnosticManager) {
        for node in cfg.iter_source() {
            if node.is_instruction() && node.segment() != Segment::Text {
                errors.push(LintError::InvalidSegment(node.node().clone()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{RVInstructionNode, RVStringParser};

    fn parse_text(s: &str) -> Vec<RVInstructionNode> {
        let parser_output = RVStringParser::parse_from_text(s);
        dbg!(&parser_output.errors);
        assert!(parser_output.errors.is_empty());
        parser_output.nodes
    }

    #[test]
    fn default_segment_is_text() {
        let nodes = parse_text("addi x1 x0 0");
        let errors = InstructionInTextPass::new().run_single_pass_along_nodes(&nodes);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn explicit_text_segment_is_allowed() {
        let nodes = parse_text(".text\naddi x1 x0 0");
        let errors = InstructionInTextPass::new().run_single_pass_along_nodes(&nodes);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn can_get_error_for_data_segment() {
        let nodes = parse_text(
            "addi x1 x0 0\nadd x1 x0 x20\n.data\naddi x1 x0 0\nsub x1 x0 x20\n.text\naddi x1 x0 0",
        );
        let errors = InstructionInTextPass::new().run_single_pass_along_nodes(&nodes);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].get_error_code(), "invalid-segment");
        assert_eq!(errors[1].get_error_code(), "invalid-segment");
    }

    #[test]
    fn can_get_error_if_data_segment_is_first() {
        let nodes = parse_text(
            ".data\naddi x1 x0 0\nadd x1 x0 x20\naddi x1 x0 0\n.text\naddi x1 x0 0\nsub x1 x0 x20\nandi x1 x0 0",
        );
        let errors = InstructionInTextPass::new().run_single_pass_along_nodes(&nodes);
        assert_eq!(errors.len(), 3);
        assert_eq!(errors[0].get_error_code(), "invalid-segment");
        assert_eq!(errors[1].get_error_code(), "invalid-segment");
        assert_eq!(errors[2].get_error_code(), "invalid-segment");
    }
}
