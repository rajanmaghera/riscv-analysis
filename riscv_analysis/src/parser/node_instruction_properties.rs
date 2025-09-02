use crate::cfg::Segment;
use crate::parser::node::RVInstructionNode;
use std::collections::HashSet;

use super::{
    BasicType, BranchType, Imm, InstructionProperties, JumpLinkRType, JumpLinkType, JumpTarget,
    LabelStringToken, RVRegister, RegisterToken, With,
};
impl InstructionProperties for RVInstructionNode {
    fn is_return(&self) -> bool {
        match self {
            RVInstructionNode::JumpLinkR(x) => {
                x.inst == JumpLinkRType::Jalr
                    && x.rd == RVRegister::X0
                    && x.rs1 == RVRegister::X1
                    && x.imm.get().value() == Some(0)
            }
            RVInstructionNode::Basic(x) => x.inst == BasicType::Uret,
            _ => false,
        }
    }

    fn might_terminate(&self) -> bool {
        match self {
            RVInstructionNode::Basic(x) => match x.inst.get() {
                // Compiled code uses Ebreak for debugging purposes (e.g. undefined behaviour)
                // For our purposes, we treat it as terminating
                BasicType::Ecall | BasicType::Ebreak => true,
                BasicType::Uret => false,
            },
            _ => false,
        }
    }

    fn is_ureturn(&self) -> bool {
        match self {
            RVInstructionNode::Basic(x) => x.inst == BasicType::Uret,
            _ => false,
        }
    }

    fn stores_to_memory(&self) -> Option<(RVRegister, (RVRegister, Imm))> {
        match self {
            RVInstructionNode::Store(x) if x.rs2 != RVRegister::X0 => {
                Some((x.rs2.get_cloned(), (x.rs1.get_cloned(), x.imm.get_cloned())))
            }
            _ => None,
        }
    }

    fn reads_from_memory(&self) -> Option<((RVRegister, Imm), RVRegister)> {
        match self {
            RVInstructionNode::Load(x) => {
                Some(((x.rs1.get_cloned(), x.imm.get_cloned()), x.rd.get_cloned()))
            }
            _ => None,
        }
    }

    fn can_skip_save_checks(&self) -> bool {
        matches!(
            self,
            RVInstructionNode::JumpLink(_)
                | RVInstructionNode::JumpLinkR(_)
                | RVInstructionNode::Csr(_)
                | RVInstructionNode::CsrI(_)
        ) || self.segment() != Segment::Text
    }

    fn calls_to(&self) -> Option<JumpTarget> {
        match self {
            RVInstructionNode::JumpLink(x) if x.rd == RVRegister::X1 => {
                Some(JumpTarget::Label(x.name.clone()))
            }
            RVInstructionNode::JumpLinkR(x) if x.rd == RVRegister::X1 => {
                Some(JumpTarget::Register(x.rs1.clone()))
            }
            _ => None,
        }
    }

    fn is_ecall(&self) -> bool {
        match self {
            RVInstructionNode::Basic(x) => x.inst == BasicType::Ecall,
            _ => false,
        }
    }

    fn jumps_to(&self) -> Option<JumpTarget> {
        match self {
            RVInstructionNode::JumpLink(x) if x.rd != RVRegister::X1 => {
                Some(JumpTarget::Label(x.name.clone()))
            }
            RVInstructionNode::JumpLinkR(x) if x.rd != RVRegister::X1 => {
                Some(JumpTarget::Register(x.rs1.clone()))
            }
            RVInstructionNode::Branch(x) => Some(JumpTarget::Label(x.name.clone())),
            _ => None,
        }
    }

    fn is_indirect_uncond_jump(&self) -> bool {
        match self {
            RVInstructionNode::JumpLinkR(_) => true,
            _ => false,
        }
    }

    fn is_direct_uncond_jump(&self) -> bool {
        match self {
            RVInstructionNode::JumpLink(_) => true,
            _ => false,
        }
    }

