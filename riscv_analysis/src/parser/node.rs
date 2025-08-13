use crate::parser::imm::{CsrImm, Imm};
use crate::parser::inst::RVInst;
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
pub enum RVInstructionNode {
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

impl RVInstructionNode {
    /// Get the labels that refer to this node.
    pub fn label_names(&self) -> impl Iterator<Item = &LabelStringToken> {
        match self {
            RVInstructionNode::Arith(x) => x.labels.iter(),
            RVInstructionNode::IArith(x) => x.labels.iter(),
            RVInstructionNode::JumpLink(x) => x.labels.iter(),
            RVInstructionNode::JumpLinkR(x) => x.labels.iter(),
            RVInstructionNode::Basic(x) => x.labels.iter(),
            RVInstructionNode::Branch(x) => x.labels.iter(),
            RVInstructionNode::Store(x) => x.labels.iter(),
            RVInstructionNode::Load(x) => x.labels.iter(),
            RVInstructionNode::Csr(x) => x.labels.iter(),
            RVInstructionNode::CsrI(x) => x.labels.iter(),
            RVInstructionNode::LoadAddr(x) => x.labels.iter(),
        }
    }

    #[must_use]
    pub fn segment(&self) -> Segment {
        match self {
            RVInstructionNode::Arith(x) => x.segment,
            RVInstructionNode::IArith(x) => x.segment,
            RVInstructionNode::JumpLink(x) => x.segment,
            RVInstructionNode::JumpLinkR(x) => x.segment,
            RVInstructionNode::Basic(x) => x.segment,
            RVInstructionNode::Branch(x) => x.segment,
            RVInstructionNode::Store(x) => x.segment,
            RVInstructionNode::Load(x) => x.segment,
            RVInstructionNode::Csr(x) => x.segment,
            RVInstructionNode::CsrI(x) => x.segment,
            RVInstructionNode::LoadAddr(x) => x.segment,
        }
    }

    #[must_use]
    pub fn token(&self) -> &RawToken {
        match self {
            RVInstructionNode::Arith(x) => &x.token,
            RVInstructionNode::IArith(x) => &x.token,
            RVInstructionNode::JumpLink(x) => &x.token,
            RVInstructionNode::JumpLinkR(x) => &x.token,
            RVInstructionNode::Basic(x) => &x.token,
            RVInstructionNode::Branch(x) => &x.token,
            RVInstructionNode::Store(x) => &x.token,
            RVInstructionNode::Load(x) => &x.token,
            RVInstructionNode::Csr(x) => &x.token,
            RVInstructionNode::CsrI(x) => &x.token,
            RVInstructionNode::LoadAddr(x) => &x.token,
        }
    }
}

impl PartialEq for RVInstructionNode {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
    }
}
impl Eq for RVInstructionNode {}
impl Hash for RVInstructionNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl RVInstructionNode {
    #[must_use]
    pub fn inst(&self) -> RVInst {
        match self {
            RVInstructionNode::Arith(x) => (x.inst.get()).into(),
            RVInstructionNode::IArith(x) => (x.inst.get()).into(),
            RVInstructionNode::JumpLink(x) => (x.inst.get()).into(),
            RVInstructionNode::JumpLinkR(x) => (x.inst.get()).into(),
            RVInstructionNode::Basic(x) => (x.inst.get()).into(),
            RVInstructionNode::Branch(x) => (x.inst.get()).into(),
            RVInstructionNode::Store(x) => (x.inst.get()).into(),
            RVInstructionNode::Load(x) => (x.inst.get()).into(),
            RVInstructionNode::Csr(x) => (x.inst.get()).into(),
            RVInstructionNode::CsrI(x) => (x.inst.get()).into(),
            RVInstructionNode::LoadAddr(_) => RVInst::La,
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
    ) -> RVInstructionNode {
        RVInstructionNode::Arith(Arith {
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
    ) -> RVInstructionNode {
        RVInstructionNode::IArith(IArith {
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
    ) -> RVInstructionNode {
        RVInstructionNode::JumpLink(JumpLink {
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
    ) -> RVInstructionNode {
        RVInstructionNode::JumpLinkR(JumpLinkR {
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
    ) -> RVInstructionNode {
        RVInstructionNode::Basic(Basic {
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
    ) -> RVInstructionNode {
        RVInstructionNode::Branch(Branch {
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
    ) -> RVInstructionNode {
        RVInstructionNode::Store(Store {
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
    ) -> RVInstructionNode {
        RVInstructionNode::Load(Load {
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
    ) -> RVInstructionNode {
        RVInstructionNode::Csr(Csr {
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
    ) -> RVInstructionNode {
        RVInstructionNode::CsrI(CsrI {
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
    ) -> RVInstructionNode {
        RVInstructionNode::LoadAddr(LoadAddr {
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
