use std::collections::HashSet;

use crate::parser::node::ParserNode;

use super::{
    BasicType, BranchType, Imm, InstructionProperties, JumpLinkRType, LabelStringToken, Register,
    RegisterToken,
};
impl InstructionProperties for ParserNode {
    fn is_return(&self) -> bool {
        match self {
            ParserNode::JumpLinkR(x) => {
                x.inst == JumpLinkRType::Jalr
                    && x.rd == Register::X0
                    && x.rs1 == Register::X1
                    && x.imm.get().value() == 0
            }
            ParserNode::Basic(x) => x.inst == BasicType::Uret,
            _ => false,
        }
    }

    fn might_terminate(&self) -> bool {
        self.is_ecall()
    }

    fn is_ureturn(&self) -> bool {
        match self {
            ParserNode::Basic(x) => x.inst == BasicType::Uret,
            _ => false,
        }
    }

    fn stores_to_memory(&self) -> Option<(Register, (Register, Imm))> {
        match self {
            ParserNode::Store(x) if x.rs2 != Register::X0 => {
                Some((x.rs2.get_cloned(), (x.rs1.get_cloned(), x.imm.get_cloned())))
            }
            _ => None,
        }
    }

    fn reads_from_memory(&self) -> Option<((Register, Imm), Register)> {
        match self {
            ParserNode::Load(x) => {
                Some(((x.rs1.get_cloned(), x.imm.get_cloned()), x.rd.get_cloned()))
            }
            _ => None,
        }
    }

    fn can_skip_save_checks(&self) -> bool {
        matches!(
            self,
            ParserNode::JumpLink(_)
                | ParserNode::JumpLinkR(_)
                | ParserNode::Csr(_)
                | ParserNode::CsrI(_)
        )
    }

    fn calls_to(&self) -> Option<LabelStringToken> {
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

    fn jumps_to(&self) -> Option<LabelStringToken> {
        match self {
            ParserNode::JumpLink(x) if x.rd != Register::X1 => Some(x.name.clone()),
            ParserNode::Branch(x) => Some(x.name.clone()),
            _ => None,
        }
    }

    fn reads_address_of(&self) -> Option<LabelStringToken> {
        match self {
            ParserNode::LoadAddr(x) => Some(x.name.clone()),
            _ => None,
        }
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
            ParserNode::Store(s) => Some((s.rs1.get_cloned(), s.imm.get_cloned())),
            ParserNode::Load(l) => Some((l.rs1.get_cloned(), l.imm.get_cloned())),
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

    fn is_some_jump_to_label(&self) -> Option<LabelStringToken> {
        match self {
            ParserNode::JumpLink(x) if x.rd == Register::X0 => Some(x.name.clone()),
            ParserNode::Branch(x) => Some(x.name.clone()),
            _ => None,
        }
    }

    fn writes_to(&self) -> HashSet<RegisterToken> {
        match self {
            ParserNode::Load(load) => [load.rd.clone()].into(),
            ParserNode::LoadAddr(load) => [load.rd.clone()].into(),
            ParserNode::Arith(arith) => [arith.rd.clone()].into(),
            ParserNode::IArith(iarith) => [iarith.rd.clone()].into(),
            ParserNode::JumpLink(jump_link) => [jump_link.rd.clone()].into(),
            ParserNode::JumpLinkR(jump_link_r) => [jump_link_r.rd.clone()].into(),
            ParserNode::Csr(csr) => [csr.rd.clone()].into(),
            ParserNode::CsrI(csri) => [csri.rd.clone()].into(),
            ParserNode::Basic(_) | ParserNode::Branch(_) | ParserNode::Store(_) => HashSet::new(),
        }
    }

    fn reads_from(&self) -> HashSet<RegisterToken> {
        let vector = match self {
            ParserNode::Arith(x) => vec![x.rs1.clone(), x.rs2.clone()],
            ParserNode::IArith(x) => vec![x.rs1.clone()],
            ParserNode::JumpLinkR(x) => vec![x.rs1.clone()],
            ParserNode::Branch(x) => vec![x.rs1.clone(), x.rs2.clone()],
            ParserNode::Store(x) => vec![x.rs1.clone(), x.rs2.clone()],
            ParserNode::Load(x) => vec![x.rs1.clone()],
            ParserNode::Csr(x) => vec![x.rs1.clone()],
            ParserNode::JumpLink(_)
            | ParserNode::Basic(_)
            | ParserNode::LoadAddr(_)
            | ParserNode::CsrI(_) => vec![],
        };
        vector.into_iter().collect()
    }
}
