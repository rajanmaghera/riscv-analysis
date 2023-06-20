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
use crate::parser::inst::IArithType;
use crate::parser::{ast::ASTNode, register::Register};

use super::DirectionalWrapper;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AvailableValue {
    Constant(i32),
    MemAddr(LabelString),     // Address of some memory location (ex. la ___)
    Memory(LabelString, i32), // Actual bit of memory + offset (ex. lw ___)
    CurrScalarOffset(Register, i32), // Value of register currently + SCALAR offset (ex. addi ___)
    OrigScalarOffset(Register, i32), // Value of register at function entrance or start of main program+ SCALAR offset (ex. addi ___)
    CurrMemReg(Register, i32), // Actual bit of memory + offset (ex. lw ___), where we do not know the label
    OrigMemReg(Register, i32), // Actual bit of memory + offset (ex. lw ___), where we are sure it is the same as the original
}

pub trait AvailableRegisterValues {
    fn is_original_value(&self, reg: &Register) -> bool;
}

impl AvailableRegisterValues for &HashMap<Register, AvailableValue> {
    fn is_original_value(&self, reg: &Register) -> bool {
        self.get(reg)
            .map(|x| match x {
                AvailableValue::OrigScalarOffset(reg2, offset) => reg == reg2 && offset == &0,
                _ => false,
            })
            .unwrap_or(false)
    }
}

impl Display for AvailableValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AvailableValue::Constant(v) => write!(f, "{}", v),
            AvailableValue::MemAddr(a) => write!(f, "{}", a),
            AvailableValue::Memory(a, off) => write!(f, "{}({})", off, a),
            AvailableValue::CurrMemReg(reg, off) => write!(f, "{}(<{}>?)", off, reg),
            AvailableValue::CurrScalarOffset(reg, off) => {
                if off == &0 {
                    write!(f, "<{}>?", reg)
                } else {
                    write!(f, "<{}>? + {}", reg, off)
                }
            }
            AvailableValue::OrigScalarOffset(reg, off) => {
                if off == &0 {
                    write!(f, "{}", reg)
                } else {
                    write!(f, "{} + {}", reg, off)
                }
            }
            AvailableValue::OrigMemReg(reg, off) => write!(f, "{}({})", off, reg),
        }
    }
}

