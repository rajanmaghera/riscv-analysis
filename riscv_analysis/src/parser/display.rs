use std::fmt::Display;

use crate::parser::Inst;

use super::ParserNode;

impl Display for ParserNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match &self {
            ParserNode::ProgramEntry(_) => "--- [PROGRAM ENTRY] ---".to_string(),
            ParserNode::FuncEntry(_) => "--- FUNCTION ENTRY ---".to_string(),
            ParserNode::Arith(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                let rd = x.rd.get().to_string();
                let rs1 = x.rs1.get().to_string();
                let rs2 = x.rs2.get().to_string();
                format!("{inst} {rd} <- {rs1}, {rs2}")
            }
            ParserNode::IArith(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                let rd = x.rd.get().to_string();
                let rs1 = x.rs1.get().to_string();
                let imm = x.imm.get().value().to_string();
                format!("{inst} {rd} <- {rs1}, {imm}")
            }
            ParserNode::Label(x) => format!("---[{}]---", x.name.get().to_string()),
            ParserNode::JumpLink(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                let name = x.name.get().to_string();
                let rd = x.rd.get().to_string();
                format!("{inst} [{name}] | {rd} <- PC")
            }
            ParserNode::JumpLinkR(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                let rs1 = x.rs1.get().to_string();
                format!("{inst} [{rs1}]")
            }
            ParserNode::Basic(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                format!("{inst}")
            }
            ParserNode::Directive(x) => {
                let dir = x.dir.to_string();
                format!("-<{dir}>-")
            }
            ParserNode::Branch(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                let rs1 = x.rs1.get().to_string();
                let rs2 = x.rs2.get().to_string();
                let name = x.name.get().to_string();
                format!("{inst} {rs1}--{rs2}, [{name}]")
            }
            ParserNode::Store(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                let rs1 = x.rs1.get().to_string();
                let rs2 = x.rs2.get().to_string();
                let imm = x.imm.get().value().to_string();
                format!("{inst} {rs2} -> {imm}({rs1})")
            }
            ParserNode::Load(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                let rd = x.rd.get().to_string();
                let rs1 = x.rs1.get().to_string();
                let imm = x.imm.get().value().to_string();
                format!("{inst} {rd} <- {imm}({rs1})")
            }
            ParserNode::LoadAddr(x) => {
                let inst = "la";
                let rd = x.rd.get().to_string();
                let name = x.name.get().to_string();
                format!("{inst} {rd} <- [{name}]")
            }
            ParserNode::Csr(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                let rd = x.rd.get().to_string();
                let csr = x.csr.get().value().to_string();
                let rs1 = x.rs1.get().to_string();
                format!("{inst} {rd} <- {csr} <- {rs1}")
            }
            ParserNode::CsrI(x) => {
                let inst: Inst = Inst::from(x.inst.get());
                let rd = x.rd.get().to_string();
                let csr = x.csr.get().value().to_string();
                let imm = x.imm.get().value().to_string();
                format!("{inst} {rd} <- {csr} <- {imm}")
            }
        };
        write!(f, "{res}")
    }
}
