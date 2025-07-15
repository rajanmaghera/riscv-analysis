use crate::{
    cfg::{Cfg, Segment},
    parser::InstructionProperties,
    passes::{DiagnosticManager, LintError, LintPass, PassConfiguration},
};

/// A lint to ensure that instructions only exist in the text
/// segment.
///
/// Instructions will only be assembled if they appear in
/// the text segment. Instructions in other locations is
/// behaviour that we do not handle.
pub struct InstructionInTextPass;
impl LintPass<InstructionInTextPassConfiguration> for InstructionInTextPass {
    fn run(cfg: &Cfg, errors: &mut DiagnosticManager, config: &InstructionInTextPassConfiguration) {
        if !config.get_enabled() {
            return;
        }
        for node in cfg {
            if node.is_instruction() && node.segment() != Segment::Text {
                errors.push(LintError::InvalidSegment(node.node().clone()));
            }
        }
    }
}
#[derive(Default)]
pub struct InstructionInTextPassConfiguration {
    /// Is the pass enabled?
    enabled: bool,
}
impl PassConfiguration for InstructionInTextPassConfiguration {
    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{arith, directive, iarith};

    #[test]
    fn default_segment_is_text() {
        let nodes = &[iarith!(Addi X1 X0 0)];
        let mut config = InstructionInTextPassConfiguration::default();
        config.set_enabled(true);
        let errors = InstructionInTextPass::run_single_pass_along_nodes(nodes, &config);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn explicit_text_segment_is_allowed() {
        let nodes = &[directive!(Text, TextSection), iarith!(Addi X1 X0 0)];
        let mut config = InstructionInTextPassConfiguration::default();
        config.set_enabled(true);
        let errors = InstructionInTextPass::run_single_pass_along_nodes(nodes, &config);
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
        let mut config = InstructionInTextPassConfiguration::default();
        config.set_enabled(true);
        let errors = InstructionInTextPass::run_single_pass_along_nodes(nodes, &config);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].get_error_code(), "invalid-segment");
        assert_eq!(errors[1].get_error_code(), "invalid-segment");
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
        let mut config = InstructionInTextPassConfiguration::default();
        config.set_enabled(true);
        let errors = InstructionInTextPass::run_single_pass_along_nodes(nodes, &config);
        assert_eq!(errors.len(), 3);
        assert_eq!(errors[0].get_error_code(), "invalid-segment");
        assert_eq!(errors[1].get_error_code(), "invalid-segment");
        assert_eq!(errors[2].get_error_code(), "invalid-segment");
    }
}
