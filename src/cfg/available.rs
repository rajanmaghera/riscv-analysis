// AVAILABLE VALUE ANALYSIS
// ========================

// This module contains the logic for determining which values are available
// at each point in the program. This is used to guess which ecall is being
// called, and to determine whether stack stores are done to the same location

/* As part of the subset of RISC-V programs that we allow, stack pointer manipulation
 * and ecall arguments must be able to be unconditionally during stack stores.
 */

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::rc::Rc;

use crate::parser::ast::LabelString;
use crate::parser::inst::{IArithType, Inst};
use crate::parser::{ast::ASTNode, register::Register};

use super::DirectionalWrapper;

// TODO FUNCTION PROPOGATION

// Option/None represents a value that does not get overwritten
// UNKNOWN represents a value that is not known, and is GARBAGE
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AvailableValue {
    // TODO constant to scalar value + ZERO?
    Constant(i32),
    MemAddr(LabelString),        // Address of some memory location (ex. la ___)
    Memory(LabelString, i32),    // Actual bit of memory + offset (ex. lw ___)
    ScalarOffset(Register, i32), // Value of register + SCALAR offset (ex. addi ___)
    MemReg(Register, i32), // Actual bit of memory + offset (ex. lw ___), where we do not know the label
}

impl Display for AvailableValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AvailableValue::Constant(v) => write!(f, "{}", v),
            AvailableValue::MemAddr(a) => write!(f, "{}", a),
            AvailableValue::Memory(a, off) => write!(f, "{}({})", off, a),
            AvailableValue::MemReg(reg, off) => write!(f, "{}({})", off, reg),
            AvailableValue::ScalarOffset(reg, off) => write!(f, "{} + {}", reg, off),
        }
    }
}
pub struct AvailableValueResult {
    pub avail_in: Vec<HashMap<Register, AvailableValue>>,
    pub avail_out: Vec<HashMap<Register, AvailableValue>>,
}
// statically perform operation and return new available value
fn perform_operation(
    ins: &HashMap<Register, AvailableValue>,
    node: &ASTNode,
) -> Option<AvailableValue> {
    let lhs = match node {
        ASTNode::Arith(expr) => ins.get(&expr.rs1.data).map(|x| x.clone()),
        ASTNode::IArith(expr) => ins.get(&expr.rs1.data).map(|x| x.clone()),
        ASTNode::Load(expr) => ins.get(&expr.rs1.data).map(|x| x.clone()),
        ASTNode::LoadAddr(expr) => Some(AvailableValue::MemAddr(expr.name.data.clone())),
        _ => None,
    };

    let rhs = match node {
        ASTNode::Arith(expr) => ins.get(&expr.rs2.data).map(|x| x.clone()),
        ASTNode::IArith(expr) => Some(AvailableValue::Constant(expr.imm.data.0)),
        ASTNode::Load(expr) => Some(AvailableValue::Constant(expr.imm.data.0)),
        ASTNode::LoadAddr(_) => Some(AvailableValue::Constant(0)),
        _ => None,
    };

    if node.inst().data == Inst::Addi {
        dbg!(&lhs, &rhs);
    }

    match (lhs, rhs) {
        (Some(AvailableValue::Constant(x)), Some(AvailableValue::Constant(y))) => node
            .inst()
            .data
            .math_op()
            .map(|op| op.operate(x, y))
            .map(|x| AvailableValue::Constant(x)),
        (Some(AvailableValue::ScalarOffset(reg, x)), Some(AvailableValue::Constant(y)))
        | (Some(AvailableValue::Constant(x)), Some(AvailableValue::ScalarOffset(reg, y))) => node
            .inst()
            .data
            .scalar_op()
            .map(|op| op.operate(x, y))
            .map(|z| AvailableValue::ScalarOffset(reg, z)),
        (_, _) => None,
    }
}
