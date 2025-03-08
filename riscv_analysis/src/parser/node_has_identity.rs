use uuid::Uuid;

use super::{HasIdentity, ParserNode};

impl HasIdentity for ParserNode {
    fn id(&self) -> Uuid {
        match self {
            ParserNode::Arith(a) => a.key,
            ParserNode::IArith(a) => a.key,
            ParserNode::Label(a) => a.key,
            ParserNode::JumpLink(a) => a.key,
            ParserNode::JumpLinkR(a) => a.key,
            ParserNode::Basic(a) => a.key,
            ParserNode::Directive(a) => a.key,
            ParserNode::Branch(a) => a.key,
            ParserNode::Store(a) => a.key,
            ParserNode::Load(a) => a.key,
            ParserNode::Csr(a) => a.key,
            ParserNode::CsrI(a) => a.key,
            ParserNode::LoadAddr(a) => a.key,
            ParserNode::FuncEntry(a) => a.key,
            ParserNode::ProgramEntry(a) => a.key,
        }
    }
}
