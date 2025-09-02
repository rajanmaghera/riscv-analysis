use crate::analysis::{Location, Value};
use crate::cfg::Cfg;
use crate::parser::{
    HasIdentity, HasRegisterSets, InstructionProperties, RVInst, RVInstructionNode, RVRegister,
    RegisterProperties,
};
use crate::passes::{DiagnosticBuilder, DiagnosticCertainty, DiagnosticLocation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};

// TODO: FIRST THING: Really think through the function that could tell us where an error could possibly
// originate from. Start from the top, then start from the bottoms. If the value was initially
// fine, but errors out, it could only happen between a function transfer or through a control-flow
// edge. Work out this logic to help with error reporting

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InstCountStat {
    pub total_insts: usize,
    pub total_direct_jumps: usize,
    pub total_indirect_jumps: usize,
    pub total_cond_branches: usize,
    pub total_function_return_insts: usize,
    pub total_function_return_insts_verified_perfect: usize,
    pub total_function_return_insts_verified_error: usize,
    pub total_function_call_insts: usize,
    pub total_function_call_insts_direct: usize,
    pub total_function_call_insts_indirect: usize,
    pub total_function_call_insts_verified_perfect: usize,
    pub total_function_call_insts_verified_error: usize,
    pub total_function_call_insts_no_read_after_call: usize,
    pub total_functions: usize,
    pub total_function_entry_nodes: usize,
    pub total_function_entry_nodes_verified_perfect: usize,
    pub total_store_insts: usize,
    pub total_store_insts_verified_stack_frame_clobber: usize,
    pub total_store_insts_verified_perfect: usize,
    pub function_entries_with_potential_errors: BTreeSet<usize>,
    pub total_instructions_that_could_be_checked: usize,
    pub total_instructions_that_checked_were_valid: usize,
    pub total_instructions_that_checked_were_invalid: usize,
}

