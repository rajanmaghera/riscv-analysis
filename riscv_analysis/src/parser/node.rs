use crate::parser::imm::{CSRImm, Imm};
use crate::parser::inst::Inst;
use crate::parser::inst::{
    ArithType, BasicType, BranchType, CSRIType, CSRType, IArithType, JumpLinkRType, JumpLinkType,
    LoadType, PseudoType, StoreType,
};

use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    Arith, Basic, Branch, Csr, CsrI, Directive, DirectiveToken, DirectiveType, FuncEntry,
    HasIdentity, IArith, JumpLink, JumpLinkR, Label, LabelStringToken, Load, LoadAddr,
    ProgramEntry, RawToken, RegisterToken, Store, With,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParserNode {
    ProgramEntry(ProgramEntry),
    FuncEntry(FuncEntry),
    Arith(Arith),
    IArith(IArith),
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
    #[must_use]
    pub fn token(&self) -> &RawToken {
        match self {
            ParserNode::Arith(x) => &x.token,
            ParserNode::IArith(x) => &x.token,
            ParserNode::Label(x) => &x.token,
            ParserNode::JumpLink(x) => &x.token,
            ParserNode::JumpLinkR(x) => &x.token,
            ParserNode::Basic(x) => &x.token,
            ParserNode::Directive(x) => &x.token,
            ParserNode::Branch(x) => &x.token,
            ParserNode::Store(x) => &x.token,
            ParserNode::Load(x) => &x.token,
            ParserNode::Csr(x) => &x.token,
            ParserNode::CsrI(x) => &x.token,
            ParserNode::LoadAddr(x) => &x.token,
            ParserNode::ProgramEntry(x) => &x.token,
            ParserNode::FuncEntry(x) => &x.token,
        }
    }
}

impl PartialEq for ParserNode {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
    }
}
impl Eq for ParserNode {}
impl Hash for ParserNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl ParserNode {
    #[must_use]
    pub fn inst(&self) -> Inst {
        match self {
            ParserNode::Arith(x) => (x.inst.get()).into(),
            ParserNode::IArith(x) => (x.inst.get()).into(),
            ParserNode::JumpLink(x) => (x.inst.get()).into(),
            ParserNode::JumpLinkR(x) => (x.inst.get()).into(),
            ParserNode::Basic(x) => (x.inst.get()).into(),
            ParserNode::Branch(x) => (x.inst.get()).into(),
            ParserNode::Store(x) => (x.inst.get()).into(),
            ParserNode::Load(x) => (x.inst.get()).into(),
            ParserNode::Csr(x) => (x.inst.get()).into(),
            ParserNode::CsrI(x) => (x.inst.get()).into(),
            ParserNode::LoadAddr(_) => Inst::La,
            ParserNode::Label(_)
            | ParserNode::Directive(_)
            | ParserNode::FuncEntry(_)
            | ParserNode::ProgramEntry(_) => Inst::Nop,
        }
    }

    #[must_use]
    pub fn new_arith(
        inst: With<ArithType>,
        rd: RegisterToken,
        rs1: RegisterToken,
        rs2: RegisterToken,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::Arith(Arith {
            inst,
            rd,
            rs1,
            rs2,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_iarith(
        inst: With<IArithType>,
        rd: RegisterToken,
        rs1: RegisterToken,
        imm: With<Imm>,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::IArith(IArith {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_jump_link(
        inst: With<JumpLinkType>,
        rd: RegisterToken,
        name: LabelStringToken,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::JumpLink(JumpLink {
            inst,
            rd,
            name,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_jump_link_r(
        inst: With<JumpLinkRType>,
        rd: RegisterToken,
        rs1: RegisterToken,
        imm: With<Imm>,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::JumpLinkR(JumpLinkR {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_basic(inst: With<BasicType>, token: RawToken) -> ParserNode {
        ParserNode::Basic(Basic {
            inst,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_directive(
        dir_token: With<DirectiveToken>,
        dir: DirectiveType,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::Directive(Directive {
            dir_token,
            dir,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_branch(
        inst: With<BranchType>,
        rs1: RegisterToken,
        rs2: RegisterToken,
        name: LabelStringToken,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::Branch(Branch {
            inst,
            rs1,
            rs2,
            name,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_store(
        inst: With<StoreType>,
        rs1: RegisterToken,
        rs2: RegisterToken,
        imm: With<Imm>,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::Store(Store {
            inst,
            rs1,
            rs2,
            imm,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_load(
        inst: With<LoadType>,
        rd: RegisterToken,
        rs1: RegisterToken,
        imm: With<Imm>,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::Load(Load {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_csr(
        inst: With<CSRType>,
        rd: RegisterToken,
        csr: With<CSRImm>,
        rs1: RegisterToken,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::Csr(Csr {
            inst,
            rd,
            rs1,
            csr,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_func_entry(file: Uuid, token: RawToken, is_interrupt_handler: bool) -> ParserNode {
        ParserNode::FuncEntry(FuncEntry {
            key: Uuid::new_v4(),
            file,
            token,
            is_interrupt_handler,
        })
    }

    #[must_use]
    pub fn new_program_entry(file: Uuid, token: RawToken) -> ParserNode {
        ParserNode::ProgramEntry(ProgramEntry {
            key: Uuid::new_v4(),
            file,
            token,
        })
    }

    #[must_use]
    pub fn new_csri(
        inst: With<CSRIType>,
        rd: RegisterToken,
        csr: With<CSRImm>,
        imm: With<Imm>,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::CsrI(CsrI {
            inst,
            rd,
            imm,
            csr,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_label(name: LabelStringToken, token: RawToken) -> ParserNode {
        ParserNode::Label(Label {
            name,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_load_addr(
        inst: With<PseudoType>,
        rd: RegisterToken,
        name: LabelStringToken,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::LoadAddr(LoadAddr {
            inst,
            rd,
            name,
            key: Uuid::new_v4(),
            token,
        })
    }
}
