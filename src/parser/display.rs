use std::fmt::Display;

use crate::parser::Inst;

use super::{LineDisplay, ParserNode, Position, Range};

impl Display for ParserNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match &self {
            ParserNode::ProgramEntry(_) => "--- [PROGRAM ENTRY] ---".to_string(),
            ParserNode::FuncEntry(_) => {
                "--- FUNCTION ENTRY ---".to_string()
            }
            ParserNode::UpperArith(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rd = x.rd.data.to_string();
                let imm = x.imm.data.0.to_string();
                format!("{inst} {rd} <- {imm}")
            }
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
                let dir = x.dir.data.to_string();
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

impl ParserNode {
    pub fn get_store_range(&self) -> Range {
        if let Some(item) = self.stores_to() {
            item.pos
        } else {
            self.get_range()
        }
    }
}

impl LineDisplay for ParserNode {
    fn get_range(&self) -> Range {
        match &self {
            ParserNode::ProgramEntry(_) => Range {
                start: Position { line: 0, column: 0 },
                end: Position { line: 0, column: 0 },
            },
            ParserNode::UpperArith(x) => {
                let mut range = x.inst.pos.clone();
                range.end = x.imm.pos.end;
                range
            }
            ParserNode::Label(x) => x.name.pos.clone(),
            ParserNode::Arith(arith) => {
                let mut range = arith.inst.pos.clone();
                range.end = arith.rs2.pos.end;
                range
            }
            ParserNode::IArith(iarith) => {
                let mut range = iarith.inst.pos.clone();
                range.end = iarith.imm.pos.end;
                range
            }
            ParserNode::JumpLink(jl) => {
                let mut range = jl.inst.pos.clone();
                range.end = jl.name.pos.end;
                range
            }
            ParserNode::JumpLinkR(jlr) => {
                let mut range = jlr.inst.pos.clone();
                range.end = jlr.inst.pos.end;
                range
            }
            ParserNode::Branch(branch) => {
                let mut range = branch.inst.pos.clone();
                range.end = branch.name.pos.end;
                range
            }
            ParserNode::Store(store) => {
                let mut range = store.inst.pos.clone();
                range.end = store.imm.pos.end;
                range
            }
            ParserNode::Load(load) => {
                let mut range = load.inst.pos.clone();
                range.end = load.imm.pos.end;
                range
            }
            ParserNode::Csr(csr) => {
                let mut range = csr.inst.pos.clone();
                range.end = csr.rs1.pos.end;
                range
            }
            ParserNode::CsrI(csr) => {
                let mut range = csr.inst.pos.clone();
                range.end = csr.imm.pos.end;
                range
            }
            ParserNode::Basic(x) => x.inst.pos.clone(),
            ParserNode::LoadAddr(x) => {
                let mut range = x.inst.pos.clone();
                range.end = x.name.pos.end;
                range
            }
            ParserNode::Directive(directive) => directive.dir.pos.clone(),
            ParserNode::FuncEntry(_) => Range {
                start: Position { line: 0, column: 0 },
                end: Position { line: 0, column: 0 },
            },
        }
    }
}
