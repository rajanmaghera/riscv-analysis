use crate::cfg::BasicBlock;
use crate::parser::inst::Inst;
use crate::parser::lexer::Lexer;
use crate::parser::register::Register;
use crate::parser::token::{LineDisplay, Range, Token, TokenInfo, WithToken};
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::iter::Peekable;
use std::ops::Deref;
use std::rc::Rc;
use uuid::Uuid;

use super::inst::InstType;
use super::token::SymbolData;

// Since we use equality as a way to compare uuids of nodes, this trait is a
// way to check that the contents of an ast node are equal. This is used in
// testing, mostly.

#[derive(Debug, PartialEq, Clone)]
pub struct Imm(pub i32);

impl TryFrom<TokenInfo> for Imm {
    type Error = ();

    fn try_from(value: TokenInfo) -> Result<Self, Self::Error> {
        match value.token {
            Token::Symbol(s) => Imm::try_from(s),
            _ => Err(()),
        }
    }
}

impl TryFrom<SymbolData> for Imm {
    type Error = ();

    fn try_from(value: SymbolData) -> Result<Self, Self::Error> {
        if value.0.starts_with("0x") {
            match i32::from_str_radix(&value.0[2..], 16) {
                Ok(i) => return Ok(Imm(i)),
                Err(_) => return Err(()),
            }
        } else if value.0.starts_with("0b") {
            match i32::from_str_radix(&value.0[2..], 2) {
                Ok(i) => return Ok(Imm(i)),
                Err(_) => return Err(()),
            }
        } else {
            match value.0.parse::<i32>() {
                Ok(i) => Ok(Imm(i)),
                Err(_) => Err(()),
            }
        }
    }
}

trait NodeData {
    fn get_id(&self) -> Uuid;
}

#[derive(Debug, Clone, PartialEq)]
pub enum BasicType {
    Ret,
    Ebreak,
    Ecall,
    Nop,
}