pub fn determine_instruction_counts(cfg: &Cfg) -> InstCountStat {
    // NOTE: VERIFIED PERFECT == VERIFIED PERFECT IF ASSUMPTIONS ARE MET
    let mut stats = InstCountStat::default();
    stats.total_functions = cfg.get_all_functions().count();
    for inst in cfg.iter_source() {
        let mut to_check = false;
        let mut to_check_and_at_least_one_error = false;
        let mut to_check_and_at_least_one_uncertain = false;
        stats.total_insts += 1;
        let func_first_line = inst
            .function()
            .as_ref()
            .unwrap()
            .entry()
            .range()
            .start()
            .one_idx_line();
        if inst.is_direct_uncond_jump() {
            stats.total_direct_jumps += 1;
        }
        if inst.is_indirect_uncond_jump() {
            stats.total_indirect_jumps += 1;

            if !inst.is_return() && !inst.calls_to().is_some() {
                // print any non-return indirect jumps to manually observe
                eprintln!(
                    "non-ret indirect jump at line {} for func beginning on line {}",
                    inst.range().start().one_idx_line(),
                    func_first_line
                )
            }
        }
        if inst.is_cond_branch() {
            stats.total_cond_branches += 1;
        }
        if inst.is_return() {
            to_check = true;
            stats.total_function_return_insts += 1;
            // At the function return site, all callee saved registers and the return register
            // must be returned to their initial values. If every single one is safe in all
            // cases, then it is "verified_perfect". If at least one is for sure not restored
            // to its own value, then it is "verified_error". Otherwise, if it is unknown that
            // the register is not restored, then it is neither.
            let registers_to_check = RVRegister::callee_saved_set();
            let exit_vals = inst.real_val_in();
            let mut maybe_error = false;
            let mut really_error = false;
            for reg in &registers_to_check {
                match exit_vals.get(&Location::Register(reg)) {
                    Value::Initial(r) if !reg.is_stack_pointer() && r == reg => {}
                    Value::InitialStackPointer(x) if reg.is_stack_pointer() && x == 0 => {}
                    Value::UnknownConst | Value::Unknown => {
                        maybe_error = true;
                    }
                    _ => {
                        really_error = true;
                    }
                }
            }
            if really_error {
                to_check_and_at_least_one_error = true;
                stats.total_function_return_insts_verified_error += 1;
            } else if !maybe_error {
                stats.total_function_return_insts_verified_perfect += 1;
            } else {
                to_check_and_at_least_one_uncertain = true;
                stats
                    .function_entries_with_potential_errors
                    .insert(func_first_line);
                eprintln!(
                    "unknown FRI values on line {} for func beginning on line {}",
                    inst.range().start().one_idx_line(),
                    func_first_line
                );
            }
        }
        if inst.calls_to().is_some() {
            to_check = true;
            stats.total_function_call_insts += 1;
            // At the function call site, the stack pointer must be a negative
            // offset of 4 of the initial value.
            if inst.is_indirect_uncond_jump() {
                stats.total_function_call_insts_indirect += 1;
            } else if inst.is_direct_uncond_jump() {
                stats.total_function_call_insts_direct += 1;
            }
            let mut really_error = false;
            let mut maybe_error = false;

            match inst
                .real_val_in()
                .get(&Location::Register(RVRegister::stack_pointer()))
            {
                Value::InitialStackPointer(x) if x % 4 == 0 && x <= 0 => {}
                Value::Unknown => {
                    maybe_error = true;
                }
                _ => {
                    really_error = true;
                }
            }

            if really_error {
                to_check_and_at_least_one_error = true;
                stats.total_function_call_insts_verified_error += 1;
            } else if !maybe_error {
                stats.total_function_call_insts_verified_perfect += 1;
            } else {
                to_check_and_at_least_one_uncertain = true;
                stats
                    .function_entries_with_potential_errors
                    .insert(func_first_line);
                eprintln!(
                    "unknown FCI values on line {} for func beginning on line {}",
                    inst.range().start().one_idx_line(),
                    func_first_line
                );
            }

            // At a function call site, no (non-return) registers should be live
            // out, otherwise we are reading a clobbered register

            let mut really_error = false;
            for reg in (RVRegister::caller_saved_set() - RVRegister::return_set()).iter() {
                if inst.live_out().contains(&reg) {
                    really_error = true;
                }
            }
            if !really_error {
                stats.total_function_call_insts_no_read_after_call += 1;
            } else {
                to_check_and_at_least_one_error = true;
                stats
                    .function_entries_with_potential_errors
                    .insert(func_first_line);
                eprintln!(
                    "read after call (at FCI) on line {} for func beginning on line {}",
                    inst.range().start().one_idx_line(),
                    func_first_line
                );
            }
        }
        if inst.is_function_entry() {
            to_check = true;
            // no (non-arg) caller-saved register should be live at the entry point
            stats.total_function_entry_nodes += 1;
            let mut really_error = false;
            for reg in (RVRegister::caller_saved_set() - RVRegister::argument_set()).iter() {
                if inst.live_in().contains(&reg) {
                    really_error = true;
                }
            }
            if !really_error {
                stats.total_function_entry_nodes_verified_perfect += 1;
            } else {
                to_check_and_at_least_one_error = true;
                stats
                    .function_entries_with_potential_errors
                    .insert(func_first_line);
                eprintln!(
                    "read before write (at FEnN) on line {} for func beginning on line {}",
                    inst.range().start().one_idx_line(),
                    func_first_line
                );
            }
        }
        if let RVInstructionNode::Store(s) = inst.node() {
            to_check = true;
            // If this instruction stores to an address, check that
            // the address is not on the caller (of inst's function)'s stack frame
            stats.total_store_insts += 1;
            let addr_reg = *s.rs1.get();
            let val = inst.real_val_in().get(&Location::Register(addr_reg));
            match val {
                Value::Unknown => {
                    // It may be clobbering the values
                    to_check_and_at_least_one_uncertain = true;
                    stats
                        .function_entries_with_potential_errors
                        .insert(func_first_line);
                    eprintln!(
                        "potential stack clobber on line {} for func beginning on line {}",
                        inst.range().start().one_idx_line(),
                        func_first_line
                    );
                }
                Value::InitialStackPointer(x) => {
                    if let Some(imm) = s.imm.value() {
                        let real_offset = x + imm;
                        if real_offset > -4 {
                            // This absolutely clobbers the wrong stack
                            to_check_and_at_least_one_error = true;
                            stats.total_store_insts_verified_stack_frame_clobber += 1;
                        } else {
                            stats.total_store_insts_verified_perfect += 1;
                        }
                    } else {
                        to_check_and_at_least_one_uncertain = true;
                        stats
                            .function_entries_with_potential_errors
                            .insert(func_first_line);
                        eprintln!(
                            "potential stack clobber on line {} for func beginning on line {}",
                            inst.range().start().one_idx_line(),
                            func_first_line
                        );
                    }
                }
                Value::UnknownConst | Value::Initial(_) | Value::Undefined | Value::Const(_) => {
                    // It will not touch the stack
                    stats.total_store_insts_verified_perfect += 1;
                }
            }
        }

        if to_check {
            stats.total_instructions_that_could_be_checked += 1;
            if to_check_and_at_least_one_error {
                stats.total_instructions_that_checked_were_invalid += 1;
            } else if !to_check_and_at_least_one_uncertain {
                stats.total_instructions_that_checked_were_valid += 1;
            }
        }
    }

    stats
}
