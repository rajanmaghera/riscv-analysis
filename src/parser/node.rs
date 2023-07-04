use crate::parser::imm::{CSRImm, Imm};
use crate::parser::inst::Inst;
use crate::parser::inst::{
    ArithType, BasicType, BranchType, CSRIType, CSRType, IArithType, JumpLinkRType, JumpLinkType,
    LoadType, PseudoType, StoreType, UpperArithType,
};

use crate::parser::register::Register;
use crate::parser::token::{Range, With};

use std::collections::HashSet;

use std::hash::{Hash, Hasher};

use uuid::Uuid;

use super::token::Position;
use super::{
    Arith, Basic, Branch, Csr, CsrI, Directive, DirectiveToken, DirectiveType, FuncEntry, IArith,
    JumpLink, JumpLinkR, Label, LabelString, Load, LoadAddr, ProgramEntry, Store, Token,
    UpperArith,
};

#[derive(Debug, Clone)]
pub enum ParserNode {
    ProgramEntry(ProgramEntry),
    FuncEntry(FuncEntry),
    Arith(Arith),
    IArith(IArith),
    UpperArith(UpperArith),
    Label(Label),
    JumpLink(JumpLink),
    JumpLinkR(JumpLinkR),
    Basic(Basic),
    Directive(Directive),
    Branch(Branch),
    Store(Store),       // Stores
    Load(Load),         // Loads, are actually mostly ITypes
    LoadAddr(LoadAddr), // Load address
    Csr(Csr),
    CsrI(CsrI),
}

