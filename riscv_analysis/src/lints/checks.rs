use crate::analysis::AvailableValue;
use crate::analysis::HasGenKillInfo;
use crate::cfg::Cfg;
use crate::cfg::CfgNode;
use crate::parser::InstructionProperties;
use crate::parser::Register;
use crate::parser::RegisterToken;
use crate::passes::DiagnosticManager;
use crate::passes::LintError;
use crate::passes::LintPass;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

// If we need to add an error to a register at its first use/store, we need to
// know their ranges. This function will take a register and return the ranges
// that need to be annotated. If it cannot find any, then it will return the original
// node's range.
impl Cfg {
    /// Perform a backwards search to find the first node that stores to the given register.
    ///
    /// From a given end point, like a return value, find the first node that stores to the given register.
    /// This function works by traversing the previous nodes until it finds a node that stores to the given register.
    /// This is used to correctly mark up the first store to a register that might
    /// have been incorrect.
    pub fn error_ranges_for_first_store(node: &Rc<CfgNode>, item: Register) -> Vec<RegisterToken> {
        let mut queue = VecDeque::new();
        let mut ranges = Vec::new();
        // push the previous nodes onto the queue
        queue.extend(node.prevs().clone());

        // keep track of visited nodes
        #[allow(clippy::mutable_key_type)]
        let mut visited = HashSet::new();
        visited.insert(Rc::clone(node));

        // visit each node in the queue
        // if the error is found, add error
        // if not, add the previous nodes to the queue
        while let Some(prev) = queue.pop_front() {
            if visited.contains(&prev) {
                continue;
            }
            visited.insert(Rc::clone(&prev));
            if let Some(reg) = prev.writes_to() {
                if *reg.get() == item {
                    ranges.push(reg);
                    continue;
                }
            }
            queue.extend(prev.prevs().clone().into_iter());
        }
        ranges
    }

    // TODO move to a more appropriate place
    // TODO make better, what even is this?
    pub fn error_ranges_for_first_usage(node: &Rc<CfgNode>, item: Register) -> Vec<RegisterToken> {
        let mut queue = VecDeque::new();
        let mut ranges = Vec::new();
        // push the next nodes onto the queue

        queue.extend(node.nexts().clone());

        // keep track of visited nodes
        #[allow(clippy::mutable_key_type)]
        let mut visited = HashSet::new();
        visited.insert(Rc::clone(node));

        // visit each node in the queue
        // if the error is found, add error
        // if not, add the next nodes to the queue
        while let Some(next) = queue.pop_front() {
            if visited.contains(&next) {
                continue;
            }
            visited.insert(Rc::clone(&next));
            if next.gen_reg().contains(&item) {
                // find the use
                let regs = next.reads_from();
                let mut it = None;
                for reg in regs {
                    if reg == item {
                        it = Some(reg);
                        break;
                    }
                }
                if let Some(reg) = it {
                    ranges.push(reg);
                    break;
                }
                break;
            }

            queue.extend(next.nexts().clone().into_iter());
        }
        ranges
    }
}

// Checks are passes that occur after the CFG is built. As much data as possible is collected
// during the CFG build. Then, the data is applied via a check.

pub struct SaveToZeroCheck;
impl LintPass for SaveToZeroCheck {
    fn run(cfg: &Cfg, errors: &mut DiagnosticManager) {
        for node in cfg {
            if let Some(register) = node.writes_to() {
                if register == Register::X0 && !node.can_skip_save_checks() {
                    errors.push(LintError::SaveToZero(register.clone()));
                }
            }
        }
    }
}

// Check that we know the stack position at every point in the program (aka. within scopes)
pub struct StackCheckPass;
impl LintPass for StackCheckPass {
    fn run(cfg: &Cfg, errors: &mut DiagnosticManager) {
        // PASS 1
        // check that we know the stack position at every point in the program
        // check that the stack is never in an invalid position
        // TODO check that the stack stores always happen to a place that is negative
        // TODO move to impl methods
        'outer: for node in cfg {
            let values = node.reg_values_out();
            match values.get(&Register::X2) {
                None => {
                    errors.push(LintError::UnknownStack(node.node()));
                    break 'outer;
                }
                Some(x) => {
                    if let AvailableValue::OriginalRegisterWithScalar(reg, off) = x {
                        if reg != &Register::X2 {
                            errors.push(LintError::InvalidStackPointer(node.node()));
                            break 'outer;
                        }
                        if off > &0 {
                            errors.push(LintError::InvalidStackPosition(node.node(), *off));
                            break 'outer;
                        }

                        if let Some((reg2, off2)) = node.uses_memory_location() {
                            if reg2 == Register::X2 && off2.value() + off >= 0 {
                                errors.push(LintError::InvalidStackOffsetUsage(
                                    node.node().clone(),
                                    off2.value() + off,
                                ));
                            }
                        }
                    } else {
                        errors.push(LintError::InvalidStackPointer(node.node()));
                        break 'outer;
                    }
                }
            }
            if let Some((reg, _)) = node.uses_memory_location() {
                if reg == Register::X2 {}
            }
        }

        // PASS 2: check that
    }
}

// TODO check if the stack is ever stored at 0 or what not
