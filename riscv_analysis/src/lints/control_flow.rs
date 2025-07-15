use crate::{
    cfg::Cfg,
    parser::InstructionProperties,
    passes::{DiagnosticBuilder, DiagnosticManager, LintError, LintPass, PassConfiguration},
};
use std::rc::Rc;

// TODO fix for program entry

/// This pass checks for the following control flow issues:
/// - A function is entered through the first line of code (Why?).
/// - A function is entered through an jump that is not a function call.
/// - Any code that has no previous nodes, i.e. is unreachable.
pub struct ControlFlowPass;
impl LintPass<ControlFlowPassConfiguration> for ControlFlowPass {
    fn run(cfg: &Cfg, errors: &mut DiagnosticManager, config: &ControlFlowPassConfiguration) {
        if !config.get_enabled() {
            return;
        }
        for node in &cfg.clone() {
            if node.is_function_entry() {
                // If the previous nodes set is not empty
                // Note: this also accounts for functions being at the beginning
                // of a program, as the ProgEntry node will be the previous node
                for prev_node in node.prevs().iter() {
                    for function in node.functions().iter() {
                        if prev_node.is_program_entry() {
                            errors.push(LintError::FirstInstructionIsFunction(
                                node.node().clone(),
                                Rc::clone(function),
                            ));
                        }
                        // Jumps (J not JAL) to the start of recognized
                        // functions are errors
                        else if prev_node.is_unconditional_jump() {
                            errors.push(LintError::InvalidJumpToFunction(
                                node.node().clone(),
                                prev_node.node().clone(),
                                Rc::clone(function),
                            ));
                            // Create at most one error per node
                            break;
                        }
                    }
                }
            } else if !node.is_program_entry() && node.prevs().is_empty() {
                errors.push_real(
                    DiagnosticBuilder::new("unreachable-code", "Unreachable line of code")
                        .description("There is no path to this instruction.")
                        .is_warning_on(node),
                );
            }
        }
    }
}
pub struct ControlFlowPassConfiguration {
    /// Is the pass enabled?
    enabled: bool,
}
impl Default for ControlFlowPassConfiguration {
    fn default() -> Self {
        ControlFlowPassConfiguration { enabled: true }
    }
}
impl PassConfiguration for ControlFlowPassConfiguration {
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
    use crate::parser::RVStringParser;
    use crate::passes::Manager;

    fn run_pass(input: &str) -> DiagnosticManager {
        let (nodes, error) = RVStringParser::parse_from_text(input);
        assert_eq!(error.len(), 0);

        let cfg = Manager::gen_full_cfg(nodes).unwrap();
        let mut config = ControlFlowPassConfiguration::default();
        config.set_enabled(true);
        ControlFlowPass::run_single_pass_along_cfg(&cfg, &config)
    }

    #[test]
    fn function_on_first_line() {
        let input = "\
            fn_a:                      \n\
                addi   a0, a0, 1       \n\
                ret                    \n\
            main:                      \n\
                li     a0, 0           \n\
                jal    fn_a            \n\
                addi   a7, zero, 10    \n\
                ecall                  \n";

        let lints = run_pass(input);

        // Error for function on at the program entry & 4 errors for all the
        // unreachable instructions in `main`
        assert_eq!(lints.len(), 5);

        // The first error should warn about the first instruction of `fn_a`

        assert_eq!(lints[0].get_error_code(), "first-instruction-is-function");
        assert_eq!(lints[0].raw_text(), "addi a0 a0 1");

        // Next four errors should be about unreachable code
        assert_eq!(lints[1].get_error_code(), "unreachable-code");
        assert_eq!(lints[1].raw_text(), "li a0 0");

        assert_eq!(lints[2].get_error_code(), "unreachable-code");
        assert_eq!(lints[2].raw_text(), "jal fn_a");

        assert_eq!(lints[3].get_error_code(), "unreachable-code");
        assert_eq!(lints[3].raw_text(), "addi a7 zero 10");

        assert_eq!(lints[4].get_error_code(), "unreachable-code");
        assert_eq!(lints[4].raw_text(), "ecall");
    }

    #[test]
    fn jump_to_function() {
        let input = "\
            main:                      \n\
                li     a0, 0           \n\
                jal    fn_a            \n\
                j      fn_a            \n\
                addi   a7, zero, 10    \n\
                ecall                  \n\
            fn_a:                      \n\
                addi   a0, a0, 1       \n\
                ret                    \n\
            ";

        let lints = run_pass(input);

        // Error for function on at the program entry & 2 errors for all the
        // unreachable instructions in `main` after the `j` instruction
        assert_eq!(lints.len(), 3);

        // The first error should warn about the first instruction of `fn_a`
        assert_eq!(lints[0].get_error_code(), "unreachable-code");
        assert_eq!(lints[0].raw_text(), "addi a7 zero 10");

        assert_eq!(lints[1].get_error_code(), "unreachable-code");
        assert_eq!(lints[1].raw_text(), "ecall");

        assert_eq!(lints[2].get_error_code(), "invalid-jump-to-function");
        assert_eq!(lints[2].raw_text(), "addi a0 a0 1");
    }

    #[test]
    fn overlapping_functions() {
        let input = "\
            main:                      \n\
                li     a0, 0           \n\
                jal    fn_a            \n\
                jal    fn_b            \n\
                addi   a7, zero, 10    \n\
                ecall                  \n\
            fn_a:                      \n\
                addi   a0, a0, 1       \n\
            fn_b:                      \n\
                addi   a0, a0, 2       \n\
                ret                    \n";

        let lints = run_pass(input);

        // Overlapping functions should not cause a control flow error
        assert_eq!(lints.len(), 0);
    }

    #[test]
    fn unreachable_directive() {
        let input = "\
            .text                      \n\
            main:                      \n\
                jal     fn_a           \n\
                la      a0, bytes      \n\
                lw      a0, 0(a0)      \n\
                addi    a7, zero, 10   \n\
                ecall                  \n\
            fn_a:                      \n\
                addi    a0, a0, 1      \n\
                ret                    \n\
            .data                      \n\
            bytes:   .space 10         \n";

        let lints = run_pass(input);

        // An "unreachable" directive shouldn't cause an error
        assert_eq!(lints.len(), 0);
    }

    #[test]
    fn immediate_exit_of_code() {
        let input = "\
            main:       \n\
            li a7, 10   \n\
            ecall       \n";
        let errors = run_pass(input);
        assert_eq!(errors.len(), 0);
    }
}
