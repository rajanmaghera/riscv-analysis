use crate::{
    cfg::RegisterSet,
    parser::{IArithType, ParserNode, RegSets, Register},
};

use super::{AvailableValue, MemoryLocation};

impl ParserNode {
    #[must_use]
    pub fn kill_reg_value(&self) -> RegisterSet {
        if self.calls_to().is_some() {
            RegSets::caller_saved() | Register::X1
        } else {
            self.kill_reg()
        }
    }

    #[must_use]
    pub fn kill_reg(&self) -> RegisterSet {
        if self.calls_to().is_some() {
            RegisterSet::new()
        } else if self.is_function_entry() {
            RegSets::caller_saved()
        } else if let Some(stored_reg) = self.stores_to().map(|x| x.data) {
            if stored_reg == Register::X0 {
                RegisterSet::new()
            } else {
                [stored_reg].into_iter().collect()
            }
        } else {
            RegisterSet::new()
        }
    }

    #[must_use]
    pub fn gen_reg(&self) -> RegisterSet {
        let regs = if self.is_return() {
            RegSets::callee_saved()
        } else {
            self.reads_from().iter().map(|x| x.data).collect()
        };

        regs - Register::X0
    }

    #[must_use]
    pub fn gen_memory_value(&self) -> Option<(MemoryLocation, AvailableValue)> {
        match self {
            ParserNode::Store(expr) => {
                if expr.rs1 == Register::X2 {
                    Some((
                        MemoryLocation::StackOffset(expr.imm.data.0),
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