// TODO ensure pseudo instructions are handled correctly
#[derive(Debug, Clone, PartialEq)]
pub enum ArithType {
    Add,
    Addw,
    And,
    Or,
    Sll,
    Sllw,
    Slt,
    Sltu,
    Sra,
    Sraw,
    Srl,
    Srlw,
    Sub,
    Xor,
    Mul,
    Mulh,
    Mulhsu,
    Mulhu,
    Div,
    Divu,
    Divw,
    Rem,
    Remu,
    Remw,
    Remuw,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BranchType {
    Beq,
    Bge,
    Bgeu,
    Blt,
    Bltu,
    Bne,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IArithType {
    Addi,
    Addiw,
    Andi,
    Ori,
    Slli,
    Slliw,
    Slti,
    Sltiu,
    Srai,
    Sraiw,
    Srli,
    Srliw,
    Xori,
    Lui // This is an outlier, but we are going to treat it as an IType
}

// TODO how is pseudo instruction handled for the FromStr trait?

#[derive(Debug, Clone, PartialEq)]
pub enum LoadType {
    Lb,
    Lbu,
    Lh,
    Lhu,
    Lw,
    Lwu,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StoreType {
    Sb,
    Sh,
    Sw,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CSRType {
    Csrrw,
    Csrrs,
    Csrrc,
    Csrrwi,
    Csrrsi,
    Csrrci,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IgnoreType {
    Fence,
    Fencei,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JumpLinkType {
    Jal,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JumpLinkRType {
    Jalr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectiveType {
    Include(WithToken<String>),
    Align(WithToken<i32>),
    Space(WithToken<i32>),
    Text,
    Data
    // TODO include more
}

#[derive(Debug, Clone)]
pub struct Arith {
    pub inst: WithToken<ArithType>,
    pub rd: WithToken<Register>,
    pub rs1: WithToken<Register>,
    pub rs2: WithToken<Register>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct IArith {
    pub inst: WithToken<IArithType>,
    pub rd: WithToken<Register>,
    pub rs1: WithToken<Register>,
    pub imm: WithToken<Imm>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub name: WithToken<String>,
    pub key: Uuid,
}
#[derive(Debug, Clone)]
pub struct JumpLink {
    pub inst: WithToken<JumpLinkType>,
    pub name: WithToken<String>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct JumpLinkR {
    pub inst: WithToken<JumpLinkRType>,
    pub name: WithToken<String>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct Basic {
    pub inst: WithToken<BasicType>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub inst: WithToken<BranchType>,
    pub rs1: WithToken<Register>,
    pub rs2: WithToken<Register>,
    pub name: WithToken<String>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct Load {
    pub inst: WithToken<LoadType>,
    pub rd: WithToken<Register>,
    pub rs1: WithToken<Register>,
    pub imm: WithToken<Imm>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct Store {
    pub inst: WithToken<StoreType>,
    pub rs1: WithToken<Register>,
    pub rs2: WithToken<Register>,
    pub imm: WithToken<Imm>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct Directive {
    pub dir: WithToken<DirectiveType>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct CSR {
    pub inst: WithToken<CSRType>,
    pub rd: WithToken<Register>,
    pub csr: WithToken<Register>,
    pub rs1: WithToken<Register>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct Ignore {
    pub inst: WithToken<IgnoreType>,
    pub key: Uuid,
}
#[derive(Debug, Clone)]
pub enum ASTNode {
    Arith(Arith),
    IArith(IArith),
    Label(Label),
    JumpLink(JumpLink),
    JumpLinkR(JumpLinkR),
    Basic(Basic),
    Directive(Directive),
    Branch(Branch),
    Store(Store), // Stores
    Load(Load),  // Loads, are actually mostly ITypes
    CSR(CSR),
    Ignore(Ignore),
}

#[derive(Debug, Clone)]
pub struct EqNodeWrapper(pub ASTNode);

impl PartialEq for EqNodeWrapper {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (ASTNode::Arith(a), ASTNode::Arith(b)) => {
                a.inst == b.inst && a.rd == b.rd && a.rs1 == b.rs1 && a.rs2 == b.rs2
            },
            (ASTNode::IArith(a), ASTNode::IArith(b)) => {
                a.inst == b.inst && a.rd == b.rd && a.rs1 == b.rs1 && a.imm == b.imm
            },
            (ASTNode::Label(a), ASTNode::Label(b)) => a.name == b.name,
            (ASTNode::JumpLink(a), ASTNode::JumpLink(b)) => a.inst == b.inst && a.name == b.name,
            (ASTNode::JumpLinkR(a), ASTNode::JumpLinkR(b)) => a.inst == b.inst && a.name == b.name,
            (ASTNode::Basic(a), ASTNode::Basic(b)) => a.inst == b.inst,
            (ASTNode::Directive(a), ASTNode::Directive(b)) => a.dir == b.dir,
            (ASTNode::Branch(a), ASTNode::Branch(b)) => {
                a.inst == b.inst && a.rs1 == b.rs1 && a.rs2 == b.rs2 && a.name == b.name
            },
            (ASTNode::Store(a), ASTNode::Store(b)) => {
                a.inst == b.inst && a.rs1 == b.rs1 && a.rs2 == b.rs2 && a.imm == b.imm
            },
            (ASTNode::Load(a), ASTNode::Load(b)) => {
                a.inst == b.inst && a.rd == b.rd && a.rs1 == b.rs1 && a.imm == b.imm
            },
            (ASTNode::CSR(a), ASTNode::CSR(b)) => {
                a.inst == b.inst && a.rd == b.rd && a.csr == b.csr && a.rs1 == b.rs1
            },
            (ASTNode::Ignore(a), ASTNode::Ignore(b)) => a.inst == b.inst,
            _ => false,
        }
    }
}
impl Eq for EqNodeWrapper {}

impl ASTNode {
    pub fn data(&self) -> EqNodeWrapper {
        EqNodeWrapper(self.clone())
    }
}
    pub fn is_entry(&self) -> bool {
        match self {
            ASTNode::Label(_) => true,
            _ => false,
        }
    }

    pub fn is_exit(&self) -> bool {
        match self {
            ASTNode::Branch(_) | ASTNode::Ret | ASTNode::Jmp(_) | ASTNode::Call(_) => true,
            _ => false,
        }
    }

    pub fn stores_to(&self) -> Option<Register> {
        match self {
            ASTNode::Add(rtype) => Some(rtype.0.data.clone()),
            ASTNode::Sub(rtype) => Some(rtype.0.data.clone()),
            _ => None,
        }
    }
}

impl LineDisplay for WithToken<ASTNode> {
    fn get_range(&self) -> Range {
        match &self.data {
            ASTNode::Label(s) => self.pos.clone(),
            ASTNode::Add(rtype) => {
                let mut range = self.pos.clone();
                range.end = rtype.2.pos.end;
                range
            }
            _ => unimplemented!(),
        }
    }
}
pub enum ParseError {
    ExpectedRegister,
    UnexpectedToken,
    UnexpectedEOF,
}

impl TryFrom<&mut Peekable<Lexer>> for WithToken<ASTNode> {
    // TODO fix unwraps

    type Error = ParseError;

    fn try_from(value: &mut Peekable<Lexer>) -> Result<Self, Self::Error> {
        use ParseError::*;
        let next_node = value.next().ok_or(UnexpectedEOF)?;
        match &next_node.token {
            Token::Symbol(s) => {
                if let Ok(inst) = Inst::try_from(s) {
                    let node = match InstType::from(&inst) {
                        InstType::RType => {
                            let rd: WithToken<Register> =
                                WithToken::try_from(value.next().ok_or(UnexpectedToken)?).unwrap();
                            let rs1: WithToken<Register> =
                                WithToken::try_from(value.next().ok_or(UnexpectedToken)?).unwrap();
                            let rs2: WithToken<Register> =
                                WithToken::try_from(value.next().ok_or(UnexpectedToken)?).unwrap();
                            let rtype = RType(rd, rs1, rs2);

                            // TODO we should verify this at compile time that the instruction is valid
                            match inst {
                                Inst::Add => Ok(WithToken::new(ASTNode::Add(rtype), next_node)),
                                Inst::Sub => Ok(WithToken::new(ASTNode::Sub(rtype), next_node)),
                                _ => unimplemented!(),
                            }
                        }
                        _ => Err(UnexpectedToken),
                    };
                    return node;
                }
                Err(UnexpectedToken)
            }
            Token::Label(s) => Ok(WithToken::new(ASTNode::Label(s.to_owned()), next_node)),
            _ => Err(UnexpectedToken),
        }
    }
}
