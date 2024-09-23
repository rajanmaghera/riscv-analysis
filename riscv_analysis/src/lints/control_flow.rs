use crate::{
    cfg::Cfg,
    parser::ParserNode,
    passes::{LintError, LintPass},
};
use std::rc::Rc;

// TODO fix for program entry

/// This pass checks for the following control flow issues:
/// - A function is entered through the first line of code (Why?).
/// - A function is entered through an jump that is not a function call.
/// - Any code that has no previous nodes, i.e. is unreachable.
pub struct ControlFlowCheck;
impl LintPass for ControlFlowCheck {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
        for node in &cfg.clone() {
            match node.node() {
                ParserNode::FuncEntry(_) => {
                    // If the previous nodes set is not empty
                    // Note: this also accounts for functions being at the beginning
                    // of a program, as the ProgEntry node will be the previous node
                    for prev_node in node.prevs().iter() {
                        for function in node.functions().iter() {
                            if prev_node.node().is_program_entry() {
                                errors.push(LintError::FirstInstructionIsFunction(
                                    node.node().clone(),
                                    Rc::clone(function),
                                ));
                            }

                            // Jumps (J not JAL) to the start of recognized
                            // functions are errors
                            else if prev_node.node().is_unconditional_jump() {
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
                }
                // The program entry should have no prevs
                ParserNode::ProgramEntry(_) => {}
                _ => {
                    if node.prevs().is_empty() {
                        errors.push(LintError::UnreachableCode(node.node().clone()));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::RVStringParser;
    use crate::passes::{LintError, LintPass, Manager};

    fn run_pass(input: &str) -> Vec<LintError> {
        let (nodes, error) = RVStringParser::parse_from_text(input);
        assert_eq!(error.len(), 0);

        let cfg = Manager::gen_full_cfg(nodes).unwrap();
        ControlFlowCheck::run_single_pass_along_cfg(&cfg)
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
        assert!(matches!(
            &lints[0], LintError::FirstInstructionIsFunction(node, _)
                if node.token().text == "addi a0 a0 1"
            )
        );

        // Next four errors should be about unreachable code
        assert!(matches!(
            &lints[1], LintError::UnreachableCode(node, ..)
                if node.token().text == "li a0 0"
            )
        );
        assert!(matches!(
            &lints[2], LintError::UnreachableCode(node, ..)
                if node.token().text == "jal fn_a"
            )
        );
        assert!(matches!(
            &lints[3], LintError::UnreachableCode(node, ..)
                if node.token().text == "addi a7 zero 10"
            )
        );
        assert!(matches!(
            &lints[4], LintError::UnreachableCode(node, ..)
                if node.token().text == "ecall"
            )
        );
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

        assert!(matches!(
            &lints[0], LintError::UnreachableCode(node, ..)
                if node.token().text == "addi a7 zero 10"
            )
        );
        assert!(matches!(
            &lints[1], LintError::UnreachableCode(node, ..)
                if node.token().text == "ecall"
            )
        );
        assert!(matches!(
            &lints[2], LintError::InvalidJumpToFunction(node, ..)
                if node.token().text == "addi a0 a0 1"
            )
        );
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
}
