// AVAILABLE VALUE ANALYSIS
// ========================

use std::collections::HashMap;
use std::hash::Hash;

use crate::parser::{LabelString, RegSets};
use crate::parser::{ParserNode, Register};
use crate::passes::{CFGError, GenerationPass};

use super::{CustomDifference, CustomIntersection, CustomInto, CustomUnion, CustomUnionFilterMap};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]

/// A value that is available at some point in the program.
///
/// This is used to determine which values at certain locations (registers, memory)
/// can be used or guaranteed. These are assigned as the value in a `HashMap.
///
/// The `Original` variants are used to determine whether a value is the same as
/// the value at the beginning of the function or graph. This is used to determine
/// whether a value is the same as the value at the beginning of the function or
/// graph. This is mostly used for stack pointer manipulation.
pub enum AvailableValue {
    /// A known constant value.
    Constant(i32),
    /// The address of some memory location.
    ///
    /// This is used when loading the address from a label. For example, using
    /// the `la` instruction to load the address of a label into a register.
    Address(LabelString),
    /// The value of a memory location at some offset.
    ///
    /// This is a copy of the actual bit of memory that lives at plus some offset.
    /// Note that this offset is not a scalar offset, but an offset to the memory
    /// address. Think of it as the offset in the `lw` instruction. For example,
    /// `lw x10, offset(label)` would be represented as `Memory(label, offset)`.
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
    RegisterWithScalar(Register, i32),
    /// The value of a register at the beginning of the function plus some scalar offset.
    ///
    /// This is used when we know the value of a register at the beginning of the
    /// function plus some scalar offset. The register must be proved to be its
    /// original value before this can be used.
    ///
    /// Scalar offsets consist of add and subtract operations with known constants.
    /// This is mostly used for stack pointer manipulation.
    OriginalRegisterWithScalar(Register, i32),
    MemoryAtRegister(Register, i32), // Actual bit of memory + offset (ex. lw ___), where we do not know the label
    MemoryAtOriginalRegister(Register, i32), // Actual bit of memory + offset (ex. lw ___), where we are sure it is the same as the original
}

pub trait AvailableRegisterValues {
    fn is_original_value(&self, reg: Register) -> bool;
}

impl AvailableRegisterValues for HashMap<Register, AvailableValue> {
    fn is_original_value(&self, reg: Register) -> bool {
        self.get(&reg).map_or(false, |x| match x {
            AvailableValue::OriginalRegisterWithScalar(reg2, offset) => {
                &reg == reg2 && offset == &0
            }
            _ => false,
        })
    }
}

trait AvailableStackHelpers {
    /// Returns the offset of the stack pointer if it is known.
    ///
    /// This is used to determine the offset of the stack pointer in relation
    /// to the value it was at the beginning of the function or graph.
    fn stack_offset(&self) -> Option<i32>;
}

impl AvailableStackHelpers for HashMap<Register, AvailableValue> {
    fn stack_offset(&self) -> Option<i32> {
        if let Some(AvailableValue::OriginalRegisterWithScalar(reg, off)) = self.get(&Register::X2)
        {
            if reg == &Register::X2 {
                return Some(*off);
            }
        }
        None
    }
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
    fn run(cfg: &mut crate::cfg::BaseCFG) -> Result<(), CFGError> {
        let mut changed = true;
        while changed {
            changed = false;
            for node in cfg.into_iter().rev() {
                // in[n] = AND out[p] for all p in prev[n]
                let in_reg_n = node
                    .prevs()
                    .clone()
                    .into_iter()
                    .map(|x| x.reg_values_in())
                    .reduce(|acc, x| x.intersection(&acc))
                    .unwrap_or_default();
                node.set_reg_values_in(in_reg_n);

                // in_stacks[n] = AND out_stacks[p] for all p in prev[n]
                let in_stack_n = node
                    .prevs()
                    .clone()
                    .into_iter()
                    .map(|x| x.stack_values_in())
                    .reduce(|acc, x| x.intersection(&acc))
                    .unwrap_or_default();
                node.set_stack_values_in(in_stack_n);

                // out[n] = gen[n] U (in[n] - kill[n]) U (callee_saved if n is entry)
                let mut out_reg_n = node
                    .reg_values_in()
                    .difference(&node.node.kill_reg_value())
                    .union(&node.node.gen_reg_value())
                    .union_if(
                        &RegSets::callee_saved().into_available(),
                        node.node.is_any_entry(),
                    );

                // out_stacks[n] = (gen_stacks[n] if we know the location of the stack pointer) U in_stacks[n]
                // (There is no kill_stacks[n])
                let mut out_stack_n = if node.node.is_any_entry() {
                    HashMap::new()
                } else {
                    node.stack_values_in().union_filter_map(
                        &node.node.gen_stack_value(),
                        |(off, val)| {
                            node.reg_values_in()
                                .stack_offset()
                                .map(|curr_stack| (curr_stack + off, val.clone()))
                        },
                    )
                };

                // AVAILABLE VALUE/STACK ESTIMATION
                // ================================
                // We use a series of rules to determine new available values
                // that change our outs.

                rule_expand_address_for_load(&node.node, &mut out_reg_n, &node.reg_values_in());
                rule_perform_math_ops(&node.node, &mut out_reg_n, &node.reg_values_in());
                rule_value_from_stack(&node.node, &mut out_reg_n, &node.stack_values_in());
                rule_known_values_to_stack(&node.node, &mut out_stack_n, node.reg_values_in());

                // If either of the outs changed, replace the old outs with the new outs
                // and mark that we changed something.
                if out_reg_n != node.reg_values_out() {
                    changed = true;
                    node.set_reg_values_out(out_reg_n);
                }
                if out_stack_n != node.stack_values_out() {
                    changed = true;
                    node.set_stack_values_out(out_stack_n);
                }
            }
        }
        Ok(())
    }
}