impl ASTNode {
    pub fn gen_stack(&self) -> Option<(i32, AvailableValue)> {
        match self {
            ASTNode::Store(expr) => {
                if expr.rs1 == Register::X2 {
                    Some((
                        expr.imm.data.0,
                        AvailableValue::CurrScalarOffset(expr.rs2.data, 0),
                    ))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    pub fn gen_value(&self) -> Option<(Register, AvailableValue)> {
        match self {
            // function entry case is handled separately
            ASTNode::LoadAddr(expr) => Some((
                expr.rd.data,
                AvailableValue::MemAddr(expr.name.data.clone()),
            )),
            ASTNode::Load(expr) => Some((
                expr.rd.data,
                AvailableValue::CurrMemReg(expr.rs1.data, expr.imm.data.0),
            )),
            ASTNode::IArith(expr) => {
                if expr.rs1 == Register::X0 {
                    match expr.inst.data {
                        IArithType::Addi
                        | IArithType::Addiw
                        | IArithType::Xori
                        | IArithType::Ori => {
                            Some((expr.rd.data, AvailableValue::Constant(expr.imm.data.0)))
                        }
                        IArithType::Andi
                        | IArithType::Slli
                        | IArithType::Slliw
                        | IArithType::Srai
                        | IArithType::Sraiw
                        | IArithType::Srli
                        | IArithType::Srliw => Some((expr.rd.data, AvailableValue::Constant(0))),
                        IArithType::Slti => todo!(),
                        IArithType::Sltiu => todo!(),
                        IArithType::Auipc => todo!(),
                    }
                } else {
                    None
                }
            }
            ASTNode::Arith(expr) => {
                if expr.rs1 == Register::X0 && expr.rs2 == Register::X0 {
                    Some((expr.rd.data, AvailableValue::Constant(0)))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// --- VALUES ---

#[derive(Clone)]
pub struct AvailableValueResult {
    pub avail_in: Vec<HashMap<Register, AvailableValue>>,
    pub avail_out: Vec<HashMap<Register, AvailableValue>>,
    // For now, we are specializing to the stack only, but we could generalize to any
    // memory location
    pub stack_in: Vec<HashMap<i32, AvailableValue>>,
    pub stack_out: Vec<HashMap<i32, AvailableValue>>,
}

impl DirectionalWrapper {
    pub fn available_value_analysis(&self) -> AvailableValueResult {
        #[derive(Clone)]
        struct AvailableValueNodeData {
            node: Rc<ASTNode>,
            ins: HashMap<Register, AvailableValue>,
            outs: HashMap<Register, AvailableValue>,
            stack_ins: HashMap<i32, AvailableValue>,
            stack_outs: HashMap<i32, AvailableValue>,
            prevs: HashSet<Rc<ASTNode>>,
        }

        let mut nodes = Vec::new();
        let mut astidx = HashMap::new();

        let mut idx = 0;
        for block in &self.cfg.blocks {
            for node in block.0.iter() {
                nodes.push(AvailableValueNodeData {
                    node: node.clone(),
                    ins: HashMap::new(),
                    outs: HashMap::new(),
                    stack_ins: HashMap::new(),
                    stack_outs: HashMap::new(),
                    prevs: self.prev_ast_map.get(node).unwrap().clone(),
                });
                astidx.insert(node.clone(), idx);
                idx += 1;
            }
        }

        let mut changed = true;
        while changed {
            changed = false;
            for i in (0..nodes.len()).rev() {
                let mut node = nodes.get(i).unwrap().clone();

                // TODO function for iterator intersection
                // TODO traits for most common check intos
                // in[n] = AND out[p] for all p in prev[n]
                let mut in_vals = HashMap::new();
                let mut prev_vals = node.prevs.clone().into_iter().map(|x| {
                    let prev = nodes.get(astidx.get(&x).unwrap().clone()).unwrap();
                    prev.outs
                        .clone()
                        .into_iter()
                        .map(|(x, y)| (x, y.clone()))
                        .collect::<HashSet<_>>()
                });

                if let Some(s) = prev_vals.next() {
                    let mut s = s;
                    for x in prev_vals {
                        s = s.intersection(&x).cloned().collect();
                    }
                    in_vals = s.into_iter().collect();
                }

                let mut in_stacks = HashMap::new();
                let mut prev_stacks = node.prevs.clone().into_iter().map(|x| {
                    let prev = nodes.get(astidx.get(&x).unwrap().clone()).unwrap();
                    prev.stack_outs
                        .clone()
                        .into_iter()
                        .map(|(x, y)| (x, y.clone()))
                        .collect::<HashSet<_>>()
                });

                if let Some(s) = prev_stacks.next() {
                    let mut s = s;
                    for x in prev_stacks {
                        s = s.intersection(&x).cloned().collect();
                    }
                    in_stacks = s.into_iter().collect();
                }
                node.ins = in_vals;
                node.stack_ins = in_stacks;

                // out[n] = gen[n] U (in[n] - kill[n])
                let mut out_vals = node.ins.clone();
                let mut out_stacks = node.stack_ins.clone();

                // take out kill value
                for reg in node.node.kill_available_value() {
                    out_vals.remove(&reg);
                }

                if let Some((reg, val)) = node.node.gen_value() {
                    out_vals.insert(reg, val);
                }
                if let Some((off, val)) = node.node.gen_stack() {
                    if let Some(sp) = node.ins.get(&Register::X2) {
                        if let AvailableValue::OrigScalarOffset(reg, x) = sp {
                            if reg == &Register::X2 {
                                out_stacks.insert(*x + off, val);
                            }
                        }
                    }
                }
                if let ASTNode::FuncEntry(_) | ASTNode::ProgramEntry(_) = &*(node.node) {
                    use Register::*;
                    for reg in vec![
                        X1, X2, X8, X9, X18, X19, X20, X21, X22, X23, X24, X25, X26, X27,
                    ] {
                        out_vals.insert(reg, AvailableValue::OrigScalarOffset(reg, 0));
                    }
                    out_stacks.clear();
                }
                // perform operation estimate
                if let Some(reg) = node.node.stores_to() {
                    if let Some(val) = perform_operation(&node.ins, &node.node) {
                        out_vals.insert(reg.data, val);
                    }

                    if let Some(val) = out_vals.get(&reg.data) {
                        match val {
                            AvailableValue::OrigMemReg(psp, off) => {
                                if psp == &Register::X2 {
                                    if let Some(stack_val) = node.stack_ins.get(&off) {
                                        out_vals.insert(
                                            reg.data,
                                            stack_val.clone(), // TODO good idea? or should I go case by case
                                        );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                // perform stack operation estimates
                for (pos, val) in out_stacks.clone() {
                    match val {
                        AvailableValue::CurrScalarOffset(reg, off) => {
                            if let Some(item) = node.ins.get(&reg) {
                                match item {
                                    AvailableValue::Constant(x) => {
                                        out_stacks.insert(pos, AvailableValue::Constant(*x + off));
                                    }
                                    AvailableValue::OrigScalarOffset(reg2, off3) => {
                                        out_stacks.insert(
                                            pos,
                                            AvailableValue::OrigScalarOffset(*reg2, *off3 + off),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }

                if out_vals != node.outs {
                    changed = true;
                    node.outs = out_vals;
                }
                if out_stacks != node.stack_outs {
                    changed = true;
                    node.stack_outs = out_stacks;
                }

                nodes[i] = node;
            }
        }

        let mut avail_in = Vec::new();
        let mut avail_out = Vec::new();
        let mut stack_in = Vec::new();
        let mut stack_out = Vec::new();
        for node in nodes {
            avail_in.push(node.ins);
            avail_out.push(node.outs);
            stack_in.push(node.stack_ins);
            stack_out.push(node.stack_outs);
        }
        AvailableValueResult {
            avail_in,
            avail_out,
            stack_in,
            stack_out,
        }
    }
}

// statically perform operation and return new available value
fn perform_operation(
    ins: &HashMap<Register, AvailableValue>,
    node: &ASTNode,
) -> Option<AvailableValue> {
    if let ASTNode::Load(load) = node {
        if let Some(AvailableValue::OrigScalarOffset(reg, off)) = ins.get(&load.rs1.data) {
            return Some(AvailableValue::OrigMemReg(*reg, *off + load.imm.data.0));
        } else if let Some(AvailableValue::MemAddr(label)) = ins.get(&load.rs1.data) {
            return Some(AvailableValue::Memory(label.clone(), load.imm.data.0));
        }
        return None;
    }

    let lhs = match node {
        ASTNode::Arith(expr) => ins.get(&expr.rs1.data).map(|x| x.clone()),
        ASTNode::IArith(expr) => ins.get(&expr.rs1.data).map(|x| x.clone()),
        _ => None,
    };

    let rhs = match node {
        ASTNode::Arith(expr) => ins.get(&expr.rs2.data).map(|x| x.clone()),
        ASTNode::IArith(expr) => Some(AvailableValue::Constant(expr.imm.data.0)),
        _ => None,
    };

    match (lhs, rhs) {
        (Some(AvailableValue::Constant(x)), Some(AvailableValue::Constant(y))) => node
            .inst()
            .data
            .math_op()
            .map(|op| op.operate(x, y))
            .map(|x| AvailableValue::Constant(x)),
        (Some(AvailableValue::OrigScalarOffset(reg, x)), Some(AvailableValue::Constant(y)))
        | (Some(AvailableValue::Constant(x)), Some(AvailableValue::OrigScalarOffset(reg, y))) => {
            node.inst()
                .data
                .scalar_op()
                .map(|op| op.operate(x, y))
                .map(|z| AvailableValue::OrigScalarOffset(reg, z))
        }
        (_, _) => None,
    }
}
