// AVAILABLE VALUE ANALYSIS
// ========================

use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::cfg::AvailableValueMap;
use crate::parser::{
    CSRImm, HasRegisterSets, InstructionProperties, LabelString, LabelStringToken,
    RegisterProperties,
};
use crate::parser::{ParserNode, Register};
use crate::passes::{CfgError, GenerationPass};

use super::memory_location::MemoryLocation;
use super::{HasGenKillInfo, HasGenValueInfo};
#[derive(Clone, Debug, PartialEq, Eq, Hash)]

/// A value that is available at some point in the program.
///
/// This is used to determine which values at certain locations (registers, memory)
/// can be used or guaranteed. These are assigned as the value in a `HashMap`.
///
/// The `Original` variants are used to determine whether a value is the same as
/// the value at the beginning of the function or graph. This is used to determine
/// whether a value is the same as the value at the beginning of the function or
/// graph. This is mostly used for stack pointer manipulation.
#[derive(Deserialize, Serialize)]
pub enum AvailableValue {
    /// A known constant value.
    #[serde(rename = "c")]
    Constant(i32),
    /// The address of some memory location.
    ///
    /// This is used when loading the address from a label. For example, using
    /// the `la` instruction to load the address of a label into a register.
    #[serde(rename = "a")]
    Address(LabelStringToken),
    /// The value of a memory location at some offset.
    ///
    /// This is a copy of the actual bit of memory that lives at plus some offset.
    /// Note that this offset is not a scalar offset, but an offset to the memory
    /// address. Think of it as the offset in the `lw` instruction. For example,
    /// `lw x10, offset(label)` would be represented as `Memory(label, offset)`.
    #[serde(rename = "m")]
    Memory(LabelString, i32),
    /// The value of a register plus some scalar offset.
    ///
    /// This is used when we know the value of a register plus some scalar offset.
    /// For example, if we know that `x10` is `x2 + 4`, then we would represent
    /// this as `ValueWithScalar(x2, 4)`. Ideally, we should be eliminating these
    /// values as much as possible.
    ///
    /// Scalar offsets consist of add and subtract operations with known constants.
    /// This is mostly used for stack pointer manipulation.
    #[serde(rename = "rs")]
    RegisterWithScalar(Register, i32),
    /// The value of a register at the beginning of the function plus some scalar offset.
    ///
    /// This is used when we know the value of a register at the beginning of the
    /// function plus some scalar offset. The register must be proved to be its
    /// original value before this can be used.
    ///
    /// Scalar offsets consist of add and subtract operations with known constants.
    /// This is mostly used for stack pointer manipulation.
    #[serde(rename = "ors")]
    OriginalRegisterWithScalar(Register, i32),
    #[serde(rename = "mr")]
    MemoryAtRegister(Register, i32), // Actual bit of memory + offset (ex. lw ___), where we do not know the label
    #[serde(rename = "omr")]
    MemoryAtOriginalRegister(Register, i32), // Actual bit of memory + offset (ex. lw ___), where we are sure it is the same as the original
    /// The value inside of a CSR register.
    #[serde(rename = "c")]
    ValueInCsr(CSRImm),
    /// Value at memory location of value in CSR register.
    #[serde(rename = "mc")]
    MemoryAtCsr(CSRImm, i32),
}

