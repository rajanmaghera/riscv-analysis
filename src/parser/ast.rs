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
trait NodeData {
    fn get_id(&self) -> Uuid;
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

impl NodeData for ASTNode {
    fn get_id(&self) -> Uuid {
        match self {
            ASTNode::RType(rtype) => rtype.key,
            ASTNode::IType(itype) => itype.key,
            ASTNode::Label(label) => label.key,
            ASTNode::JumpType(jtype) => jtype.key,
            ASTNode::BasicType(btype) => btype.key,
            ASTNode::Directive(directive) => directive.key,
        }
    }
}

impl PartialEq for dyn NodeData {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}
impl Eq for dyn NodeData {}
impl Hash for dyn NodeData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_id().hash(state);
    }
}

impl ASTNode {

    // TODO derive AST new funcs using procedural macros

    pub fn new_arith(
        inst: WithToken<Arith>,
        rd: WithToken<Register>,
        rs1: WithToken<Register>,
        rs2: WithToken<Register>,
    ) -> ASTNode {
        ASTNode::RType(RType {
            inst,
            rd,
            rs1,
            rs2,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_itype(
        inst: WithToken<ITypeInst>,
        rd: WithToken<Register>,
        rs1: WithToken<Register>,
        imm: WithToken<Imm>,
    ) -> ASTNode {
        ASTNode::IType(IType {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_label(name: WithToken<String>) -> ASTNode {
        ASTNode::Label(LabelData {
            name,
            key: Uuid::new_v4(),
        })
    }

    pub fn is_entry(&self) -> bool {
        match self {
            ASTNode::Label(_) => true,
            _ => false,
        }
    }

    pub fn is_exit(&self) -> bool {
        match self {
            ASTNode::JumpType(_) => true,
            _ => false,
        }
    }

    pub fn stores_to(&self) -> Option<WithToken<Register>> {
        match self {
            ASTNode::RType(rtype) => Some(rtype.rd.clone()),
            ASTNode::IType(itype) => Some(itype.rd.clone()),
            _ => None,
        }
    }
}

impl LineDisplay for WithToken<ASTNode> {
    fn get_range(&self) -> Range {
        match &self.data {
            ASTNode::Label(s) => self.pos.clone(),
            ASTNode::RType(rtype) => {
                let mut range = self.pos.clone();
                range.end = rtype.rs2.pos.end;
                range
            }
            ASTNode::IType(itype) => {
                let mut range = self.pos.clone();
                range.end = itype.imm.pos.end;
                range
            }
            ASTNode::JumpType(jtype) => {
                let mut range = self.pos.clone();
                range.end = jtype.name.pos.end;
                range
            }
            ASTNode::BasicType(_) => self.pos.clone(),
            ASTNode::Directive(directive) => match &directive.dir.data {
                DirectiveType::Data => self.pos.clone(),
                DirectiveType::Text => self.pos.clone(),
                DirectiveType::Include(incl) => {
                    let mut range = self.pos.clone();
                    range.end = incl.pos.end.clone();
                    range
                }
                DirectiveType::Align(align) => {
                    let mut range = self.pos.clone();
                    range.end = align.pos.end.clone();
                    range
                },
                DirectiveType::Space(item) => {
                    let mut range = self.pos.clone();
                    range.end = item.pos.end.clone();
                    range
                },
            },
        }
    }
}
pub enum ParseError {
    ExpectedRegister,
    UnexpectedToken,
    UnexpectedEOF,
}

impl TryFrom<&mut Peekable<Lexer>> for ASTNode {
    // TODO fix unwraps

    type Error = ParseError;

    fn try_from(value: &mut Peekable<Lexer>) -> Result<Self, Self::Error> {
        use ParseError::*;
        let next_node = value.next().ok_or(UnexpectedEOF)?;
        match &next_node.token {
            Token::Symbol(s) => {
                dbg!(s);
                if let Ok(inst) = Inst::try_from(s) {
                    let node = match InstType::from(&inst) {
                        InstType::RType(inst) => {
                            let rd: WithToken<Register> =
                                WithToken::try_from(value.next().ok_or(UnexpectedToken)?).unwrap();
                            let rs1: WithToken<Register> =
                                WithToken::try_from(value.next().ok_or(UnexpectedToken)?).unwrap();
                            let rs2: WithToken<Register> =
                                WithToken::try_from(value.next().ok_or(UnexpectedToken)?).unwrap();
                            Ok(ASTNode::new_rtype(
                                WithToken::new(inst, next_node),
                                rd,
                                rs1,
                                rs2,
                            ))
                        }
                        InstType::IType(inst) => {
                            let rd: WithToken<Register> =
                                WithToken::try_from(value.next().ok_or(UnexpectedToken)?).unwrap();
                            let rs1: WithToken<Register> =
                                WithToken::try_from(value.next().ok_or(UnexpectedToken)?).unwrap();
                            let imm: WithToken<Imm> =
                                WithToken::try_from(value.next().ok_or(UnexpectedToken)?).unwrap();
                            Ok(ASTNode::new_itype(
                                WithToken::new(inst, next_node),
                                rd,
                                rs1,
                                imm,
                            ))
                        }
                        _ => Err(UnexpectedToken),
                    };
                    return node;
                }
                Err(UnexpectedToken)
            }
            Token::Label(s) => Ok(ASTNode::new_label(WithToken::new(s.clone(), next_node))),
            _ => Err(UnexpectedToken),
        }
    }
}
