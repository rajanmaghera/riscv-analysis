use std::fmt::Display;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ArithType, BasicType, BranchType, CSRIType, CSRImm, CSRType, DirectiveToken, IArithType,
    IgnoreType, Imm, JumpLinkRType, JumpLinkType, LabelString, LoadType, PseudoType, RawToken,
    Register, StoreType, With,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arith {
    pub inst: With<ArithType>,
    pub rd: With<Register>,
    pub rs1: With<Register>,
    pub rs2: With<Register>,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IArith {
    pub inst: With<IArithType>,
    pub rd: With<Register>,
    pub rs1: With<Register>,
    pub imm: With<Imm>,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub name: With<LabelString>,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpLink {
    pub inst: With<JumpLinkType>,
    pub rd: With<Register>,
    pub name: With<LabelString>,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpLinkR {
    pub inst: With<JumpLinkRType>,
    pub rd: With<Register>,
    pub rs1: With<Register>,
    pub imm: With<Imm>,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Basic {
    pub inst: With<BasicType>,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub inst: With<BranchType>,
    pub rs1: With<Register>,
    pub rs2: With<Register>,
    pub name: With<LabelString>,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Load {
    pub inst: With<LoadType>,
    pub rd: With<Register>,
    pub rs1: With<Register>,
    pub imm: With<Imm>,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    pub inst: With<StoreType>,
    pub rs1: With<Register>,
    pub rs2: With<Register>,
    pub imm: With<Imm>,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum DataType {
    Byte,
    Half,
    Word,
    Double,
    Dword,
    Float,
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Byte => write!(f, "byte"),
            DataType::Half => write!(f, "half"),
            DataType::Word => write!(f, "word"),
            DataType::Double => write!(f, "double"),
            DataType::Dword => write!(f, "dword"),
            DataType::Float => write!(f, "float"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirectiveType {
    Include(With<String>),
    Align(With<Imm>),
    Ascii { text: With<String>, null_term: bool },
    DataSection,
    TextSection,
    Data(DataType, Vec<With<Imm>>),
    Space(With<Imm>),
}

impl Display for DirectiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirectiveType::Include(s) => write!(f, "include {s}"),
            DirectiveType::Align(i) => write!(f, "align {}", i.data.0),
            DirectiveType::Ascii { text, .. } => {
                write!(f, "ascii \"{}\"", text.data)
            }
            DirectiveType::DataSection => write!(f, ".data"),
            DirectiveType::TextSection => write!(f, ".text"),
            DirectiveType::Data(dt, data) => {
                write!(f, "{dt} ")?;
                for d in data {
                    write!(f, "{}, ", d.data.0)?;
                }
                Ok(())
            }
            DirectiveType::Space(i) => write!(f, "space {}", i.data.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directive {
    pub dir_token: With<DirectiveToken>,
    pub dir: DirectiveType,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Csr {
    pub inst: With<CSRType>,
    pub rd: With<Register>,
    pub csr: With<CSRImm>,
    pub rs1: With<Register>,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrI {
    pub inst: With<CSRIType>,
    pub rd: With<Register>,
    pub csr: With<CSRImm>,
    pub imm: With<Imm>,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ignore {
    pub inst: With<IgnoreType>,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadAddr {
    pub inst: With<PseudoType>,
    pub rd: With<Register>,
    pub name: With<LabelString>,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuncEntry {
    #[serde(skip)]
    pub file: Uuid,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramEntry {
    #[serde(skip)]
    pub file: Uuid,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}
