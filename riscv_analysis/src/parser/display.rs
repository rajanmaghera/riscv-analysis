use std::fmt::Display;

use crate::parser::RVInst;

use super::RVInstructionNode;

impl Display for RVInstructionNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RVInstructionNode::Arith(x) => {
                let inst: RVInst = RVInst::from(x.inst.get());
                write!(f, "{inst} {} <- {}, {}", x.rd, x.rs1, x.rs2)
            }
            RVInstructionNode::IArith(x) => {
                let inst: RVInst = RVInst::from(x.inst.get());
                write!(f, "{inst} {} <- {}, {}", x.rd, x.rs1, x.imm.get())
            }
            RVInstructionNode::JumpLink(x) => {
                let inst: RVInst = RVInst::from(x.inst.get());
                write!(f, "{inst} [{}] | {} <- PC", x.name, x.rd)
            }
            RVInstructionNode::JumpLinkR(x) => {
                let inst: RVInst = RVInst::from(x.inst.get());
                write!(f, "{inst} [{}]", x.rs1)
            }
            RVInstructionNode::Basic(x) => {
                let inst: RVInst = RVInst::from(x.inst.get());
                write!(f, "{inst}")
            }
            RVInstructionNode::Branch(x) => {
                let inst: RVInst = RVInst::from(x.inst.get());
                write!(f, "{inst} {}--{}, [{}]", x.rs1, x.rs2, x.name)
            }
            RVInstructionNode::Store(x) => {
                let inst: RVInst = RVInst::from(x.inst.get());
                write!(f, "{inst} {} -> {}({})", x.rs2, x.imm.get(), x.rs1)
            }
            RVInstructionNode::Load(x) => {
                let inst: RVInst = RVInst::from(x.inst.get());
                write!(f, "{inst} {} <- {}({})", x.rd, x.imm.get(), x.rs1)
            }
            RVInstructionNode::LoadAddr(x) => {
                let inst = "la";
                let rd = x.rd.to_string();
                let name = x.name.to_string();
                write!(f, "{inst} {rd} <- [{name}]")
            }
            RVInstructionNode::Csr(x) => {
                let inst: RVInst = RVInst::from(x.inst.get());
                write!(f, "{inst} {} <- {} <- {}", x.rd, x.csr.get().value(), x.rs1)
            }
            RVInstructionNode::CsrI(x) => {
                let inst: RVInst = RVInst::from(x.inst.get());
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
