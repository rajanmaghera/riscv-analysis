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