impl ParserNode {
    pub fn id(&self) -> Uuid {
        match self {
            ParserNode::Arith(a) => a.key,
            ParserNode::IArith(a) => a.key,
            ParserNode::UpperArith(a) => a.key,
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

impl PartialEq for ParserNode {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
impl Eq for ParserNode {}
impl Hash for ParserNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl ParserNode {
    pub fn inst(&self) -> With<Inst> {
        let token = match self {
            ParserNode::Arith(x) => x.inst.token.clone(),
            ParserNode::IArith(x) => x.inst.token.clone(),
            ParserNode::UpperArith(x) => x.inst.token.clone(),
            ParserNode::Label(x) => x.name.token.clone(),
            ParserNode::JumpLink(x) => x.inst.token.clone(),
            ParserNode::JumpLinkR(x) => x.inst.token.clone(),
            ParserNode::Basic(x) => x.inst.token.clone(),
            ParserNode::Directive(x) => x.token.token.clone(),
            ParserNode::Branch(x) => x.inst.token.clone(),
            ParserNode::Store(x) => x.inst.token.clone(),
            ParserNode::Load(x) => x.inst.token.clone(),
            ParserNode::Csr(x) => x.inst.token.clone(),
            ParserNode::CsrI(x) => x.inst.token.clone(),
            ParserNode::LoadAddr(x) => x.inst.token.clone(),
            ParserNode::ProgramEntry(_) | ParserNode::FuncEntry(_) => Token::Symbol(String::new()),
        };
        let inst: Inst = match self {
            ParserNode::Arith(x) => (&x.inst.data).into(),
            ParserNode::IArith(x) => (&x.inst.data).into(),
            ParserNode::UpperArith(x) => (&x.inst.data).into(),
            ParserNode::JumpLink(x) => (&x.inst.data).into(),
            ParserNode::JumpLinkR(x) => (&x.inst.data).into(),
            ParserNode::Basic(x) => (&x.inst.data).into(),
            ParserNode::Branch(x) => (&x.inst.data).into(),
            ParserNode::Store(x) => (&x.inst.data).into(),
            ParserNode::Load(x) => (&x.inst.data).into(),
            ParserNode::Csr(x) => (&x.inst.data).into(),
            ParserNode::CsrI(x) => (&x.inst.data).into(),
            ParserNode::LoadAddr(_) => Inst::La,
            ParserNode::Label(_)
            | ParserNode::Directive(_)
            | ParserNode::FuncEntry(_)
            | ParserNode::ProgramEntry(_) => Inst::Nop,
        };
        let pos = match self {
            ParserNode::Arith(x) => x.inst.pos.clone(),
            ParserNode::IArith(x) => x.inst.pos.clone(),
            ParserNode::UpperArith(x) => x.inst.pos.clone(),
            ParserNode::Label(x) => x.name.pos.clone(),
            ParserNode::JumpLink(x) => x.inst.pos.clone(),
            ParserNode::JumpLinkR(x) => x.inst.pos.clone(),
            ParserNode::Basic(x) => x.inst.pos.clone(),
            ParserNode::Directive(x) => x.token.pos.clone(),
            ParserNode::Branch(x) => x.inst.pos.clone(),
            ParserNode::Store(x) => x.inst.pos.clone(),
            ParserNode::Load(x) => x.inst.pos.clone(),
            ParserNode::Csr(x) => x.inst.pos.clone(),
            ParserNode::CsrI(x) => x.inst.pos.clone(),
            ParserNode::LoadAddr(x) => x.inst.pos.clone(),
            ParserNode::ProgramEntry(_) | ParserNode::FuncEntry(_) => Range {
                start: Position { line: 0, column: 0 },
                end: Position { line: 0, column: 0 },
            },
        };
        With {
            token,
            data: inst,
            pos,
            file: self.file(),
        }
    }
    pub fn file(&self) -> Uuid {
        let file = match self {
            ParserNode::Arith(x) => x.inst.file.clone(),
            ParserNode::IArith(x) => x.inst.file.clone(),
            ParserNode::UpperArith(x) => x.inst.file.clone(),
            ParserNode::Label(x) => x.name.file.clone(),
            ParserNode::JumpLink(x) => x.inst.file.clone(),
            ParserNode::JumpLinkR(x) => x.inst.file.clone(),
            ParserNode::Basic(x) => x.inst.file.clone(),
            ParserNode::Directive(x) => x.token.file.clone(),
            ParserNode::Branch(x) => x.inst.file.clone(),
            ParserNode::Store(x) => x.inst.file.clone(),
            ParserNode::Load(x) => x.inst.file.clone(),
            ParserNode::Csr(x) => x.inst.file.clone(),
            ParserNode::CsrI(x) => x.inst.file.clone(),
            ParserNode::LoadAddr(x) => x.inst.file.clone(),
            ParserNode::ProgramEntry(x) => x.file.clone(),
            ParserNode::FuncEntry(x) => x.file.clone(),
        };
        file
    }

    pub fn new_arith(
        inst: With<ArithType>,
        rd: With<Register>,
        rs1: With<Register>,
        rs2: With<Register>,
    ) -> ParserNode {
        ParserNode::Arith(Arith {
            inst,
            rd,
            rs1,
            rs2,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_iarith(
        inst: With<IArithType>,
        rd: With<Register>,
        rs1: With<Register>,
        imm: With<Imm>,
    ) -> ParserNode {
        ParserNode::IArith(IArith {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_upper_arith(
        inst: With<UpperArithType>,
        rd: With<Register>,
        imm: With<Imm>,
    ) -> ParserNode {
        ParserNode::UpperArith(UpperArith {
            inst,
            rd,
            imm,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_jump_link(
        inst: With<JumpLinkType>,
        rd: With<Register>,
        name: With<LabelString>,
    ) -> ParserNode {
        ParserNode::JumpLink(JumpLink {
            inst,
            rd,
            name,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_jump_link_r(
        inst: With<JumpLinkRType>,
        rd: With<Register>,
        rs1: With<Register>,
        imm: With<Imm>,
    ) -> ParserNode {
        ParserNode::JumpLinkR(JumpLinkR {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_basic(inst: With<BasicType>) -> ParserNode {
        ParserNode::Basic(Basic {
            inst,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_directive(token: With<DirectiveToken>, dir: DirectiveType) -> ParserNode {
        ParserNode::Directive(Directive {
            token,
            dir,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_branch(
        inst: With<BranchType>,
        rs1: With<Register>,
        rs2: With<Register>,
        name: With<LabelString>,
    ) -> ParserNode {
        ParserNode::Branch(Branch {
            inst,
            rs1,
            rs2,
            name,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_store(
        inst: With<StoreType>,
        rs1: With<Register>,
        rs2: With<Register>,
        imm: With<Imm>,
    ) -> ParserNode {
        ParserNode::Store(Store {
            inst,
            rs1,
            rs2,
            imm,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_load(
        inst: With<LoadType>,
        rd: With<Register>,
        rs1: With<Register>,
        imm: With<Imm>,
    ) -> ParserNode {
        ParserNode::Load(Load {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_csr(
        inst: With<CSRType>,
        rd: With<Register>,
        csr: With<CSRImm>,
        rs1: With<Register>,
    ) -> ParserNode {
        ParserNode::Csr(Csr {
            inst,
            rd,
            rs1,
            csr,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_func_entry(file: Uuid) -> ParserNode {
        ParserNode::FuncEntry(FuncEntry {
            key: Uuid::new_v4(),
            file,
        })
    }

    pub fn new_program_entry(file: Uuid) -> ParserNode {
        ParserNode::ProgramEntry(ProgramEntry {
            key: Uuid::new_v4(),
            file,
        })
    }

    pub fn new_csri(
        inst: With<CSRIType>,
        rd: With<Register>,
        csr: With<CSRImm>,
        imm: With<Imm>,
    ) -> ParserNode {
        ParserNode::CsrI(CsrI {
            inst,
            rd,
            imm,
            csr,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_label(name: With<LabelString>) -> ParserNode {
        ParserNode::Label(Label {
            name,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_load_addr(
        inst: With<PseudoType>,
        rd: With<Register>,
        name: With<LabelString>,
    ) -> ParserNode {
        ParserNode::LoadAddr(LoadAddr {
            inst,
            rd,
            name,
            key: Uuid::new_v4(),
        })
    }

    pub fn is_return(&self) -> bool {
        match self {
            ParserNode::JumpLinkR(x) => {
                x.inst == JumpLinkRType::Jalr
                    && x.rd == Register::X0
                    && x.rs1 == Register::X1
                    && x.imm == Imm(0)
            }
            _ => false,
        }
    }

    pub fn is_memory_access(&self) -> bool {
        matches!(self, ParserNode::Load(_) | ParserNode::Store(_))
    }

    pub fn calls_to(&self) -> Option<With<LabelString>> {
        match self {
            ParserNode::JumpLink(x) if x.rd == Register::X1 => Some(x.name.clone()),
            _ => None,
        }
    }

    pub fn is_ecall(&self) -> bool {
        match self {
            ParserNode::Basic(x) => x.inst == BasicType::Ecall,
            _ => false,
        }
    }

    pub fn jumps_to(&self) -> Option<With<LabelString>> {
        match self {
            ParserNode::JumpLink(x) if x.rd != Register::X1 => Some(x.name.clone()),
            ParserNode::Branch(x) => Some(x.name.clone()),
            _ => None,
        }
    }

    pub fn is_any_entry(&self) -> bool {
        matches!(self, ParserNode::ProgramEntry(_) | ParserNode::FuncEntry(_))
    }

    pub fn is_function_entry(&self) -> bool {
        matches!(self, ParserNode::FuncEntry(_))
    }

    pub fn is_program_entry(&self) -> bool {
        matches!(self, ParserNode::ProgramEntry(_))
    }

    // NOTE: This is in context to a register store, not a memory store
    pub fn stores_to(&self) -> Option<With<Register>> {
        match self {
            ParserNode::Load(load) => Some(load.rd.clone()),
            ParserNode::LoadAddr(load) => Some(load.rd.clone()),
            ParserNode::Arith(arith) => Some(arith.rd.clone()),
            ParserNode::IArith(iarith) => Some(iarith.rd.clone()),
            ParserNode::UpperArith(upper_arith) => Some(upper_arith.rd.clone()),
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

    pub fn reads_from(&self) -> HashSet<With<Register>> {
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
            | ParserNode::UpperArith(_)
            | ParserNode::Label(_)
            | ParserNode::JumpLink(_)
            | ParserNode::Basic(_)
            | ParserNode::Directive(_)
            | ParserNode::LoadAddr(_)
            | ParserNode::CsrI(_) => vec![],
        };
        vector.into_iter().collect()
    }

    pub fn set_uuid(&mut self, uuid: Uuid) {
        match self {
            ParserNode::Arith(x) => x.key = uuid,
            ParserNode::IArith(x) => x.key = uuid,
            ParserNode::UpperArith(x) => x.key = uuid,
            ParserNode::Label(x) => x.key = uuid,
            ParserNode::JumpLink(x) => x.key = uuid,
            ParserNode::JumpLinkR(x) => x.key = uuid,
            ParserNode::Basic(x) => x.key = uuid,
            ParserNode::Directive(x) => x.key = uuid,
            ParserNode::Branch(x) => x.key = uuid,
            ParserNode::Store(x) => x.key = uuid,
            ParserNode::Load(x) => x.key = uuid,
            ParserNode::Csr(x) => x.key = uuid,
            ParserNode::CsrI(x) => x.key = uuid,
            ParserNode::LoadAddr(x) => x.key = uuid,
            ParserNode::ProgramEntry(_) | ParserNode::FuncEntry(_) => (),
        }
    }

}
