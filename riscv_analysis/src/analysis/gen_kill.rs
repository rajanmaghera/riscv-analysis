use std::collections::HashSet;

use crate::parser::{IArithType, ParserNode, RegSets, Register};

use super::AvailableValue;

impl ParserNode {
    #[must_use]
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

    #[must_use]
    pub fn kill_reg(&self) -> HashSet<Register> {
        let regs = if self.calls_to().is_some() {
            HashSet::new()
        } else if self.is_function_entry() {
            RegSets::caller_saved()
        } else {
            self.stores_to()
                .map(|x| vec![x.data].into_iter().collect())
                .unwrap_or_default()
        };

        regs.into_iter()
            .filter(|x| *x != Register::X0)
            .collect::<HashSet<_>>()
    }

    #[must_use]
    pub fn gen_reg(&self) -> HashSet<Register> {
        let regs = if self.is_return() {
            RegSets::callee_saved()
        } else {
            self.reads_from().into_iter().map(|x| x.data).collect()
        };
        regs.into_iter()
            .filter(|x| *x != Register::X0)
            .collect::<HashSet<_>>()
    }

    #[must_use]
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

    #[must_use]
    /// # Panics
    ///
    /// TODO remove this panic
    pub fn gen_reg_value(&self) -> Option<(Register, AvailableValue)> {
        // The function entry case and program entry case is handled separately
        // to account for all the "original" registers.
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
                        | IArithType::Lui
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
                        _ => None,
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
