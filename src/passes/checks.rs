use crate::cfg::{AvailableRegisterValues, RegSets};
use crate::parser::inst::BasicType;
use crate::parser::register::Register;
use crate::parser::token::{LineDisplay, Range};
use crate::{
    cfg::{AnnotatedCFG, AvailableValue},
    parser::ast::ASTNode,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

use super::*;

// If we need to add an error to a register at its first use/store, we need to
// know their ranges. This function will take a register and return the ranges
// that need to be annotated. If it cannot find any, then it will return the original
// node's range.
impl AnnotatedCFG {
    fn error_ranges_for_first_usage(&self, node: &Rc<ASTNode>, item: Register) -> Vec<Range> {
        let mut queue = VecDeque::new();
        let mut ranges = Vec::new();
        // push the next nodes onto the queue
        for next in self.next_ast_map.get(node).unwrap() {
            queue.push_back(next.clone());
        }

        // keep track of visited nodes
        let mut visited = HashSet::new();
        visited.insert(node.clone());

        // visit each node in the queue
        // if the error is found, add error
        // if not, add the next nodes to the queue
        while let Some(next) = queue.pop_front() {
            if visited.contains(&next) {
                continue;
            }
            visited.insert(next.clone());
            if next.gen().contains(&item) {
                // find the use
                let regs = next.gen();
                let mut it = None;
                for reg in regs {
                    if reg == item {
                        it = Some(reg);
                        break;
                    }
                }
                if let Some(_reg) = it {
                    // TODO fix range of token register
                    ranges.push(next.get_range().clone());
                    break;
                }
                break;
            }
            for next_next in self.next_ast_map.get(&next).unwrap() {
                queue.push_back(next_next.clone());
            }
        }
        ranges
    }
}

// Checks are passes that occur after the CFG is built. As much data as possible is collected
// during the CFG build. Then, the data is applied via a check.

pub struct SaveToZeroCheck;
impl Pass for SaveToZeroCheck {
    fn run(&self, cfg: &AnnotatedCFG) -> Result<(), PassErrors> {
        let mut errors = Vec::new();
        for block in cfg.blocks.clone() {
            for node in block.0.clone() {
                if let Some(register) = (*node).stores_to() {
                    if register == Register::X0 && !node.is_return() {
                        errors.push(PassError::SaveToZero(register.get_range()));
                    }
                }
            }
        }

        if errors.len() > 0 {
            Err(PassErrors { errors })
        } else {
            Ok(())
        }
    }
}

pub struct DeadValueCheck;
impl Pass for DeadValueCheck {
    fn run(&self, cfg: &AnnotatedCFG) -> Result<(), PassErrors> {
        let live_data = &cfg.liveness;
        let mut errors = Vec::new();
        let mut i: usize = 0;
        // recalc mapping of nodes to idx
        let mut node_idx = HashMap::new();
        for block in cfg.blocks.clone() {
            for node in block.0.clone() {
                node_idx.insert(node, i);
                i += 1;
            }
        }
        let mut i = 0;
        for block in cfg.blocks.clone() {
            for node in block.0.clone() {
                // check for any assignments that don't make it
                // to the end of the node
                for def in node.kill() {
                    if !live_data.live_out.get(i).unwrap().contains(&def) {
                        // TODO dead assignment register
                        // TODO darken variable in LSP
                        errors.push(PassError::DeadAssignment(node.get_store_range().clone()));
                    }
                }

                // check the out of the node for any uses that
                // should not be there (temporaries)
                // TODO merge with Callee saved register check
                if let Some(name) = node.calls_func_to() {
                    // check the expected return values of the function:
                    let returns = cfg.function_rets(name.data.0.as_str()).unwrap();
                    let out: HashSet<Register> = RegSets::caller_saved()
                        .difference(&returns)
                        .cloned()
                        .collect();
                    let out = out
                        .intersection(&live_data.live_out.get(i).unwrap())
                        .cloned();

                    // if there is anything left, then there is an error
                    // for each item, keep going to the next node until a use of
                    // that item is found
                    let mut ranges = Vec::new();
                    for item in out {
                        ranges.append(&mut cfg.error_ranges_for_first_usage(&node, item));
                    }
                    for item in ranges {
                        errors.push(PassError::InvalidUseAfterCall(item, name.clone()))
                    }
                }
                i += 1;
            }
        }
        if errors.len() > 0 {
            Err(PassErrors { errors })
        } else {
            Ok(())
        }
    }
}

// Check if you can enter a function through the first line of code
// Check if you can enter a function through a jump (a previous exists)
// Check if any code has no previous (except for the first line of code)
pub struct ControlFlowCheck;
impl Pass for ControlFlowCheck {
    fn run(&self, cfg: &AnnotatedCFG) -> Result<(), PassErrors> {
        let mut errors = Vec::new();
        let mut bigidx = 0;
        for block in cfg.blocks.clone() {
            for node in block.0.clone() {
                match &(*node) {
                    ASTNode::FuncEntry(x) => {
                        if bigidx == 0 || cfg.prev_ast_map.get(&node).unwrap().len() > 0 {
                            errors.push(PassError::ImproperFuncEntry(
                                x.name.get_range().clone(),
                                x.name.clone(),
                            ));
                        }
                    }
                    _ => {
                        if bigidx != 0 && cfg.prev_ast_map.get(&node).unwrap().len() == 0 {
                            errors.push(PassError::UnreachableCode(node.get_range().clone()));
                        }
                    }
                }
                bigidx += 1;
            }
        }

        if errors.len() > 0 {
            Err(PassErrors { errors })
        } else {
            Ok(())
        }
    }
}

// Check if every ecall has a known call number
// TODO Check if there are any instructions after an ecall to terminate the program
pub struct EcallCheck;
impl Pass for EcallCheck {
    fn run(&self, cfg: &AnnotatedCFG) -> Result<(), PassErrors> {
        let mut errors = Vec::new();
        let mut bigidx = 0;
        for block in cfg.blocks.clone() {
            for node in block.0.clone() {
                match &(*node) {
                    ASTNode::Basic(x) => {
                        if x.inst.data == BasicType::Ecall {
                            if cfg
                                .available
                                .avail_in
                                .get(bigidx)
                                .unwrap()
                                .get(&Register::X17)
                                .is_none()
                            {
                                errors.push(PassError::UnknownEcall(x.inst.get_range().clone()));
                            }
                        }
                    }
                    _ => {}
                }
                bigidx += 1;
            }
        }

        if errors.len() > 0 {
            Err(PassErrors { errors })
        } else {
            Ok(())
        }
    }
}

// TODO deprecate
// Check if there are any in values to the start of functions that are not args or saved registers
// Check if there are any in values at the start of a program
pub struct GarbageInputValueCheck;
impl Pass for GarbageInputValueCheck {
    fn run(&self, cfg: &AnnotatedCFG) -> Result<(), PassErrors> {
        let mut errors = Vec::new();
        let mut bigidx = 0;
        for block in cfg.blocks.clone() {
            for node in block.0.clone() {
                if bigidx == 0 {
                    let mut garbage = cfg
                        .liveness
                        .live_in
                        .get(bigidx)
                        .unwrap()
                        .clone()
                        .into_iter()
                        .collect::<Vec<_>>();
                    garbage.retain(|x| !RegSets::saved().contains(&x));
                    if garbage.len() > 0 {
                        let mut ranges = Vec::new();
                        for reg in garbage {
                            let mut ranges_tmp = cfg.error_ranges_for_first_usage(&node, reg);
                            ranges.append(&mut ranges_tmp)
                        }
                        for range in ranges {
                            errors.push(PassError::InvalidUseBeforeAssignment(range.clone()));
                        }
                    }
                } else {
                    match &(*node) {
                        ASTNode::FuncEntry(x) => {
                            let args = cfg.function_args(x.name.data.0.as_str()).unwrap();
                            let mut garbage = cfg
                                .liveness
                                .live_in
                                .get(bigidx)
                                .unwrap()
                                .clone()
                                .into_iter()
                                .collect::<Vec<_>>();
                            garbage.retain(|x| !args.contains(&x));
                            garbage.retain(|x| !RegSets::saved().contains(&x));
                            if garbage.len() > 0 {
                                let mut ranges = Vec::new();
                                for reg in garbage {
                                    let mut ranges_tmp =
                                        cfg.error_ranges_for_first_usage(&node, reg);
                                    ranges.append(&mut ranges_tmp)
                                }
                                for range in ranges {
                                    errors
                                        .push(PassError::InvalidUseBeforeAssignment(range.clone()));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                bigidx += 1;
            }
        }

        if errors.len() > 0 {
            Err(PassErrors { errors })
        } else {
            Ok(())
        }
    }
}

// Check that we know the stack position at every point in the program (aka. within scopes)
pub struct StackCheckPass;
impl Pass for StackCheckPass {
    fn run(&self, cfg: &AnnotatedCFG) -> Result<(), PassErrors> {
        let mut errors = Vec::new();

        // PASS 1
        // check that we know the stack position at every point in the program
        // check that the stack is never in an invalid position
        let mut bigidx = 0;
        for block in cfg.blocks.clone() {
            for node in block.0.clone() {
                let values = cfg.available.avail_out.get(bigidx).unwrap();
                match values.get(&Register::X2) {
                    None => {
                        errors.push(PassError::UnknownStack(node.get_range().clone()));
                        return Err(PassErrors { errors });
                    }
                    Some(x) => match x {
                        AvailableValue::OrigScalarOffset(reg, off) => {
                            if reg != &Register::X2 {
                                errors
                                    .push(PassError::InvalidStackPointer(node.get_range().clone()));
                                return Err(PassErrors { errors });
                            }
                            if off > &0 {
                                errors.push(PassError::InvalidStackPosition(
                                    node.get_range().clone(),
                                    off.clone(),
                                ));
                                return Err(PassErrors { errors });
                            }
                        }
                        _ => {
                            errors.push(PassError::InvalidStackPointer(node.get_range().clone()));
                            return Err(PassErrors { errors });
                        }
                    },
                }
                bigidx += 1;
            }
        }

        // PASS 2: check that

        if errors.len() > 0 {
            Err(PassErrors { errors })
        } else {
            Ok(())
        }
    }
}

// check if the value of a calle-saved register is read as its original value
pub struct CalleeSavedGarbageReadCheck;
impl Pass for CalleeSavedGarbageReadCheck {
    fn run(&self, cfg: &AnnotatedCFG) -> Result<(), PassErrors> {
        let mut errors = Vec::new();
        for (i, node) in cfg.nodes.clone().into_iter().enumerate() {
            for read in node.reads_from() {
                // if the node uses a calle saved register but not a memory access and the value going in is the original value, then we are reading a garbage value
                // DESIGN DECISION: we allow any memory accesses for calle saved registers

                if RegSets::saved().contains(&read.data)
                    && (!node.is_memory_access())
                    && cfg
                        .available
                        .avail_in
                        .get(i)
                        .unwrap()
                        .is_original_value(&read.data)
                {
                    errors.push(PassError::InvalidUseBeforeAssignment(
                        read.get_range().clone(),
                    ));
                    // then we are reading a garbage value
                }
            }
        }
        if errors.len() > 0 {
            Err(PassErrors { errors })
        } else {
            Ok(())
        }
    }
}

pub struct CalleeSavedRegisterCheck;
impl Pass for CalleeSavedRegisterCheck {
    fn run(&self, cfg: &AnnotatedCFG) -> Result<(), PassErrors> {
        let mut errors = Vec::new();

        // for all functions
        for (_func_name, func_ret) in &cfg.label_return_map {
            // TODO scan function to find all "first" definitions of function,
            // then mark those up

            let func_ret = func_ret.iter().next().unwrap();
            // check if the original values for all calle saved are available at the end
            let val = cfg
                .available
                .avail_in
                .get(cfg.nodes.iter().position(|x| x == func_ret).unwrap())
                .unwrap();
            use Register::*;
            for reg in vec![
                X1, X2, X8, X9, X18, X19, X20, X21, X22, X23, X24, X25, X26, X27,
            ] {
                match val.get(&reg) {
                    None => {
                        errors.push(PassError::OverwriteCalleeSavedRegister(
                            func_ret.get_range().clone(),
                            reg,
                        ));
                    }
                    Some(val) => match val {
                        AvailableValue::OrigScalarOffset(reg2, offset) => {
                            if reg2 != &reg || offset != &0 {
                                errors.push(PassError::OverwriteCalleeSavedRegister(
                                    func_ret.get_range().clone(),
                                    reg,
                                ));
                            }
                        }
                        _ => {
                            errors.push(PassError::OverwriteCalleeSavedRegister(
                                func_ret.get_range().clone(),
                                reg,
                            ));
                        }
                    },
                }
            }
        }
        if errors.len() > 0 {
            Err(PassErrors { errors })
        } else {
            Ok(())
        }
    }
}
