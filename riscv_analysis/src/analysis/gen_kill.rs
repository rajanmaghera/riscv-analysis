use crate::{
    cfg::RegisterSet,
    parser::{
        CSRIType, CSRType, HasRegisterSets, IArithType, InstructionProperties, ParserNode,
        Register, RegisterProperties,
    },
};

use super::{AvailableValue, HasGenKillInfo, HasGenValueInfo, MemoryLocation};

impl HasGenKillInfo for ParserNode {
    fn kill_reg(&self) -> RegisterSet {
        (if self.calls_to().is_some() {
            Register::caller_saved_set()
        } else if self.is_function_entry() {
            Register::caller_saved_set()
        } else if let Some(stored_reg) = self.writes_to().map(|x| *x.get()) {
            RegisterSet::from_iter([stored_reg])
        } else {
            RegisterSet::new()
        }) - Register::const_zero_set()
    }

    fn gen_reg(&self) -> RegisterSet {
        (if self.is_ureturn() {
            Register::all_writable_set()
        } else if self.is_return() {
            Register::callee_saved_set()
        } else {
            self.reads_from().into_iter().map(|x| *x.get()).collect()
        }) - Register::const_zero_set()
    }
}

impl HasGenValueInfo for ParserNode {
    fn gen_memory_value(&self) -> Option<(MemoryLocation, AvailableValue)> {
        match self {
            ParserNode::Csr(expr) => match expr.inst.get() {
                CSRType::Csrrw => Some((
                    MemoryLocation::CsrRegister(*expr.csr.get()),
                    AvailableValue::RegisterWithScalar(*expr.rs1.get(), 0),
                )),
                // TODO handle other CSR instructions
                _ => None,
            },
            ParserNode::CsrI(expr) => match expr.inst.get() {
                CSRIType::Csrrwi => Some((
                    MemoryLocation::CsrRegister(*expr.csr.get()),
                    AvailableValue::Constant(expr.imm.get().value()),
                )),
                // TODO handle other CSR instructions
                _ => None,
            },
            ParserNode::Store(expr) => {
                if expr.rs1.get().is_stack_pointer() {
                    Some((
                        MemoryLocation::StackOffset(expr.imm.get().value()),
                        AvailableValue::RegisterWithScalar(*expr.rs2.get(), 0),
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
                Some((expr.rd.get(), AvailableValue::ValueInCsr(*expr.csr.get())))
            }
            ParserNode::CsrI(expr) => {
                Some((expr.rd.get(), AvailableValue::ValueInCsr(*expr.csr.get())))
            }

            ParserNode::LoadAddr(expr) => {
                Some((expr.rd.get(), AvailableValue::Address(expr.name.clone())))
            }
            ParserNode::Load(expr) => Some((
                expr.rd.get(),
                AvailableValue::MemoryAtRegister(*expr.rs1.get(), expr.imm.get().value()),
            )),
            ParserNode::IArith(expr) => {
                if expr.rs1 == Register::X0 {
                    match expr.inst.get() {
                        IArithType::Addi
                        | IArithType::Lui
                        | IArithType::Addiw
                        | IArithType::Xori
                        | IArithType::Ori => Some((
                            expr.rd.get(),
                            AvailableValue::Constant(expr.imm.get().value()),
                        )),
                        IArithType::Andi
                        | IArithType::Slli
                        | IArithType::Slliw
                        | IArithType::Srai
                        | IArithType::Sraiw
                        | IArithType::Srli
                        | IArithType::Srliw => Some((expr.rd.get(), AvailableValue::Constant(0))),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            ParserNode::Arith(expr) => {
                if expr.rs1 == Register::X0 && expr.rs2 == Register::X0 {
                    Some((expr.rd.get(), AvailableValue::Constant(0)))
                } else {
                    None
                }
            }
            _ => None,
        }
        .map(|(x, y)| (*x, y));

        if let Some((reg, _)) = item {
            if reg == Register::X0 {
                return None;
            }
        }

        item
    }
}
