use crate::{
    cfg::RegisterSet,
    parser::{
        CSRIType, CSRType, HasRegisterSets, IArithType, InstructionProperties, ParserNode, Register,
    },
};

use super::{AvailableValue, HasGenKillInfo, HasGenValueInfo, MemoryLocation};

impl HasGenKillInfo for ParserNode {
    fn kill_reg(&self) -> RegisterSet {
        if self.calls_to().is_some() {
            Register::caller_saved_set()
        } else if self.is_function_entry() {
            Register::caller_saved_set()
        } else if let Some(stored_reg) = self.writes_to().map(|x| x.data) {
            if stored_reg == Register::X0 {
                RegisterSet::new()
            } else {
                [stored_reg].into_iter().collect()
            }
        } else {
            RegisterSet::new()
        }
    }

    fn gen_reg(&self) -> RegisterSet {
        let regs = if self.is_ureturn() {
            Register::all_writable_set()
        } else if self.is_return() {
            Register::callee_saved_set()
        } else {
            self.reads_from().iter().map(|x| x.data).collect()
        };

        regs - Register::X0
    }
}

impl HasGenValueInfo for ParserNode {
    fn gen_memory_value(&self) -> Option<(MemoryLocation, AvailableValue)> {
        match self {
            ParserNode::Csr(expr) => match expr.inst.data {
                CSRType::Csrrw => Some((
                    MemoryLocation::CsrRegister(expr.csr.data),
                    AvailableValue::RegisterWithScalar(expr.rs1.data, 0),
                )),
                // TODO handle other CSR instructions
                _ => None,
            },
            ParserNode::CsrI(expr) => match expr.inst.data {
                CSRIType::Csrrwi => Some((
                    MemoryLocation::CsrRegister(expr.csr.data),
                    AvailableValue::Constant(expr.imm.data.0),
                )),
                // TODO handle other CSR instructions
                _ => None,
            },
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

    fn gen_reg_value(&self) -> Option<(Register, AvailableValue)> {
        // The function entry case and program entry case is handled separately
        // to account for all the "original" registers.
        let item = match self {
            ParserNode::Csr(expr) => {
                Some((expr.rd.data, AvailableValue::ValueInCsr(expr.csr.data)))
            }
            ParserNode::CsrI(expr) => {
                Some((expr.rd.data, AvailableValue::ValueInCsr(expr.csr.data)))
            }

            ParserNode::LoadAddr(expr) => {
                Some((expr.rd.data, AvailableValue::Address(expr.name.clone())))
            }
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
        };

        if let Some((reg, _)) = &item {
            if *reg == Register::X0 {
                return None;
            }
        }

        item
    }
}