/// Performs the available value analysis on the graph.
///
/// This function contains the logic for determining which values are available
/// at any given point in the program.
///
/// As part of the subset of RISC-V programs that we allow, stack pointer
/// manipulation, calle-saved register stores/restores, and ecall arguments
/// must be unconditionally determined. We allow a "somewhat" arbitrary
/// amount of operations to be performed on known constants for our purposes.
///
/// With the available value anaylsis, there are a few caveats. While we can
/// expand this to an arbitrary number of operations, we currently only support
/// a few operations that are used to enfore the rules we care about. This
/// is what you should know about the current implementation:
/// - Any number of known math operations on known constants are allowed and
///   can be represented as a known constant.
/// - Values from arbitrary memory locations are not known. They are represented
///   in the enum, but we cannot perform many operations on them. This is
///   expected to change in an upcoming version.
/// - Any type of `AvailableValue` can be stored and restored from the stack.
/// - The stack pointer can only be manipulated by a known constant. This is
///   not ideal, but it is a limitation of the current implementation. For our
///   purposes, it's good enough.
/// - There is no propogation of values across functions. Each function is
///   analyzed independently.
///
pub struct AvailableValuePass;
impl GenerationPass for AvailableValuePass {
    fn run(cfg: &mut crate::cfg::Cfg) -> Result<(), Box<CfgError>> {
        let mut changed = true;

        // Because of this type of algorithm, there might be a back branch,
        // like a loop, that has not been visited before the first in[n] is
        // calculated. To fix this, we keep track of what nodes have been
        // visited and only factor those into each calculation at a given point.
        // We still ensure that, by the end, all nodes have been visited and
        // the values with the correct previous nodes are calculated.
        #[allow(clippy::mutable_key_type)]
        let mut visited = HashSet::new();
        while changed {
            changed = false;
            for node in cfg.iter() {
                // in[n] = AND out[p] for all p in prev[n]
                let in_reg_n = node
                    .prevs()
                    .clone()
                    .into_iter()
                    .filter(|x| visited.contains(x))
                    .map(|x| x.reg_values_out())
                    .reduce(|mut acc, x| {
                        acc &= &x;
                        acc
                    })
                    .unwrap_or_default();
                changed |= node.set_reg_values_in(in_reg_n);

                // in_memory[n] = AND out_memory[p] for all p in prev[n]
                let in_memory_n = node
                    .prevs()
                    .clone()
                    .into_iter()
                    .filter(|x| visited.contains(x))
                    .map(|x| x.memory_values_out())
                    .reduce(|mut acc, x| {
                        acc &= &x;
                        acc
                    })
                    .unwrap_or_default();
                changed |= node.set_memory_values_in(in_memory_n);

                // out[n] = gen[n] U (in[n] - kill[n]) U (callee_saved if n is entry)
                let mut out_reg_n = node.reg_values_in();
                out_reg_n -= node.kill_reg().iter();
                if node.calls_to().is_some() {
                    out_reg_n -= Register::return_addr_set().iter();
                }
                if let Some((reg, reg_value)) = node.gen_reg_value() {
                    out_reg_n.insert(reg, reg_value);
                }
                if node.is_handler_function_entry() {
                    out_reg_n.extend(Register::all_writable_set().into_available_values());
                }
                if node.is_function_entry() {
                    out_reg_n.extend(Register::callee_saved_set().into_available_values());
                }
                if node.is_program_entry() {
                    out_reg_n.extend(Register::sp_ra_set().into_available_values());
                }

                // out_memory[n] = (gen_memory[n] if we know the location of the stack pointer) U in_memory[n]
                // (There is no kill_stacks[n])
                let mut out_memory_n = if node.is_any_entry() {
                    AvailableValueMap::new()
                } else {
                    let mut map = node.memory_values_in();
                    if let Some((MemoryLocation::StackOffset(offset), value)) =
                        node.gen_memory_value()
                    {
                        if let Some(curr_stack) = node.reg_values_in().stack_offset() {
                            map.insert(MemoryLocation::StackOffset(curr_stack + offset), value);
                        }
                    } else {
                        if let Some((memory, value)) = node.gen_memory_value() {
                            map.insert(memory, value);
                        }
                    }
                    map
                };

                // AVAILABLE VALUE/STACK ESTIMATION
                // ================================
                // We use a series of rules to determine new available values
                // that change our outs.

                rule_expand_address_for_load(&node.node(), &mut out_reg_n, &node.reg_values_in());
                rule_value_from_stack(&node.node(), &mut out_reg_n, &node.memory_values_in());
                rule_pull_value_from_csr_memory(
                    &node.node(),
                    &mut out_reg_n,
                    &node.memory_values_out(),
                );
                rule_zero_to_const(
                    &mut out_reg_n,
                    &node.reg_values_in(),
                    &mut out_memory_n,
                    &node.memory_values_in(),
                );
                rule_perform_math_ops(&node.node(), &mut out_reg_n, &node.reg_values_in());
                rule_push_value_to_csr_memory(&node.node(), &mut out_memory_n, &out_reg_n);
                rule_known_values_to_stack(&mut out_memory_n, &node.reg_values_in());
                // TODO stack reset?

                // If either of the outs changed, replace the old outs with the new outs
                // and mark that we changed something.
                changed |= node.set_reg_values_out(out_reg_n);
                changed |= node.set_memory_values_out(out_memory_n);

                // Add node to visited
                visited.insert(Rc::clone(&node));
            }
        }
        Ok(())
    }
}

