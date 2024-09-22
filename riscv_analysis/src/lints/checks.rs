use crate::analysis::AvailableValue;
use crate::cfg::Cfg;
use crate::cfg::CfgNode;
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
    /// Perform a backwards search to find the first node that stores to the given register.
    ///
    /// From a given end point, like a return value, find the first node that stores to the given register.
    /// This function works by traversing the previous nodes until it finds a node that stores to the given register.
    /// This is used to correctly mark up the first store to a register that might
    /// have been incorrect.
    fn error_ranges_for_first_store(node: &Rc<CfgNode>, item: Register) -> Vec<With<Register>> {
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
            if let Some(reg) = prev.node().stores_to() {
                if reg.data == item {
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
    fn error_ranges_for_first_usage(node: &Rc<CfgNode>, item: Register) -> Vec<With<Register>> {
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
        for node in cfg {
            if let Some(register) = node.node().stores_to() {
                if register == Register::X0 && !node.node().can_skip_save_checks() {
                    errors.push(LintError::SaveToZero(register.clone()));
                }
            }
        }
    }
}

pub struct DeadValueCheck;
impl LintPass for DeadValueCheck {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
        for node in cfg {
            // check the out of the node for any uses that
            // should not be there (temporaries)
            // TODO merge with Callee saved register check
            if let Some((function, call_site)) = node.calls_to(cfg) {
                // check the expected return values of the function:

                let out = (RegSets::caller_saved() - function.returns()) & node.live_out();

                // if there is anything left, then there is an error
                // for each item, keep going to the next node until a use of
                // that item is found
                let mut ranges = Vec::new();
                for item in &out {
                    ranges.append(&mut Cfg::error_ranges_for_first_usage(&node, item));
                }
                for item in ranges {
                    errors.push(LintError::InvalidUseAfterCall(
                        item,
                        Rc::clone(&function),
                        call_site.clone(),
                    ));
                }
            }
            // Check for any assignments that don't make it
            // to the end of the node. These assignments are not
            // used.
            else if let Some(def) = node.node().stores_to() {
                if !node.live_out().contains(&def.data) && !node.node().can_skip_save_checks() {
                    errors.push(LintError::DeadAssignment(def));
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
        for node in cfg {
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
        for node in cfg {
            if node.node().is_program_entry() {
                // get registers
                let garbage = node.live_in() - RegSets::program_args();
                if !garbage.is_empty() {
                    let mut ranges = Vec::new();
                    for reg in &garbage {
                        let mut ranges_tmp = Cfg::error_ranges_for_first_usage(&node, reg);
                        ranges.append(&mut ranges_tmp);
                    }
                    for range in ranges {
                        errors.push(LintError::InvalidUseBeforeAssignment(range.clone()));
                    }
                }
            } else if let Some(func) = node.is_function_entry() {
                let args = func.arguments();
                let garbage = node.live_in() - args - RegSets::saved();
                if !garbage.is_empty() {
                    let mut ranges = Vec::new();
                    for reg in &garbage {
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

                        if let Some((reg2, off2)) = node.node().uses_memory_location() {
                            if reg2 == Register::X2 && off2.0 + off >= 0 {
                                errors.push(LintError::InvalidStackOffsetUsage(
                                    node.node().clone(),
                                    off2.0 + off,
                                ));
                            }
                        }
                    } else {
                        errors.push(LintError::InvalidStackPointer(node.node()));
                        break 'outer;
                    }
                }
            }
            if let Some((reg, _)) = node.node().uses_memory_location() {
                if reg == Register::X2 {}
            }
        }

        // PASS 2: check that
    }
}

// check if the value of a calle-saved register is read as its original value
pub struct CalleeSavedGarbageReadCheck;
impl LintPass for CalleeSavedGarbageReadCheck {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
        for node in cfg {
            for read in node.node().reads_from() {
                // if the node uses a calle saved register but not a memory access and the value going in is the original value, then we are reading a garbage value
                // DESIGN DECISION: we allow any memory accesses for calle saved registers

                if RegSets::saved().contains(&read.data)
                    && node.node().uses_memory_location().is_none()
                    && node.reg_values_in().is_original_value(read.data)
                {
                    errors.push(LintError::InvalidUseBeforeAssignment(read.clone()));
                    // then we are reading a garbage value
                }
            }
        }
    }
}

// TODO check if the stack is ever stored at 0 or what not

// Check if the values of callee-saved registers are restored to the original value at the end of the function
pub struct CalleeSavedRegisterCheck;
impl LintPass for CalleeSavedRegisterCheck {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
        for func in cfg.functions().values() {
            let exit_vals = func.exit().reg_values_in();
            for reg in &RegSets::callee_saved() {
                match exit_vals.get(&reg) {
                    Some(AvailableValue::OriginalRegisterWithScalar(reg2, offset))
                        if reg2 == &reg && offset == &0 =>
                    {
                        // GOOD!
                    }
                    _ => {
                        // TODO combine with lost register check

                        // This means that we are overwriting a callee-saved register
                        // We will traverse the function to find the first time
                        // from the return point that that register was overwritten.
                        let ranges = Cfg::error_ranges_for_first_store(&func.exit(), reg);
                        for range in ranges {
                            errors.push(LintError::OverwriteCalleeSavedRegister(range));
                        }
                    }
                }
            }
        }
    }
}

// Check if the value of a calle-saved register is ever "lost" (aka. overwritten without being restored)
// This provides a more detailed image compared to above, and could be turned into extra
// diagnostic information in the future.
pub struct LostCalleeSavedRegisterCheck;
impl LintPass for LostCalleeSavedRegisterCheck {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
        for node in cfg {
            let callee = RegSets::saved();

            // If: within a function, node stores to a saved register,
            // and the value going in was the original value
            // We intentionally do not check for callee-saved registers
            // as the value is mean to be modified
            if let Some(reg) = node.node().stores_to() {
                if callee.contains(&reg.data)
                    && node.is_part_of_some_function()
                    && node.reg_values_in().get(&reg.data)
                        == Some(&AvailableValue::OriginalRegisterWithScalar(reg.data, 0))
                {
                    // Check that the value exists somewhere in the available values
                    // like the stack.
                    // if not, then we have a problem
                    let mut found = false;

                    // check stack values:
                    let stack = node.memory_values_out();
                    for (_, val) in stack {
                        if let AvailableValue::OriginalRegisterWithScalar(reg2, offset) = val {
                            if reg2 == reg.data && offset == 0 {
                                found = true;
                                break;
                            }
                        }
                    }

                    // check register values:
                    let regs = node.reg_values_out();
                    for (_, val) in regs {
                        if let AvailableValue::OriginalRegisterWithScalar(reg2, offset) = val {
                            if reg2 == reg.data && offset == 0 {
                                found = true;
                                break;
                            }
                        }
                    }

                    if !found {
                        errors.push(LintError::LostRegisterValue(reg));
                    }
                }
            }
        }
    }
}
