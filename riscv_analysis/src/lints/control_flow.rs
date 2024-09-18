use crate::{
    cfg::Cfg,
    parser::ParserNode,
    passes::{LintError, LintPass},
};
use std::rc::Rc;

// Check if you can enter a function through the first line of code
// Check if you can enter a function through a jump (a previous exists)
// Check if any code has no previous (except for the first line of code)
// TODO fix for program entry
pub struct FunctionControlFlowCheck;
impl LintPass for FunctionControlFlowCheck {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
        for node in &cfg.clone() {
            match node.node() {
                ParserNode::FuncEntry(_) => {
                    // If the previous nodes set is not empty
                    // Note: this also accounts for functions being at the beginning
                    // of a program, as the ProgEntry node will be the previous node
                    if let Some(prev_node) = node.prevs().iter().next() {
                        for function in node.functions().iter() {
                            if prev_node.node().is_program_entry() {
                                errors.push(LintError::FirstInstructionIsFunction(
                                    node.node().clone(),
                                    Rc::clone(function),
                                ));
                            } else {
                                errors.push(LintError::InvalidJumpToFunction(
                                    node.node().clone(),
                                    prev_node.node().clone(),
                                    Rc::clone(function),
                                ));
                            }

                        }
                    }
                }
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