/// Rule that converts all zero registers to constant zeros.
///
/// If a register that an available value reads from is the zero register, then
/// replace it with a constant zero. This is because constants are easier
/// to deal with than registers and the analysis has no idea how to deal
/// with the zero register.
fn rule_zero_to_const(
    available_out: &mut AvailableValueMap<Register>,
    available_in: &AvailableValueMap<Register>,
    memory_out: &mut AvailableValueMap<MemoryLocation>,
    memory_in: &AvailableValueMap<MemoryLocation>,
) {
    for val in available_in {
        match val.1 {
            AvailableValue::OriginalRegisterWithScalar(r, i)
            | AvailableValue::RegisterWithScalar(r, i) => {
                if r.is_const_zero() {
                    available_out.insert(*val.0, AvailableValue::Constant(*i));
                }
            }
            _ => {}
        }
    }
    for val in memory_in {
        match val.1 {
            AvailableValue::OriginalRegisterWithScalar(r, i)
            | AvailableValue::RegisterWithScalar(r, i) => {
                if r.is_const_zero() {
                    memory_out.insert(val.0.clone(), AvailableValue::Constant(*i));
                }
            }
            _ => {}
        }
    }
}

/// Rule that uses known addresses for load instructions to expand their represenation.
///
/// If a load instruction is found and the register where the address is contains
/// a reference to a register value or memory address, then replace the loaded value
/// with a reference to the specific memory location.
fn rule_expand_address_for_load(
    node: &ParserNode,
    available_out: &mut AvailableValueMap<Register>,
    available_in: &AvailableValueMap<Register>,
) {
    if let Some(store_reg) = node.writes_to() {
        if let ParserNode::Load(load) = node {
            if let Some(AvailableValue::OriginalRegisterWithScalar(reg, off)) =
                available_in.get(&load.rs1.get())
            {
                available_out.insert(
                    *store_reg.get(),
                    AvailableValue::MemoryAtOriginalRegister(*reg, *off + load.imm.get().0),
                );
            } else if let Some(AvailableValue::Address(label)) = available_in.get(&load.rs1.get()) {
                available_out.insert(
                    *store_reg.get(),
                    AvailableValue::Memory(label.get().clone(), load.imm.get().0),
                );
            }
        }
    }
}

/// Rule that performs math operations on register values.
///
/// If a register is stored to and we can determine the new value based on the
/// values before and known math operations, store the new value in the register.
fn rule_perform_math_ops(
    node: &ParserNode,
    available_out: &mut AvailableValueMap<Register>,
    available_in: &AvailableValueMap<Register>,
) {
    if let Some(reg) = node.writes_to() {
        let lhs = match node {
            ParserNode::Arith(expr) => available_in.get(&expr.rs1.get()).cloned(),
            ParserNode::IArith(expr) => available_in.get(&expr.rs1.get()).cloned(),
            _ => None,
        };

        let rhs = match node {
            ParserNode::Arith(expr) => available_in.get(&expr.rs2.get()).cloned(),
            ParserNode::IArith(expr) => Some(AvailableValue::Constant(expr.imm.get().0)),
            _ => None,
        };

        let result = match (lhs, rhs) {
            (Some(AvailableValue::Constant(x)), Some(AvailableValue::Constant(y))) => node
                .inst()
                .math_op()
                .map(|op| op.operate(x, y))
                .map(AvailableValue::Constant),
            (
                Some(AvailableValue::OriginalRegisterWithScalar(new_reg, x)),
                Some(AvailableValue::Constant(y)),
            )
            | (
                Some(AvailableValue::Constant(x)),
                Some(AvailableValue::OriginalRegisterWithScalar(new_reg, y)),
            ) => node
                .inst()
                .scalar_op()
                .map(|op| op.operate(x, y))
                .map(|z| AvailableValue::OriginalRegisterWithScalar(new_reg, z)),
            (_, _) => None,
        };
        if let Some(val) = result {
            available_out.insert(*reg.get(), val);
        }
    }
}

