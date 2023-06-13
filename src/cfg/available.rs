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

impl ASTNode {
    pub fn gen_value(&self) -> Option<(Register, AvailableValue)> {
        match self {
            // TODO include all saved registers
            ASTNode::FuncEntry(_) => {
                Some((Register::X2, AvailableValue::ScalarOffset(Register::X2, 0)))
            }
            ASTNode::LoadAddr(expr) => Some((
                expr.rd.data,
                AvailableValue::MemAddr(expr.name.data.clone()),
            )),
            ASTNode::Load(expr) => Some((
                expr.rd.data,
                AvailableValue::MemReg(expr.rs1.data, expr.imm.data.0),
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

// Memory locations/stack and values are tracked separately

// --- VALUES ---

pub struct AvailableValueResult {
    pub avail_in: Vec<HashMap<Register, AvailableValue>>,
    pub avail_out: Vec<HashMap<Register, AvailableValue>>,
}

impl DirectionalWrapper {
    pub fn available_value_analysis(&self) -> AvailableValueResult {
        #[derive(Clone)]
        struct AvailableValueNodeData {
            node: Rc<ASTNode>,
            ins: HashMap<Register, AvailableValue>,
            outs: HashMap<Register, AvailableValue>,
            prevs: HashSet<Rc<ASTNode>>,
        }

        // TODO differenciate between unknown value and no value
        let mut nodes = Vec::new();
        let mut astidx = HashMap::new();

        let mut idx = 0;
        for block in &self.cfg.blocks {
            for node in block.0.iter() {
                let mut out_vals = HashMap::new();
                if let Some((reg, val)) = node.gen_value() {
                    out_vals.insert(reg, val);
                }

                nodes.push(AvailableValueNodeData {
                    node: node.clone(),
                    ins: HashMap::new(),
                    outs: out_vals,
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

                // in[n] = AND out[p] for all p in prev[n]
                let mut in_vals = HashMap::new();
                if node.prevs.len() > 0 {
                    let prev = node.prevs.iter().next().unwrap();
                    in_vals = nodes
                        .get(astidx.get(prev).unwrap().clone())
                        .unwrap()
                        .outs
                        .clone();
                    for prev in &node.prevs {
                        let prev = nodes.get(astidx.get(prev).unwrap().clone()).unwrap();
                        for (reg, val) in &prev.outs {
                            if let Some(prev_val) = in_vals.get(reg) {
                                if prev_val != val {
                                    in_vals.remove(reg);
                                }
                            } else {
                                in_vals.insert(reg.clone(), val.clone());
                            }
                        }
                    }
                }
                node.ins = in_vals;
                // TODO separate AvailableValue into two structs
                // with GenValue (ex. Unavailable)

                // out[n] = gen[n] U (in[n] - kill[n])
                let mut out_vals = node.ins.clone();
                // take out kill value
                for reg in node.node.kill_value() {
                    out_vals.remove(&reg);
                }
                // perform operation estimate
                if let Some((reg, val)) = node.node.gen_value() {
                    match val {
                        AvailableValue::Constant(v) => {
                            out_vals.insert(reg, AvailableValue::Constant(v));
                        }
                        AvailableValue::MemAddr(a) => {
                            out_vals.insert(reg, AvailableValue::MemAddr(a));
                        }
                        AvailableValue::Memory(l, off) => {
                            out_vals.insert(reg, AvailableValue::Memory(l, off));
                        }
                        AvailableValue::MemReg(src, off) => {
                            if let Some(val) = node.ins.get(&src) {
                                match val {
                                    AvailableValue::MemAddr(a) => {
                                        out_vals
                                            .insert(reg, AvailableValue::Memory(a.clone(), off));
                                    }
                                    AvailableValue::Constant(v) => {
                                        out_vals.insert(reg, AvailableValue::Constant(v + off));
                                    }
                                    _ => {
                                        out_vals.insert(reg, AvailableValue::MemReg(src, off));
                                    }
                                };
                            }
                        }
                        AvailableValue::ScalarOffset(src, off) => {
                            out_vals.insert(reg, AvailableValue::ScalarOffset(src, off));
                        }
                    };
                }
                if let Some(reg) = node.node.stores_to() {
                    if let Some(val) = perform_operation(&node.ins, &node.node) {
                        out_vals.insert(reg.data, val);
                    }
                }

                if out_vals != node.outs {
                    changed = true;
                    node.outs = out_vals;
                }

                nodes[i] = node;
            }
        }

        let mut avail_in = Vec::new();
        let mut avail_out = Vec::new();
        for node in nodes {
            avail_in.push(node.ins);
            avail_out.push(node.outs);
        }
        AvailableValueResult {
            avail_in,
            avail_out,
        }
    }
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
