use std::fmt::Display;

use uuid::Uuid;

use super::{
    ArithType, BasicType, BranchType, CSRIType, CSRImm, CSRType, DirectiveToken, IArithType,
    IgnoreType, Imm, JumpLinkRType, JumpLinkType, LabelString, LoadType, PseudoType, Register,
    StoreType, UpperArithType, With,
};

#[derive(Debug, Clone)]
pub struct Arith {
    pub inst: With<ArithType>,
    pub rd: With<Register>,
    pub rs1: With<Register>,
    pub rs2: With<Register>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct IArith {
    pub inst: With<IArithType>,
    pub rd: With<Register>,
    pub rs1: With<Register>,
    pub imm: With<Imm>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub name: With<LabelString>,
    pub key: Uuid,
}
#[derive(Debug, Clone)]
pub struct JumpLink {
    pub inst: With<JumpLinkType>,
    pub rd: With<Register>,
    pub name: With<LabelString>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct JumpLinkR {
    pub inst: With<JumpLinkRType>,
    pub rd: With<Register>,
    pub rs1: With<Register>,
    pub imm: With<Imm>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct Basic {
    pub inst: With<BasicType>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub inst: With<BranchType>,
    pub rs1: With<Register>,
    pub rs2: With<Register>,
    pub name: With<LabelString>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct Load {
    pub inst: With<LoadType>,
    pub rd: With<Register>,
    pub rs1: With<Register>,
    pub imm: With<Imm>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct Store {
    pub inst: With<StoreType>,
    pub rs1: With<Register>,
    pub rs2: With<Register>,
    pub imm: With<Imm>,
    pub key: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectiveType {
    Include(With<String>),
}

impl Display for DirectiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirectiveType::Include(s) => write!(f, "include {}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Directive {
    pub token: With<DirectiveToken>,
    pub dir: DirectiveType,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct Csr {
    pub inst: With<CSRType>,
    pub rd: With<Register>,
    pub csr: With<CSRImm>,
    pub rs1: With<Register>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct CsrI {
    pub inst: With<CSRIType>,
    pub rd: With<Register>,
    pub csr: With<CSRImm>,
    pub imm: With<Imm>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct Ignore {
    pub inst: With<IgnoreType>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct LoadAddr {
    pub inst: With<PseudoType>,
    pub rd: With<Register>,
    pub name: With<LabelString>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct UpperArith {
    pub inst: With<UpperArithType>,
    pub rd: With<Register>,
    pub imm: With<Imm>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct FuncEntry {
    pub file: Uuid,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct ProgramEntry {
    pub file: Uuid,
    pub key: Uuid,
}
