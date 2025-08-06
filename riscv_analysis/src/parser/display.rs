use std::fmt::Display;

use crate::parser::Inst;

use super::ParserNode;

impl Display for ParserNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserNode::Arith(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                write!(f, "{inst} {} <- {}, {}", x.rd, x.rs1, x.rs2)
            }
            ParserNode::IArith(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                write!(f, "{inst} {} <- {}, {}", x.rd, x.rs1, x.imm.get())
            }
            ParserNode::JumpLink(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                write!(f, "{inst} [{}] | {} <- PC", x.name, x.rd)
            }
            ParserNode::JumpLinkR(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                write!(f, "{inst} [{}]", x.rs1)
            }
            ParserNode::Basic(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                write!(f, "{inst}")
            }
            ParserNode::Branch(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                write!(f, "{inst} {}--{}, [{}]", x.rs1, x.rs2, x.name)
            }
            ParserNode::Store(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                write!(f, "{inst} {} -> {}({})", x.rs2, x.imm.get(), x.rs1)
            }
            ParserNode::Load(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                write!(f, "{inst} {} <- {}({})", x.rd, x.imm.get(), x.rs1)
            }
            ParserNode::LoadAddr(x) => {
                let inst = "la";
                let rd = x.rd.to_string();
                let name = x.name.to_string();
                write!(f, "{inst} {rd} <- [{name}]")
            }
            ParserNode::Csr(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                write!(f, "{inst} {} <- {} <- {}", x.rd, x.csr.get().value(), x.rs1)
            }
            ParserNode::CsrI(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                write!(
                    f,
                    "{inst} {} <- {} <- {}",
                    x.rd,
                    x.csr.get().value(),
                    x.imm.get()
                )
            }
        }
    }
}