/// Rule that restores guaranteed register values from the stack.
///
/// If a register is stored to from a memory location that is the stack, and
/// the stack contains a value at the offset, then store the value from the
/// stack into the register.
fn rule_value_from_stack(
    node: &impl InstructionProperties,
    available_out: &mut AvailableValueMap<Register>,
    memory_in: &AvailableValueMap<MemoryLocation>,
) {
    if let Some(reg) = node.writes_to() {
        if let Some(AvailableValue::ValueInCsr(csr)) = available_out.get(&reg.get()) {
            if let Some(csr_value) = memory_in.get(&MemoryLocation::CsrRegister(*csr)) {
                available_out.insert(*reg.get(), csr_value.clone());
            }
        }

        if let Some(AvailableValue::MemoryAtOriginalRegister(psp, off)) =
            available_out.get(&reg.get())
        {
            if psp.is_stack_pointer() {
                if let Some(stack_val) = memory_in.get(&MemoryLocation::StackOffset(*off)) {
                    available_out.insert(*reg.get(), stack_val.clone());
                }
            }
        }
    }
}

/// Rule that pushes guaranteed known values to the stack.
///
/// If a value on the stack is a reference to some register value (A),
/// but that register value is either a constant or the guaranteed register
/// value at the entry of the function (B), then replace A with B.
fn rule_known_values_to_stack(
    memory_out: &mut AvailableValueMap<MemoryLocation>,
    available_in: &AvailableValueMap<Register>,
) {
    for (pos, val) in memory_out.clone() {
        if let AvailableValue::RegisterWithScalar(reg, off) = val {
            if let Some(item) = available_in.get(&reg) {
                match item {
                    AvailableValue::Constant(x) => {
                        memory_out.insert(pos, AvailableValue::Constant(*x + off));
                    }
                    AvailableValue::OriginalRegisterWithScalar(reg2, off3) => {
                        memory_out.insert(
                            pos,
                            AvailableValue::OriginalRegisterWithScalar(*reg2, *off3 + off),
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}

fn rule_push_value_to_csr_memory(
    node: &impl InstructionProperties,
    memory_out: &mut AvailableValueMap<MemoryLocation>,
    available_in: &AvailableValueMap<Register>,
) {
    // If the node writes to memory
    if let Some((source, (reg, off))) = node.stores_to_memory() {
        // If the register contains a csr value
        if let Some(AvailableValue::ValueInCsr(csr)) = available_in.get(&reg) {
            // Push the value to the memory
            memory_out.insert(
                MemoryLocation::CsrRegisterValueOffset(*csr, off.0),
                AvailableValue::RegisterWithScalar(source, 0),
            );
        }
    }
}

fn rule_pull_value_from_csr_memory(
    node: &impl InstructionProperties,
    available_out: &mut AvailableValueMap<Register>,
    memory_out: &AvailableValueMap<MemoryLocation>,
) {
    // If the node reads from memory
    if let Some(((reg, off), dest)) = node.reads_from_memory() {
        // If the source address is a csr memory location
        if let Some(AvailableValue::ValueInCsr(csr)) = available_out.get(&reg) {
            // If the memory at csr contains a value
            if let Some(value) =
                memory_out.get(&MemoryLocation::CsrRegisterValueOffset(*csr, off.0))
            {
                // Pull the value from the memory
                available_out.insert(dest, value.clone());
            }
        }
    }
}

// TODO: generic function that converts available value to memory location and vice versa
