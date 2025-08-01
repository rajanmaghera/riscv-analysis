use crate::parser::imm::{CsrImm, Imm};
use crate::parser::inst::Inst;
use crate::parser::inst::{
    ArithType, BasicType, BranchType, CsrIType, CsrType, IArithType, JumpLinkRType, JumpLinkType,
    LoadType, PseudoType, StoreType,
};
use std::collections::HashSet;

use std::hash::{Hash, Hasher};

use super::{
    Arith, Basic, Branch, Csr, CsrI, HasIdentity, IArith, JumpLink, JumpLinkR, LabelStringToken,
    Load, LoadAddr, RawToken, RegisterToken, Store, With,
};
use crate::cfg::Segment;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParserNode {
    Arith(Arith),
    IArith(IArith),
    JumpLink(JumpLink),
    JumpLinkR(JumpLinkR),
    Basic(Basic),
    Branch(Branch),
    Store(Store),       // Stores
    Load(Load),         // Loads, are actually mostly ITypes
    LoadAddr(LoadAddr), // Load address
    Csr(Csr),
    CsrI(CsrI),
}

impl ParserNode {
    /// Get the labels that refer to this node.
    pub fn label_names(&self) -> impl Iterator<Item = &LabelStringToken> {
        match self {
            ParserNode::Arith(x) => x.labels.iter(),
            ParserNode::IArith(x) => x.labels.iter(),
            ParserNode::JumpLink(x) => x.labels.iter(),
            ParserNode::JumpLinkR(x) => x.labels.iter(),
            ParserNode::Basic(x) => x.labels.iter(),
            ParserNode::Branch(x) => x.labels.iter(),
            ParserNode::Store(x) => x.labels.iter(),
            ParserNode::Load(x) => x.labels.iter(),
            ParserNode::Csr(x) => x.labels.iter(),
            ParserNode::CsrI(x) => x.labels.iter(),
            ParserNode::LoadAddr(x) => x.labels.iter(),
        }
    }

    #[must_use]
    pub fn segment(&self) -> Segment {
        match self {
            ParserNode::Arith(x) => x.segment,
            ParserNode::IArith(x) => x.segment,
            ParserNode::JumpLink(x) => x.segment,
            ParserNode::JumpLinkR(x) => x.segment,
            ParserNode::Basic(x) => x.segment,
            ParserNode::Branch(x) => x.segment,
            ParserNode::Store(x) => x.segment,
            ParserNode::Load(x) => x.segment,
            ParserNode::Csr(x) => x.segment,
            ParserNode::CsrI(x) => x.segment,
            ParserNode::LoadAddr(x) => x.segment,
        }
    }

    #[must_use]
    pub fn token(&self) -> &RawToken {
        match self {
            ParserNode::Arith(x) => &x.token,
            ParserNode::IArith(x) => &x.token,
            ParserNode::JumpLink(x) => &x.token,
            ParserNode::JumpLinkR(x) => &x.token,
            ParserNode::Basic(x) => &x.token,
            ParserNode::Branch(x) => &x.token,
            ParserNode::Store(x) => &x.token,
            ParserNode::Load(x) => &x.token,
            ParserNode::Csr(x) => &x.token,
            ParserNode::CsrI(x) => &x.token,
            ParserNode::LoadAddr(x) => &x.token,
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
        }
    }

    #[must_use]
    pub fn new_arith(
        inst: With<ArithType>,
        rd: RegisterToken,
        rs1: RegisterToken,
        rs2: RegisterToken,
        token: RawToken,
        segment: Segment,
        labels: HashSet<LabelStringToken>,
    ) -> ParserNode {
        ParserNode::Arith(Arith {
            inst,
            rd,
            rs1,
            rs2,
            key: Uuid::new_v4(),
            token,
            segment,
            labels,
        })
    }

    #[must_use]
    pub fn new_iarith(
        inst: With<IArithType>,
        rd: RegisterToken,
        rs1: RegisterToken,
        imm: With<Imm>,
        token: RawToken,
        segment: Segment,
        labels: HashSet<LabelStringToken>,
    ) -> ParserNode {
        ParserNode::IArith(IArith {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
            token,
            segment,
            labels,
        })
    }

