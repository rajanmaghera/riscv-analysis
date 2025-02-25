use std::collections::HashSet;

use crate::parser::node::ParserNode;

use super::{
    BasicType, BranchType, Imm, InstructionProperties, JumpLinkRType, LabelString, Register, With,
};
impl InstructionProperties for ParserNode {
    fn is_return(&self) -> bool {
        match self {
            ParserNode::JumpLinkR(x) => {
                x.inst == JumpLinkRType::Jalr
                    && x.rd == Register::X0
                    && x.rs1 == Register::X1
                    && x.imm == Imm(0)
            }
            ParserNode::Basic(x) => x.inst == BasicType::Uret,
            _ => false,
        }
    }

    fn is_ureturn(&self) -> bool {
        match self {
            ParserNode::Basic(x) => x.inst == BasicType::Uret,
            _ => false,
        }
    }

    fn stores_to_memory(&self) -> Option<(Register, (Register, Imm))> {
        match self {
            ParserNode::Store(x) if x.rs2.data != Register::X0 => {
                Some((x.rs2.data, (x.rs1.data, x.imm.data.clone())))
            }
            _ => None,
        }
    }

    fn reads_from_memory(&self) -> Option<((Register, Imm), Register)> {
        match self {
            ParserNode::Load(x) => Some(((x.rs1.data, x.imm.data.clone()), x.rd.data)),
            _ => None,
        }
    }

    fn can_skip_save_checks(&self) -> bool {
        matches!(
            self,
            ParserNode::ProgramEntry(_)
                | ParserNode::FuncEntry(_)
                | ParserNode::JumpLink(_)
                | ParserNode::JumpLinkR(_)
                | ParserNode::Csr(_)
                | ParserNode::CsrI(_)
        )
    }

    fn calls_to(&self) -> Option<With<LabelString>> {
        match self {
            ParserNode::JumpLink(x) if x.rd == Register::X1 => Some(x.name.clone()),
            _ => None,
        }
    }

    fn is_ecall(&self) -> bool {
        match self {
            ParserNode::Basic(x) => x.inst == BasicType::Ecall,
            _ => false,
        }
    }

    fn jumps_to(&self) -> Option<With<LabelString>> {
        match self {
            ParserNode::JumpLink(x) if x.rd != Register::X1 => Some(x.name.clone()),
            ParserNode::Branch(x) => Some(x.name.clone()),
            _ => None,
        }
    }

    fn reads_address_of(&self) -> Option<With<LabelString>> {
        match self {
            ParserNode::LoadAddr(x) => Some(x.name.clone()),
            _ => None,
        }
    }

    fn is_any_entry(&self) -> bool {
        matches!(self, ParserNode::ProgramEntry(_) | ParserNode::FuncEntry(_))
    }

    fn is_function_entry(&self) -> bool {
        matches!(self, ParserNode::FuncEntry(_))
    }

    fn is_handler_function_entry(&self) -> bool {
        matches!(self, ParserNode::FuncEntry(x) if x.is_interrupt_handler)
    }

    fn is_program_entry(&self) -> bool {
        matches!(self, ParserNode::ProgramEntry(_))
    }

    fn is_instruction(&self) -> bool {
        matches!(
            self,
            ParserNode::Arith(_)
                | ParserNode::IArith(_)
                | ParserNode::JumpLink(_)
                | ParserNode::JumpLinkR(_)
                | ParserNode::Basic(_)
                | ParserNode::Branch(_)
                | ParserNode::Store(_)
                | ParserNode::Load(_)
                | ParserNode::LoadAddr(_)
                | ParserNode::Csr(_)
                | ParserNode::CsrI(_)
        )
    }

    fn uses_memory_location(&self) -> Option<(Register, Imm)> {
        match self {
            ParserNode::Store(s) => Some((s.rs1.data, s.imm.data.clone())),
            ParserNode::Load(l) => Some((l.rs1.data, l.imm.data.clone())),
            _ => None,
        }
    }

    fn is_unconditional_jump(&self) -> bool {
        match self {
            ParserNode::JumpLink(x) if x.rd == Register::X0 => true,
            ParserNode::JumpLinkR(x) if x.rd == Register::X0 => true,
            ParserNode::Branch(x) => {
                x.rs1 == Register::X0
                    && x.rs2 == Register::X0
                    && (x.inst == BranchType::Beq
                        || x.inst == BranchType::Bge
                        || x.inst == BranchType::Bgeu)
            }
            _ => false,
        }
    }

    fn is_some_jump_to_label(&self) -> Option<With<LabelString>> {
        match self {
            ParserNode::JumpLink(x) if x.rd == Register::X0 => Some(x.name.clone()),
            ParserNode::Branch(x) => Some(x.name.clone()),
            _ => None,
        }
    }

    fn writes_to(&self) -> Option<With<Register>> {
        match self {
            ParserNode::Load(load) => Some(load.rd.clone()),
            ParserNode::LoadAddr(load) => Some(load.rd.clone()),
            ParserNode::Arith(arith) => Some(arith.rd.clone()),
            ParserNode::IArith(iarith) => Some(iarith.rd.clone()),
            ParserNode::JumpLink(jump_link) => Some(jump_link.rd.clone()),
            ParserNode::JumpLinkR(jump_link_r) => Some(jump_link_r.rd.clone()),
            ParserNode::Csr(csr) => Some(csr.rd.clone()),
            ParserNode::CsrI(csri) => Some(csri.rd.clone()),
            ParserNode::ProgramEntry(_)
            | ParserNode::FuncEntry(_)
            | ParserNode::Label(_)
            | ParserNode::Basic(_)
            | ParserNode::Directive(_)
            | ParserNode::Branch(_)
            | ParserNode::Store(_) => None,
        }
    }

    fn reads_from(&self) -> HashSet<With<Register>> {
        let vector = match self {
            ParserNode::Arith(x) => vec![x.rs1.clone(), x.rs2.clone()],
            ParserNode::IArith(x) => vec![x.rs1.clone()],
            ParserNode::JumpLinkR(x) => vec![x.rs1.clone()],
            ParserNode::Branch(x) => vec![x.rs1.clone(), x.rs2.clone()],
            ParserNode::Store(x) => vec![x.rs1.clone(), x.rs2.clone()],
            ParserNode::Load(x) => vec![x.rs1.clone()],
            ParserNode::Csr(x) => vec![x.rs1.clone()],
            ParserNode::ProgramEntry(_)
            | ParserNode::FuncEntry(_)
            | ParserNode::Label(_)
            | ParserNode::JumpLink(_)
            | ParserNode::Basic(_)
            | ParserNode::Directive(_)
            | ParserNode::LoadAddr(_)
            | ParserNode::CsrI(_) => vec![],
        };
        vector.into_iter().collect()
    }
}
