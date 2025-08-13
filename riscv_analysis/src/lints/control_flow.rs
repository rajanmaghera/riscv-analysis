use crate::cfg::Segment;
use crate::{
    cfg::Cfg,
    parser::InstructionProperties,
    passes::{DiagnosticBuilder, DiagnosticManager, LintError, LintPass},
};
use std::rc::Rc;
// TODO fix for program entry

/// This pass checks for the following control flow issues:
/// - A function is entered through the first line of code (Why?).
/// - A function is entered through an jump that is not a function call.
/// - Any code that has no previous nodes, i.e. is unreachable.
#[non_exhaustive]
pub struct ControlFlowPass;
impl ControlFlowPass {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ControlFlowPass {
    fn default() -> Self {
        Self::new()
    }
}

impl LintPass for ControlFlowPass {
    fn get_pass_name(&self) -> &'static str {
        "control-flow"
    }
    fn run(&self, cfg: &Cfg, errors: &mut DiagnosticManager) {
        for node in cfg.iter_source() {
            if node.is_function_entry() {
                for function in node.functions().iter() {
                    // If the previous nodes set is not empty
                    // Note: this also accounts for functions being at the beginning
                    // of a program, as the ProgEntry node will be the previous node
                    if node.is_program_entry() {
                        errors.push(LintError::FirstInstructionIsFunction(
                            node.node().clone(),
                            Rc::clone(function),
                        ));
                        continue;
                    }
                    for prev_node in cfg.get_prevs(node.as_ref()) {
                        // Jumps (J not JAL) to the start of recognized
                        // functions are errors
                        if prev_node.is_unconditional_jump() {
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
            } else if !node.is_program_entry()
                && cfg.get_prevs(node.as_ref()).len() == 0
                && node.segment() == Segment::Text
            {
                errors.push(
                    DiagnosticBuilder::new("unreachable-code", "Unreachable line of code")
                        .description("There is no path to this instruction.")
                        .is_warning_on(node.as_ref()),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::parser::{ProgramEntryType, RVStringParser};
    use crate::passes::Manager;

    fn run_pass(input: &str) -> DiagnosticManager {
        let parser_output = RVStringParser::parse_from_text(input);
        assert_eq!(parser_output.errors.len(), 0);
        let cfg = Manager::gen_full_cfg(&parser_output, None, &ProgramEntryType::FirstInstruction)
            .unwrap();
        ControlFlowPass::new().run_single_pass_along_cfg(&cfg)
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
        assert_eq!(lints.len(), 2);

        // The first error should warn about the first instruction of `fn_a`

        assert_eq!(lints[0].get_error_code(), "first-instruction-is-function");
        assert_eq!(lints[0].raw_text(), "addi   a0, a0, 1");

        // Next error should be about unreachable code
        assert_eq!(lints[1].get_error_code(), "unreachable-code");
        assert_eq!(lints[1].raw_text(), "li     a0, 0");
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
        assert_eq!(lints.len(), 2);

        // The first error should warn about the first instruction of `fn_a`
        assert_eq!(lints[0].get_error_code(), "unreachable-code");
        assert_eq!(lints[0].raw_text(), "addi   a7, zero, 10");

        assert_eq!(lints[1].get_error_code(), "invalid-jump-to-function");
        assert_eq!(lints[1].raw_text(), "addi   a0, a0, 1");
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