    #[must_use]
    pub fn new_jump_link(
        inst: With<JumpLinkType>,
        rd: RegisterToken,
        name: LabelStringToken,
        token: RawToken,
        segment: Segment,
        labels: HashSet<LabelStringToken>,
    ) -> ParserNode {
        ParserNode::JumpLink(JumpLink {
            inst,
            rd,
            name,
            key: Uuid::new_v4(),
            token,
            segment,
            labels,
        })
    }

    #[must_use]
    pub fn new_jump_link_r(
        inst: With<JumpLinkRType>,
        rd: RegisterToken,
        rs1: RegisterToken,
        imm: With<Imm>,
        token: RawToken,
        segment: Segment,
        labels: HashSet<LabelStringToken>,
    ) -> ParserNode {
        ParserNode::JumpLinkR(JumpLinkR {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
            token,
            segment,
            labels,
        })
    }

    #[must_use]
    pub fn new_basic(
        inst: With<BasicType>,
        token: RawToken,
        segment: Segment,
        labels: HashSet<LabelStringToken>,
    ) -> ParserNode {
        ParserNode::Basic(Basic {
            inst,
            key: Uuid::new_v4(),
            token,
            segment,
            labels,
        })
    }

    #[must_use]
    pub fn new_branch(
        inst: With<BranchType>,
        rs1: RegisterToken,
        rs2: RegisterToken,
        name: LabelStringToken,
        token: RawToken,
        segment: Segment,
        labels: HashSet<LabelStringToken>,
    ) -> ParserNode {
        ParserNode::Branch(Branch {
            inst,
            rs1,
            rs2,
            name,
            key: Uuid::new_v4(),
            token,
            segment,
            labels,
        })
    }

    #[must_use]
    pub fn new_store(
        inst: With<StoreType>,
        rs1: RegisterToken,
        rs2: RegisterToken,
        imm: With<Imm>,
        token: RawToken,
        segment: Segment,
        labels: HashSet<LabelStringToken>,
    ) -> ParserNode {
        ParserNode::Store(Store {
            inst,
            rs1,
            rs2,
            imm,
            key: Uuid::new_v4(),
            token,
            segment,
            labels,
        })
    }

    #[must_use]
    pub fn new_load(
        inst: With<LoadType>,
        rd: RegisterToken,
        rs1: RegisterToken,
        imm: With<Imm>,
        token: RawToken,
        segment: Segment,
        labels: HashSet<LabelStringToken>,
    ) -> ParserNode {
        ParserNode::Load(Load {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
            token,
            segment,
            labels,
        })
    }

    #[must_use]
    pub fn new_csr(
        inst: With<CsrType>,
        rd: RegisterToken,
        csr: With<CsrImm>,
        rs1: RegisterToken,
        token: RawToken,
        segment: Segment,
        labels: HashSet<LabelStringToken>,
    ) -> ParserNode {
        ParserNode::Csr(Csr {
            inst,
            rd,
            rs1,
            csr,
            key: Uuid::new_v4(),
            token,
            segment,
            labels,
        })
    }

    #[must_use]
    pub fn new_csri(
        inst: With<CsrIType>,
        rd: RegisterToken,
        csr: With<CsrImm>,
        imm: With<Imm>,
        token: RawToken,
        segment: Segment,
        labels: HashSet<LabelStringToken>,
    ) -> ParserNode {
        ParserNode::CsrI(CsrI {
            inst,
            rd,
            imm,
            csr,
            key: Uuid::new_v4(),
            token,
            segment,
            labels,
        })
    }

    #[must_use]
    pub fn new_load_addr(
        inst: With<PseudoType>,
        rd: RegisterToken,
        name: LabelStringToken,
        token: RawToken,
        segment: Segment,
        labels: HashSet<LabelStringToken>,
    ) -> ParserNode {
        ParserNode::LoadAddr(LoadAddr {
            inst,
            rd,
            name,
            key: Uuid::new_v4(),
            token,
            segment,
            labels,
        })
    }
}
