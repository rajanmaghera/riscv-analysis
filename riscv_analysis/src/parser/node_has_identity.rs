use uuid::Uuid;

use super::{HasIdentity, RVInstructionNode};

impl HasIdentity for RVInstructionNode {
    fn id(&self) -> Uuid {
        match self {
            RVInstructionNode::Arith(a) => a.key,
            RVInstructionNode::IArith(a) => a.key,
            RVInstructionNode::JumpLink(a) => a.key,
            RVInstructionNode::JumpLinkR(a) => a.key,
            RVInstructionNode::Basic(a) => a.key,
            RVInstructionNode::Branch(a) => a.key,
            RVInstructionNode::Store(a) => a.key,
            RVInstructionNode::Load(a) => a.key,
            RVInstructionNode::Csr(a) => a.key,
            RVInstructionNode::CsrI(a) => a.key,
            RVInstructionNode::LoadAddr(a) => a.key,
        }
    }
}
