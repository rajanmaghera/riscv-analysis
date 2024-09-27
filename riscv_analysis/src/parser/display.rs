use std::fmt::Display;

use uuid::Uuid;

use crate::{parser::Inst, passes::DiagnosticLocation};

use super::ParserNode;

impl Display for ParserNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match &self {
            ParserNode::ProgramEntry(_) => "--- [PROGRAM ENTRY] ---".to_string(),
            ParserNode::ProgramExit(_) => "--- [PROGRAM EXIT] ---".to_string(),
            ParserNode::FuncEntry(_) => "--- FUNCTION ENTRY ---".to_string(),
            ParserNode::FuncExit(_) => "--- FUNCTION EXIT ---".to_string(),
            ParserNode::Arith(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rd = x.rd.data.to_string();
                let rs1 = x.rs1.data.to_string();
                let rs2 = x.rs2.data.to_string();
                format!("{inst} {rd} <- {rs1}, {rs2}")
            }
            ParserNode::IArith(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rd = x.rd.data.to_string();
                let rs1 = x.rs1.data.to_string();
                let imm = x.imm.data.0.to_string();
                format!("{inst} {rd} <- {rs1}, {imm}")
            }
            ParserNode::Label(x) => format!("---[{}]---", x.name.data.0),
            ParserNode::JumpLink(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let name = x.name.data.0.clone();
                let rd = x.rd.data.to_string();
                format!("{inst} [{name}] | {rd} <- PC")
            }
            ParserNode::JumpLinkR(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rs1 = x.rs1.data.to_string();
                format!("{inst} [{rs1}]")
            }
            ParserNode::Basic(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                format!("{inst}")
            }
            ParserNode::Directive(x) => {
                let dir = x.dir.to_string();
                format!("-<{dir}>-")
            }
            ParserNode::Branch(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rs1 = x.rs1.data.to_string();
                let rs2 = x.rs2.data.to_string();
                let name = x.name.data.0.clone();
                format!("{inst} {rs1}--{rs2}, [{name}]")
            }
            ParserNode::Store(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rs1 = x.rs1.data.to_string();
                let rs2 = x.rs2.data.to_string();
                let imm = x.imm.data.0.to_string();
                format!("{inst} {rs2} -> {imm}({rs1})")
            }
            ParserNode::Load(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rd = x.rd.data.to_string();
                let rs1 = x.rs1.data.to_string();
                let imm = x.imm.data.0.to_string();
                format!("{inst} {rd} <- {imm}({rs1})")
            }
            ParserNode::LoadAddr(x) => {
                let inst = "la";
                let rd = x.rd.data.to_string();
                let name = x.name.data.0.clone();
                format!("{inst} {rd} <- [{name}]")
            }
            ParserNode::Csr(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rd = x.rd.data.to_string();
                let csr = x.csr.data.0.to_string();
                let rs1 = x.rs1.data.to_string();
                format!("{inst} {rd} <- {csr} <- {rs1}")
            }
            ParserNode::CsrI(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rd = x.rd.data.to_string();
                let csr = x.csr.data.0.to_string();
                let imm = x.imm.data.0.to_string();
                format!("{inst} {rd} <- {csr} <- {imm}")
            }
        };
        write!(f, "{res}")
    }
}

impl DiagnosticLocation for ParserNode {
    fn file(&self) -> Uuid {
        self.token().file
    }
    fn range(&self) -> super::Range {
        self.token().pos
    }
}