    fn is_cond_branch(&self) -> bool {
        match self {
            RVInstructionNode::Branch(_) => true,
            _ => false,
        }
    }

    fn reads_address_of(&self) -> Option<LabelStringToken> {
        match self {
            RVInstructionNode::LoadAddr(x) => Some(x.name.clone()),
            _ => None,
        }
    }

    fn is_instruction(&self) -> bool {
        matches!(
            self,
            RVInstructionNode::Arith(_)
                | RVInstructionNode::IArith(_)
                | RVInstructionNode::JumpLink(_)
                | RVInstructionNode::JumpLinkR(_)
                | RVInstructionNode::Basic(_)
                | RVInstructionNode::Branch(_)
                | RVInstructionNode::Store(_)
                | RVInstructionNode::Load(_)
                | RVInstructionNode::LoadAddr(_)
                | RVInstructionNode::Csr(_)
                | RVInstructionNode::CsrI(_)
        )
    }

    fn uses_memory_location(&self) -> Option<(RVRegister, Imm)> {
        match self {
            RVInstructionNode::Store(s) => Some((s.rs1.get_cloned(), s.imm.get_cloned())),
            RVInstructionNode::Load(l) => Some((l.rs1.get_cloned(), l.imm.get_cloned())),
            _ => None,
        }
    }

    fn is_unconditional_jump(&self) -> bool {
        match self {
            RVInstructionNode::JumpLink(x) if x.rd == RVRegister::X0 => true,
            RVInstructionNode::JumpLinkR(x) if x.rd == RVRegister::X0 => true,
            RVInstructionNode::Branch(x) => {
                x.rs1 == RVRegister::X0
                    && x.rs2 == RVRegister::X0
                    && (x.inst == BranchType::Beq
                        || x.inst == BranchType::Bge
                        || x.inst == BranchType::Bgeu)
            }
            _ => false,
        }
    }

    fn writes_to(&self) -> HashSet<RegisterToken> {
        match self {
            RVInstructionNode::Load(load) => [load.rd.clone()].into(),
            RVInstructionNode::LoadAddr(load) => [load.rd.clone()].into(),
            RVInstructionNode::Arith(arith) => [arith.rd.clone()].into(),
            RVInstructionNode::IArith(iarith) => [iarith.rd.clone()].into(),
            RVInstructionNode::JumpLink(jump_link) => match jump_link.inst.get() {
                JumpLinkType::Jal => [jump_link.rd.clone()].into(),
                JumpLinkType::Tail => [
                    jump_link.rd.clone(),
                    With::new(RVRegister::X6, jump_link.rd.token().clone()),
                ]
                .into(),
            },
            RVInstructionNode::JumpLinkR(jump_link_r) => [jump_link_r.rd.clone()].into(),
            RVInstructionNode::Csr(csr) => [csr.rd.clone()].into(),
            RVInstructionNode::CsrI(csri) => [csri.rd.clone()].into(),
            RVInstructionNode::Basic(_)
            | RVInstructionNode::Branch(_)
            | RVInstructionNode::Store(_) => HashSet::new(),
        }
    }

    fn reads_from(&self) -> HashSet<RegisterToken> {
        let vector = match self {
            RVInstructionNode::Arith(x) => vec![x.rs1.clone(), x.rs2.clone()],
            RVInstructionNode::IArith(x) => vec![x.rs1.clone()],
            RVInstructionNode::JumpLinkR(x) => vec![x.rs1.clone()],
            RVInstructionNode::Branch(x) => vec![x.rs1.clone(), x.rs2.clone()],
            RVInstructionNode::Store(x) => vec![x.rs1.clone(), x.rs2.clone()],
            RVInstructionNode::Load(x) => vec![x.rs1.clone()],
            RVInstructionNode::Csr(x) => vec![x.rs1.clone()],
            RVInstructionNode::JumpLink(_)
            | RVInstructionNode::Basic(_)
            | RVInstructionNode::LoadAddr(_)
            | RVInstructionNode::CsrI(_) => vec![],
        };
        vector.into_iter().collect()
    }
}
