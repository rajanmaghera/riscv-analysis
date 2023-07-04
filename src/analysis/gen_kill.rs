use std::collections::HashSet;

use crate::parser::{IArithType, ParserNode, RegSets, Register};

use super::AvailableValue;

impl ParserNode {
    pub fn kill_reg_value(&self) -> HashSet<Register> {
        match self.clone() {
            ParserNode::FuncEntry(_) => RegSets::caller_saved(),
            ParserNode::JumpLink(x) => {
                // If a jump and link instruction is a call to a function, denoted
                // by the program counter storing to the ra register, then all
                // the caller saved registers are killed.
                let mut set = if x.rd.data == Register::X1 {
                    RegSets::caller_saved()
                } else {
                    HashSet::new()
                };
                set.insert(x.rd.data);
                set
            }
            _ => self.kill_reg(),
        }
    }

    pub fn kill_reg(&self) -> HashSet<Register> {
        let regs: HashSet<Register> = match self.clone() {
            ParserNode::FuncEntry(_) => RegSets::callee_saved(),
            ParserNode::JumpLink(_) if self.calls_to().is_some() => HashSet::new(),
            _ => self
                .stores_to()
                .map(|x| vec![x.data].into_iter().collect())
                .unwrap_or_default(),
        };
        regs.into_iter()
            .filter(|x| *x != Register::X0)
            .collect::<HashSet<_>>()
    }

    pub fn gen_reg(&self) -> HashSet<Register> {
        let regs: HashSet<Register> = match self {
            _ if self.is_return() => RegSets::callee_saved(),
            _ => self.reads_from().into_iter().map(|x| x.data).collect(),
        };
        regs.into_iter()
            .filter(|x| *x != Register::X0)
            .collect::<HashSet<_>>()
    }

    pub fn gen_stack_value(&self) -> Option<(i32, AvailableValue)> {
        match self {
            ParserNode::Store(expr) => {
                if expr.rs1 == Register::X2 {
                    Some((
                        expr.imm.data.0,
                        AvailableValue::RegisterWithScalar(expr.rs2.data, 0),
                    ))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    pub fn gen_reg_value(&self) -> Option<(Register, AvailableValue)> {
        // The function entry case and program entry case is handled separately
        // to account for all the "original" registers.
        // TODO do registers need to be saved at program entry?
        match self {
            ParserNode::LoadAddr(expr) => Some((
                expr.rd.data,
                AvailableValue::Address(expr.name.data.clone()),
            )),
            ParserNode::Load(expr) => Some((
                expr.rd.data,
                AvailableValue::MemoryAtRegister(expr.rs1.data, expr.imm.data.0),
            )),
            ParserNode::IArith(expr) => {
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
            ParserNode::Arith(expr) => {
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
