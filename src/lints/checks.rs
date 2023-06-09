use crate::analysis::AvailableRegisterValues;
use crate::analysis::AvailableValue;
use crate::analysis::CustomClonedSets;
use crate::cfg::CFGNode;
use crate::cfg::Cfg;
use crate::parser::ParserNode;
use crate::parser::RegSets;
use crate::parser::Register;
use crate::parser::With;
use crate::passes::LintError;
use crate::passes::LintPass;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

// If we need to add an error to a register at its first use/store, we need to
// know their ranges. This function will take a register and return the ranges
// that need to be annotated. If it cannot find any, then it will return the original
// node's range.
impl Cfg {
    // TODO move to a more appropriate place
    // TODO make better, what even is this?
    fn error_ranges_for_first_usage(node: &Rc<CFGNode>, item: Register) -> Vec<With<Register>> {
        let mut queue = VecDeque::new();
        let mut ranges = Vec::new();
        // push the next nodes onto the queue

        queue.extend(node.nexts().clone().into_iter());

        // keep track of visited nodes
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
            if next.node().gen_reg().contains(&item) {
                // find the use
                let regs = next.node().reads_from();
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
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
        for node in &cfg.clone() {
            if let Some(register) = node.node().stores_to() {
                if register == Register::X0 && !node.node().is_return() {
                    errors.push(LintError::SaveToZero(register.clone()));
                }
            }
        }
    }
}

pub struct DeadValueCheck;
impl LintPass for DeadValueCheck {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
        for (_i, node) in cfg.clone().into_iter().enumerate() {
            // check for any assignments that don't make it
            // to the end of the node
            if let Some(def) = node.node().stores_to() {
                if !node.live_out().contains(&def.data) {
                    // TODO dead assignment register

                    errors.push(LintError::DeadAssignment(def));
                }
            }

            // check the out of the node for any uses that
            // should not be there (temporaries)
            // TODO merge with Callee saved register check
            if let Some(name) = node.calls_to(cfg) {
                // check the expected return values of the function:
                let out: HashSet<Register> = RegSets::caller_saved()
                    .difference_c(&name.returns())
                    .intersection_c(&node.live_out());

                // if there is anything left, then there is an error
                // for each item, keep going to the next node until a use of
                // that item is found
                let mut ranges = Vec::new();
                for item in out {
                    ranges.append(&mut Cfg::error_ranges_for_first_usage(&node, item));
                }
                for item in ranges {
                    errors.push(LintError::InvalidUseAfterCall(item, Rc::clone(&name)));
                }
            }
        }
    }
}

// Check if you can enter a function through the first line of code
// Check if you can enter a function through a jump (a previous exists)
// Check if any code has no previous (except for the first line of code)
// TODO fix for program entry
pub struct ControlFlowCheck;
impl LintPass for ControlFlowCheck {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
        for (i, node) in cfg.clone().into_iter().enumerate() {
            match node.node() {
                ParserNode::FuncEntry(_) => {
                    if i == 0 || !node.prevs().is_empty() {
                        if let Some(function) = node.function().clone() {
                            errors
                                .push(LintError::ImproperFuncEntry(node.node().clone(), function));
                        }
                    }
                }
                _ => {
                    if i != 0 && node.prevs().is_empty() {
                        errors.push(LintError::UnreachableCode(node.node().clone()));
                    }
                }
            }
        }
    }
}

// Check if every ecall has a known call number
// Check if there are any instructions after an ecall to terminate the program
pub struct EcallCheck;
impl LintPass for EcallCheck {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
        for (_i, node) in cfg.clone().into_iter().enumerate() {
            if node.node().is_ecall() && node.known_ecall().is_none() {
                errors.push(LintError::UnknownEcall(node.node().clone()));
            }
        }
    }
}

