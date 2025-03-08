use std::fmt::Display;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ArithType, BasicType, BranchType, CsrIType, CsrImm, CsrType, DirectiveToken, IArithType,
    IgnoreType, Imm, JumpLinkRType, JumpLinkType, LabelStringToken, LoadType, PseudoType, RawToken,
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
    pub name: LabelStringToken,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpLink {
    pub inst: With<JumpLinkType>,
    pub rd: With<Register>,
    pub name: LabelStringToken,
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
    pub name: LabelStringToken,
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
            DirectiveType::Align(i) => write!(f, "align {}", i.get().value()),
            DirectiveType::Ascii { text, .. } => {
                write!(f, "ascii \"{text}\"")
            }
            DirectiveType::DataSection => write!(f, ".data"),
            DirectiveType::TextSection => write!(f, ".text"),
            DirectiveType::Data(dt, data) => {
                write!(f, "{dt} ")?;
                for d in data {
                    write!(f, "{}, ", d.get().value())?;
                }
                Ok(())
            }
            DirectiveType::Space(i) => write!(f, "space {}", i.get().value()),
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
    pub inst: With<CsrType>,
    pub rd: With<Register>,
    pub csr: With<CsrImm>,
    pub rs1: With<Register>,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip)]
    pub token: RawToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrI {
    pub inst: With<CsrIType>,
    pub rd: With<Register>,
    pub csr: With<CsrImm>,
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
    pub name: LabelStringToken,
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
    #[serde(skip)]
    pub is_interrupt_handler: bool,
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