/// Rule that uses known addresses for load instructions to expand their represenation.
///
/// If a load instruction is found and the register where the address is contains
/// a reference to a register value or memory address, then replace the loaded value
/// with a reference to the specific memory location.
fn rule_expand_address_for_load(
    node: &ParserNode,
    available_out: &mut HashMap<Register, AvailableValue>,
    available_in: &HashMap<Register, AvailableValue>,
) {
    if let Some(reg) = node.stores_to() {
        if let ParserNode::Load(load) = node {
            if let Some(AvailableValue::OriginalRegisterWithScalar(reg, off)) =
                available_in.get(&load.rs1.data)
            {
                available_out.insert(
                    reg.clone(),
                    AvailableValue::MemoryAtOriginalRegister(*reg, *off + load.imm.data.0),
                );
            } else if let Some(AvailableValue::Address(label)) = available_in.get(&load.rs1.data) {
                available_out.insert(
                    reg.data,
                    AvailableValue::Memory(label.clone(), load.imm.data.0),
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
    available_out: &mut HashMap<Register, AvailableValue>,
    available_in: &HashMap<Register, AvailableValue>,
) {
    if let Some(reg) = node.stores_to() {
        let lhs = match node {
            ParserNode::Arith(expr) => available_in
                .get(&expr.rs1.data)
                .map(std::clone::Clone::clone),
            ParserNode::IArith(expr) => available_in
                .get(&expr.rs1.data)
                .map(std::clone::Clone::clone),
            _ => None,
        };

        let rhs = match node {
            ParserNode::Arith(expr) => available_in
                .get(&expr.rs2.data)
                .map(std::clone::Clone::clone),
            ParserNode::IArith(expr) => Some(AvailableValue::Constant(expr.imm.data.0)),
            _ => None,
        };

        let res = match (lhs, rhs) {
            (Some(AvailableValue::Constant(x)), Some(AvailableValue::Constant(y))) => node
                .inst()
                .data
                .math_op()
                .map(|op| op.operate(x, y))
                .map(AvailableValue::Constant),
            (
                Some(AvailableValue::OriginalRegisterWithScalar(reg, x)),
                Some(AvailableValue::Constant(y)),
            )
            | (
                Some(AvailableValue::Constant(x)),
                Some(AvailableValue::OriginalRegisterWithScalar(reg, y)),
            ) => node
                .inst()
                .data
                .scalar_op()
                .map(|op| op.operate(x, y))
                .map(|z| AvailableValue::OriginalRegisterWithScalar(reg, z)),
            (_, _) => None,
        };
        if let Some(val) = res {
            available_out.insert(reg.data, val);
        }
    }
}

/// Rule that restores guaranteed register values from the stack.
///
/// If a register is stored to from a memory location that is the stack, and
/// the stack contains a value at the offset, then store the value from the
/// stack into the register.
fn rule_value_from_stack(
    node: &ParserNode,
    available_out: &mut HashMap<Register, AvailableValue>,
    stack_in: &HashMap<i32, AvailableValue>,
) {
    if let Some(reg) = node.stores_to() {
        if let Some(AvailableValue::MemoryAtOriginalRegister(psp, off)) =
            available_out.get(&reg.data)
        {
            if psp.is_sp() {
                if let Some(stack_val) = stack_in.get(off) {
                    available_out.insert(reg.data, stack_val.clone());
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
    node: &ParserNode,
    stack_out: &mut HashMap<i32, AvailableValue>,
    available_in: HashMap<Register, AvailableValue>,
) {
    for (pos, val) in stack_out.clone() {
        if let AvailableValue::RegisterWithScalar(reg, off) = val {
            if let Some(item) = available_in.get(&reg) {
                match item {
                    AvailableValue::Constant(x) => {
                        stack_out.insert(pos, AvailableValue::Constant(*x + off));
                    }
                    AvailableValue::OriginalRegisterWithScalar(reg2, off3) => {
                        stack_out.insert(
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