// TODO deprecate
// Check if there are any in values to the start of functions that are not args or saved registers
// Check if there are any in values at the start of a program
pub struct GarbageInputValueCheck;
impl LintPass for GarbageInputValueCheck {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
        for node in &cfg.clone() {
            if node.node().is_program_entry() {
                let mut garbage = node.live_in().clone();
                garbage.retain(|x| !RegSets::saved().contains(x));
                if !garbage.is_empty() {
                    let mut ranges = Vec::new();
                    for reg in garbage {
                        let mut ranges_tmp = Cfg::error_ranges_for_first_usage(&node, reg);
                        ranges.append(&mut ranges_tmp);
                    }
                    for range in ranges {
                        errors.push(LintError::InvalidUseBeforeAssignment(range.clone()));
                    }
                }
            } else if let Some(func) = node.is_function_entry() {
                let args = func.arguments();
                let mut garbage = node.live_in().clone();
                garbage.retain(|reg| !args.contains(reg));
                garbage.retain(|reg| !RegSets::saved().contains(reg));
                if !garbage.is_empty() {
                    let mut ranges = Vec::new();
                    for reg in garbage {
                        let mut ranges_tmp = Cfg::error_ranges_for_first_usage(&node, reg);
                        ranges.append(&mut ranges_tmp);
                    }
                    for range in ranges {
                        errors.push(LintError::InvalidUseBeforeAssignment(range.clone()));
                    }
                }
            }
        }
    }
}

// Check that we know the stack position at every point in the program (aka. within scopes)
pub struct StackCheckPass;
impl LintPass for StackCheckPass {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
        // PASS 1
        // check that we know the stack position at every point in the program
        // check that the stack is never in an invalid position
        // TODO move to impl methods
        'outer: for (_i, node) in cfg.clone().into_iter().enumerate() {
            let values = node.reg_values_out();
            match values.get(&Register::X2) {
                None => {
                    errors.push(LintError::UnknownStack(node.node().clone()));
                    break 'outer;
                }
                Some(x) => {
                    if let AvailableValue::OriginalRegisterWithScalar(reg, off) = x {
                        if reg != &Register::X2 {
                            errors.push(LintError::InvalidStackPointer(node.node().clone()));
                            break 'outer;
                        }
                        if off > &0 {
                            errors.push(LintError::InvalidStackPosition(node.node().clone(), *off));
                            break 'outer;
                        }
                    } else {
                        errors.push(LintError::InvalidStackPointer(node.node().clone()));
                        break 'outer;
                    }
                }
            }
        }

        // PASS 2: check that
    }
}

// check if the value of a calle-saved register is read as its original value
pub struct CalleeSavedGarbageReadCheck;
impl LintPass for CalleeSavedGarbageReadCheck {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
        for (_i, node) in cfg.nodes.clone().into_iter().enumerate() {
            for read in node.node().reads_from() {
                // if the node uses a calle saved register but not a memory access and the value going in is the original value, then we are reading a garbage value
                // DESIGN DECISION: we allow any memory accesses for calle saved registers

                if RegSets::saved().contains(&read.data)
                    && (!node.node().is_memory_access())
                    && node.reg_values_in().is_original_value(read.data)
                {
                    errors.push(LintError::InvalidUseBeforeAssignment(read.clone()));
                    // then we are reading a garbage value
                }
            }
        }
    }
}

pub struct CalleeSavedRegisterCheck;
impl LintPass for CalleeSavedRegisterCheck {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
        use Register::{X1, X18, X19, X2, X20, X21, X22, X23, X24, X25, X26, X27, X8, X9};
        // for all functions
        for func in cfg.label_function_map.values() {
            // TODO scan function to find all "first" definitions of function,
            // then mark those up

            // check if the original values for all calle saved are available at the end
            let val = func.exit.reg_values_in();
            for reg in [
                X1, X2, X8, X9, X18, X19, X20, X21, X22, X23, X24, X25, X26, X27,
            ] {
                match val.get(&reg) {
                    Some(AvailableValue::OriginalRegisterWithScalar(reg2, offset))
                        if reg2 != &reg || offset != &0 =>
                    {
                        errors.push(LintError::OverwriteCalleeSavedRegister(
                            func.exit.node().clone(),
                            reg,
                        ));
                    }
                    _ => {
                        errors.push(LintError::OverwriteCalleeSavedRegister(
                            func.exit.node().clone(),
                            reg,
                        ));
                    }
                }
            }
        }
    }
}
