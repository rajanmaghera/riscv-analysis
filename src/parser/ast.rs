use crate::parser::imm::*;
use crate::parser::inst::Inst;
use crate::parser::inst::*;
use crate::parser::lexer::Lexer;
use crate::parser::mem::*;
use crate::parser::register::Register;
use crate::parser::token::{LineDisplay, Range, Token, WithToken};
use std::convert::TryFrom;
use std::fmt::{self, Display};
use std::hash::{Hash, Hasher};
use std::iter::Peekable;
use std::rc::Rc;
use uuid::Uuid;

use super::imm;
// TODO make a test case with every supported RARS instruction
use super::inst::{self, InstType};
use super::token::TokenInfo;

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
    Data, // TODO include more
}

impl Display for DirectiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DirectiveType::Include(_) => write!(f, ".include"),
            DirectiveType::Align(_) => write!(f, ".align"),
            DirectiveType::Space(_) => write!(f, ".space"),
            DirectiveType::Text => write!(f, ".text"),
            DirectiveType::Data => write!(f, ".data"),
        }
    }
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
    pub rd: WithToken<Register>,
    pub name: WithToken<String>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct JumpLinkR {
    pub inst: WithToken<JumpLinkRType>,
    pub rd: WithToken<Register>,
    pub rs1: WithToken<Register>,
    pub imm: WithToken<Imm>,
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
    pub csr: WithToken<Imm>,
    pub rs1: WithToken<Register>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct Ignore {
    pub inst: WithToken<IgnoreType>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct LoadAddr {
    pub inst: WithToken<PseudoType>,
    pub rd: WithToken<Register>,
    pub name: WithToken<String>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct UpperArith {
    pub inst: WithToken<UpperArithType>,
    pub rd: WithToken<Register>,
    pub imm: WithToken<Imm>,
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
    Store(Store),       // Stores
    Load(Load),         // Loads, are actually mostly ITypes
    LoadAddr(LoadAddr), // Load address
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
            }
            (ASTNode::IArith(a), ASTNode::IArith(b)) => {
                a.inst == b.inst && a.rd == b.rd && a.rs1 == b.rs1 && a.imm == b.imm
            }
            (ASTNode::Label(a), ASTNode::Label(b)) => a.name == b.name,
            (ASTNode::JumpLink(a), ASTNode::JumpLink(b)) => a.inst == b.inst && a.name == b.name,
            (ASTNode::JumpLinkR(a), ASTNode::JumpLinkR(b)) => {
                a.inst == b.inst && a.rd == b.rd && a.rs1 == b.rs1 && a.imm == b.imm
            }
            (ASTNode::Basic(a), ASTNode::Basic(b)) => a.inst == b.inst,
            (ASTNode::Directive(a), ASTNode::Directive(b)) => a.dir == b.dir,
            (ASTNode::Branch(a), ASTNode::Branch(b)) => {
                a.inst == b.inst && a.rs1 == b.rs1 && a.rs2 == b.rs2 && a.name == b.name
            }
            (ASTNode::Store(a), ASTNode::Store(b)) => {
                a.inst == b.inst && a.rs1 == b.rs1 && a.rs2 == b.rs2 && a.imm == b.imm
            }
            (ASTNode::Load(a), ASTNode::Load(b)) => {
                a.inst == b.inst && a.rd == b.rd && a.rs1 == b.rs1 && a.imm == b.imm
            }
            (ASTNode::CSR(a), ASTNode::CSR(b)) => {
                a.inst == b.inst && a.rd == b.rd && a.csr == b.csr && a.rs1 == b.rs1
            }
            (ASTNode::Ignore(a), ASTNode::Ignore(b)) => a.inst == b.inst,
            (ASTNode::LoadAddr(a), ASTNode::LoadAddr(b)) => {
                a.inst == b.inst && a.rd == b.rd && a.name == b.name
            }
            _ => false,
        }
    }
}
impl Eq for EqNodeWrapper {}

pub trait EqNodeData {
    fn data(&self) -> EqNodeWrapper;
}

pub trait EqNodeDataVec {
    fn data(&self) -> Vec<EqNodeWrapper>;
}

impl EqNodeData for ASTNode {
    fn data(&self) -> EqNodeWrapper {
        EqNodeWrapper(self.clone())
    }
}

impl EqNodeDataVec for Vec<ASTNode> {
    fn data(&self) -> Vec<EqNodeWrapper> {
        self.iter().map(|x| x.data()).collect()
    }
}

impl EqNodeDataVec for Vec<Rc<ASTNode>> {
    fn data(&self) -> Vec<EqNodeWrapper> {
        self.iter().map(|x| x.data()).collect()
    }
}

impl NodeData for ASTNode {
    fn get_id(&self) -> Uuid {
        match self {
            ASTNode::Arith(a) => a.key,
            ASTNode::IArith(a) => a.key,
            ASTNode::UpperArith(a) => a.key,
            ASTNode::Label(a) => a.key,
            ASTNode::JumpLink(a) => a.key,
            ASTNode::JumpLinkR(a) => a.key,
            ASTNode::Basic(a) => a.key,
            ASTNode::Directive(a) => a.key,
            ASTNode::Branch(a) => a.key,
            ASTNode::Store(a) => a.key,
            ASTNode::Load(a) => a.key,
            ASTNode::CSR(a) => a.key,
            ASTNode::Ignore(a) => a.key,
            ASTNode::LoadAddr(a) => a.key,
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
        inst: WithToken<ArithType>,
        rd: WithToken<Register>,
        rs1: WithToken<Register>,
        rs2: WithToken<Register>,
    ) -> ASTNode {
        ASTNode::Arith(Arith {
            inst,
            rd,
            rs1,
            rs2,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_iarith(
        inst: WithToken<IArithType>,
        rd: WithToken<Register>,
        rs1: WithToken<Register>,
        imm: WithToken<Imm>,
    ) -> ASTNode {
        ASTNode::IArith(IArith {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_upper_arith(
        inst: WithToken<UpperArithType>,
        rd: WithToken<Register>,
        imm: WithToken<Imm>,
    ) -> ASTNode {
        ASTNode::UpperArith(UpperArith {
            inst,
            rd,
            imm,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_jump_link(
        inst: WithToken<JumpLinkType>,
        rd: WithToken<Register>,
        name: WithToken<String>,
    ) -> ASTNode {
        ASTNode::JumpLink(JumpLink {
            inst,
            rd,
            name,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_jump_link_r(
        inst: WithToken<JumpLinkRType>,
        rd: WithToken<Register>,
        rs1: WithToken<Register>,
        imm: WithToken<Imm>,
    ) -> ASTNode {
        ASTNode::JumpLinkR(JumpLinkR {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_basic(inst: WithToken<BasicType>) -> ASTNode {
        ASTNode::Basic(Basic {
            inst,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_directive(dir: WithToken<DirectiveType>) -> ASTNode {
        ASTNode::Directive(Directive {
            dir,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_branch(
        inst: WithToken<BranchType>,
        rs1: WithToken<Register>,
        rs2: WithToken<Register>,
        name: WithToken<String>,
    ) -> ASTNode {
        ASTNode::Branch(Branch {
            inst,
            rs1,
            rs2,
            name,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_store(
        inst: WithToken<StoreType>,
        rs1: WithToken<Register>,
        rs2: WithToken<Register>,
        imm: WithToken<Imm>,
    ) -> ASTNode {
        ASTNode::Store(Store {
            inst,
            rs1,
            rs2,
            imm,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_load(
        inst: WithToken<LoadType>,
        rd: WithToken<Register>,
        rs1: WithToken<Register>,
        imm: WithToken<Imm>,
    ) -> ASTNode {
        ASTNode::Load(Load {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_csr(
        inst: WithToken<CSRType>,
        rd: WithToken<Register>,
        csr: WithToken<Imm>,
        rs1: WithToken<Register>,
    ) -> ASTNode {
        ASTNode::CSR(CSR {
            inst,
            rd,
            rs1,
            csr,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_ignore(inst: WithToken<IgnoreType>) -> ASTNode {
        ASTNode::Ignore(Ignore {
            inst,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_label(name: WithToken<String>) -> ASTNode {
        ASTNode::Label(Label {
            name,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_load_addr(
        inst: WithToken<PseudoType>,
        rd: WithToken<Register>,
        name: WithToken<String>,
    ) -> ASTNode {
        ASTNode::LoadAddr(LoadAddr {
            inst,
            rd,
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
            ASTNode::JumpLink(_) => true,
            ASTNode::JumpLinkR(_) => true,
            ASTNode::Branch(_) => true,
            _ => false,
        }
    }

    // NOTE: This is in context to a register store, not a memory store
    pub fn stores_to(&self) -> Option<WithToken<Register>> {
        match self {
            ASTNode::Load(load) => Some(load.rd.clone()),
            ASTNode::Arith(arith) => Some(arith.rd.clone()),
            ASTNode::IArith(iarith) => Some(iarith.rd.clone()),
            ASTNode::CSR(csr) => Some(csr.rd.clone()),
            _ => None,
        }
    }
}

// -- PRETTY PRINTING --
pub struct VecASTDisplayWrapper<'a>(&'a Vec<ASTNode>);
pub trait ToDisplayForVecASTNode {
    fn to_display(&self) -> VecASTDisplayWrapper;
}
impl ToDisplayForVecASTNode for Vec<ASTNode> {
    fn to_display(&self) -> VecASTDisplayWrapper {
        VecASTDisplayWrapper(self)
    }
}

impl<'a> fmt::Display for VecASTDisplayWrapper<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut last = false;
        for node in self.0 {
            if last {
                write!(f, "\n")?;
            }
            let out = match node {
                ASTNode::UpperArith(x) => {
                    let inst: Inst = Inst::from(&x.inst.data);
                    let rd = x.rd.data.to_string();
                    let imm = x.imm.data.0.to_string();
                    format!("{} {} <- {}", inst, rd, imm)
                }
                ASTNode::Arith(x) => {
                    let inst: Inst = Inst::from(&x.inst.data);
                    let rd = x.rd.data.to_string();
                    let rs1 = x.rs1.data.to_string();
                    let rs2 = x.rs2.data.to_string();
                    format!("{} {} <- {}, {}", inst, rd, rs1, rs2)
                }
                ASTNode::IArith(x) => {
                    let inst: Inst = Inst::from(&x.inst.data);
                    let rd = x.rd.data.to_string();
                    let rs1 = x.rs1.data.to_string();
                    let imm = x.imm.data.0.to_string();
                    format!("{} {} <- {}, {}", inst, rd, rs1, imm)
                }
                ASTNode::Label(x) => format!("---[{}]---", x.name.data),
                ASTNode::JumpLink(x) => {
                    let inst: Inst = Inst::from(&x.inst.data);
                    let name = x.name.data.to_string();
                    let rd = x.rd.data.to_string();
                    format!("{} [{}] | {} <- PC", inst, name, rd)
                }
                ASTNode::JumpLinkR(x) => {
                    let inst: Inst = Inst::from(&x.inst.data);
                    let rs1 = x.rs1.data.to_string();
                    format!("{} [{}]", inst, rs1)
                }
                ASTNode::Basic(x) => {
                    let inst: Inst = Inst::from(&x.inst.data);
                    format!("{}", inst)
                }
                ASTNode::Directive(x) => {
                    let dir = x.dir.data.to_string();
                    format!("-<{}>-", dir)
                }
                ASTNode::Branch(x) => {
                    let inst: Inst = Inst::from(&x.inst.data);
                    let rs1 = x.rs1.data.to_string();
                    let rs2 = x.rs2.data.to_string();
                    let name = x.name.data.to_string();
                    format!("{} {}--{}, [{}]", inst, rs1, rs2, name)
                }
                ASTNode::Store(x) => {
                    let inst: Inst = Inst::from(&x.inst.data);
                    let rs1 = x.rs1.data.to_string();
                    let rs2 = x.rs2.data.to_string();
                    let imm = x.imm.data.0.to_string();
                    format!("{} {} -> {}({})", inst, rs2, imm, rs1)
                }
                ASTNode::Load(x) => {
                    let inst: Inst = Inst::from(&x.inst.data);
                    let rd = x.rd.data.to_string();
                    let rs1 = x.rs1.data.to_string();
                    let imm = x.imm.data.0.to_string();
                    format!("{} {} <- {}({})", inst, rd, imm, rs1)
                }
                // TODO don't use the pseudo type here
                ASTNode::LoadAddr(x) => {
                    let inst = "la";
                    let rd = x.rd.data.to_string();
                    let name = x.name.data.to_string();
                    format!("{} {} <- [{}]", inst, rd, name)
                }
                ASTNode::CSR(x) => {
                    let inst: Inst = Inst::from(&x.inst.data);
                    let rd = x.rd.data.to_string();
                    let csr = x.csr.data.0.to_string();
                    format!("{} {}, {}", inst, rd, csr)
                }
                ASTNode::Ignore(_) => {
                    format!("<ignored>")
                }
            };
            write!(f, "{}", out)?;
            last = true;
        }
        Ok(())
    }
}

// TODO this might differ based on how the nodes are made
impl LineDisplay for WithToken<ASTNode> {
    fn get_range(&self) -> Range {
        match &self.data {
            ASTNode::UpperArith(x) => {
                let mut range = self.pos.clone();
                range.end = x.imm.pos.end.clone();
                range
            }
            ASTNode::Label(_) => self.pos.clone(),
            ASTNode::Arith(arith) => {
                let mut range = self.pos.clone();
                range.end = arith.rs2.pos.end.clone();
                range
            }
            ASTNode::IArith(iarith) => {
                let mut range = self.pos.clone();
                range.end = iarith.imm.pos.end.clone();
                range
            }
            ASTNode::JumpLink(jl) => {
                let mut range = self.pos.clone();
                range.end = jl.name.pos.end.clone();
                range
            }
            ASTNode::JumpLinkR(jlr) => {
                let mut range = self.pos.clone();
                range.end = jlr.inst.pos.end.clone();
                range
            }
            ASTNode::Branch(branch) => {
                let mut range = self.pos.clone();
                range.end = branch.name.pos.end.clone();
                range
            }
            ASTNode::Store(store) => {
                let mut range = self.pos.clone();
                range.end = store.imm.pos.end.clone();
                range
            }
            ASTNode::Load(load) => {
                let mut range = self.pos.clone();
                range.end = load.imm.pos.end.clone();
                range
            }
            ASTNode::CSR(csr) => {
                let mut range = self.pos.clone();
                range.end = csr.csr.pos.end.clone();
                range
            }
            ASTNode::Ignore(_) => self.pos.clone(),
            ASTNode::Basic(_) => self.pos.clone(),
            ASTNode::LoadAddr(_) => self.pos.clone(),
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
                }
                DirectiveType::Space(item) => {
                    let mut range = self.pos.clone();
                    range.end = item.pos.end.clone();
                    range
                }
            },
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum ParseError {
    ExpectedRegister,
    ExpectedImm,
    ExpectedLabel,
    IsNewline,
    Ignored,
    UnexpectedToken,
    UnexpectedEOF,
}

impl TryFrom<&mut Peekable<Lexer>> for ASTNode {
    // TODO errors are not robust

    type Error = ParseError;

    fn try_from(value: &mut Peekable<Lexer>) -> Result<Self, Self::Error> {
        use ParseError::*;
        let next_node = value.next().ok_or(UnexpectedEOF)?;
        match &next_node.token {
            Token::Symbol(s) => {
                // TODO implement loads with % syntax
                if let Ok(inst) = Inst::try_from(s) {
                    let node = match InstType::from(&inst) {
                        InstType::ArithType(inst) => {
                            let rd = WithToken::<Register>::try_from(
                                value.next().ok_or(UnexpectedToken)?,
                            )
                            .map_err(|_| ExpectedRegister)?;
                            let rs1 = WithToken::<Register>::try_from(
                                value.next().ok_or(UnexpectedToken)?,
                            )
                            .map_err(|_| ExpectedRegister)?;
                            let rs2 = WithToken::<Register>::try_from(
                                value.next().ok_or(UnexpectedToken)?,
                            )
                            .map_err(|_| ExpectedRegister)?;
                            Ok(ASTNode::new_arith(
                                WithToken::new(inst, next_node),
                                rd,
                                rs1,
                                rs2,
                            ))
                        }
                        InstType::IArithType(inst) => {
                            let rd: WithToken<Register> =
                                WithToken::try_from(value.next().ok_or(UnexpectedToken)?)
                                    .map_err(|_| ExpectedRegister)?;
                            let rs1: WithToken<Register> =
                                WithToken::try_from(value.next().ok_or(UnexpectedToken)?)
                                    .map_err(|_| ExpectedRegister)?;
                            let imm: WithToken<Imm> =
                                WithToken::try_from(value.next().ok_or(UnexpectedToken)?)
                                    .map_err(|_| ExpectedImm)?;
                            Ok(ASTNode::new_iarith(
                                WithToken::new(inst, next_node),
                                rd,
                                rs1,
                                imm,
                            ))
                        }

                        // TODO ensure that symbol is not a register
                        // TODO how is error handling handled for non registers
                        // TODO can we clean this up
                        // TODO .clone() is probably not what we want
                        InstType::JumpLinkType(inst) => {
                            let name_token = value.next().ok_or(UnexpectedToken)?;
                            if let Ok(reg) = WithToken::<Register>::try_from(name_token.clone()) {
                                let next = value.next().ok_or(UnexpectedToken)?;
                                let name: WithToken<String> =
                                    WithToken::new(next.token.to_string(), name_token);
                                return Ok(ASTNode::new_jump_link(
                                    WithToken::new(inst, next_node),
                                    reg,
                                    name,
                                ));
                            }
                            let name: WithToken<String> =
                                WithToken::new(name_token.token.to_string(), name_token.clone());
                            Ok(ASTNode::new_jump_link(
                                WithToken::new(inst, next_node),
                                WithToken::new(Register::X1, name_token),
                                name,
                            ))
                        }
                        InstType::JumpLinkRType(inst) => {
                            let reg1 = WithToken::<Register>::try_from(
                                value.next().ok_or(UnexpectedToken)?,
                            )
                            .map_err(|_| ExpectedRegister)?;

                            let next = value.next().ok_or(UnexpectedToken)?;
                            return if let Ok(rs1) = WithToken::<Register>::try_from(
                                next.clone(),
                            ) {
                                let next = value.next().ok_or(UnexpectedToken)?;
                                let imm =
                                    WithToken::<Imm>::try_from(next).map_err(|_| ExpectedImm)?;
                                Ok(ASTNode::new_jump_link_r(
                                    WithToken::new(inst, next_node),
                                    reg1,
                                    rs1,
                                    imm,
                                ))
                            } else if let Ok(mem) =
                                WithToken::<Mem>::try_from(next.clone())
                            {
                                Ok(ASTNode::new_jump_link_r(
                                    WithToken::new(inst, next_node),
                                    reg1,
                                    mem.data.reg,
                                    mem.data.offset,
                                ))
                            } else if let Ok(imm) =
                                WithToken::<Imm>::try_from(next.clone())
                            {
                                Ok(ASTNode::new_jump_link_r(
                                    WithToken::new(inst, next_node.clone()),
                                    WithToken::new(Register::X1, next_node),
                                    reg1,
                                    imm,
                                ))
                            } else {
                                Ok(ASTNode::new_jump_link_r(
                                    WithToken::new(inst, next_node.clone()),
                                    WithToken::new(Register::X1, next_node.clone()),
                                    reg1,
                                    WithToken::new(Imm(0), next_node),
                                ))
                            };
                        }
                        InstType::LoadType(inst) => {
                            let rd = WithToken::<Register>::try_from(
                                value.next().ok_or(UnexpectedToken)?,
                            )
                            .map_err(|_| ExpectedRegister)?;
                            let next = value.next().ok_or(UnexpectedToken)?;
                            return if let Ok(mem) =
                                WithToken::<Mem>::try_from(next.clone())
                            {
                                Ok(ASTNode::new_load(
                                    WithToken::new(inst, next_node),
                                    rd,
                                    mem.data.reg,
                                    mem.data.offset,
                                ))
                            } else if let Ok(imm) =
                                WithToken::<Imm>::try_from(next.clone())
                            {
                                Ok(ASTNode::new_load(
                                    WithToken::new(inst, next_node.clone()),
                                    rd,
                                    WithToken::new(Register::X0, next_node),
                                    imm,
                                ))
                                // TODO swtich label type to LabelType struct, to
                                // disallow characters like ( or )
                            } else if let Ok(label) =
                                WithToken::<String>::try_from(next)
                            {  
                                dbg!(label);
                                unimplemented!("Implement label loading, turning one instruction into two AST nodes")
                            } else {
                                Err(UnexpectedToken)
                            };
                        }
                        InstType::StoreType(inst) => {
                            let rs2 = WithToken::<Register>::try_from(
                                value.next().ok_or(UnexpectedToken)?,
                            )
                            .map_err(|_| ExpectedRegister)?;

                            let next = value.next().ok_or(UnexpectedToken)?;
                            return if let Ok(mem) =
                                WithToken::<Mem>::try_from(next.clone())
                            {
                                Ok(ASTNode::new_store(
                                    WithToken::new(inst, next_node),
                                    rs2,
                                    mem.data.reg,
                                    mem.data.offset,
                                ))
                            } else if let Ok(imm) =
                                WithToken::<Imm>::try_from(next.clone())
                            {
                                Ok(ASTNode::new_store(
                                    WithToken::new(inst, next_node.clone()),
                                    rs2,
                                    WithToken::new(Register::X0, next_node),
                                    imm,
                                ))
                            } else if let Ok(_) =
                                WithToken::<String>::try_from(next)
                            {
                                unimplemented!("Implement label storing, turning one instruction into two AST nodes")
                            } else {
                                Err(UnexpectedToken)
                            };
                        }
                        InstType::BranchType(inst) => {
                            let rs1 = WithToken::<Register>::try_from(
                                value.next().ok_or(UnexpectedToken)?,
                            )
                            .map_err(|_| ExpectedRegister)?;
                            let rs2 = WithToken::<Register>::try_from(
                                value.next().ok_or(UnexpectedToken)?,
                            )
                            .map_err(|_| ExpectedRegister)?;
                            let label =
                                WithToken::<String>::try_from(value.next().ok_or(UnexpectedToken)?)
                                    .map_err(|_| ExpectedLabel)?;
                            Ok(ASTNode::new_branch(
                                WithToken::new(inst, next_node),
                                rs1,
                                rs2,
                                label,
                            ))
                        }
                        InstType::IgnoreType(_) => Err(Ignored),
                        InstType::BasicType(inst) => {
                            Ok(ASTNode::new_basic(WithToken::new(inst, next_node)))
                        }
                        InstType::CSRType(_) => {
                            unimplemented!("Implement CSR instructions")
                        }
                        InstType::PseudoType(inst) => {
                            dbg!(&inst);
                            match inst {
                                PseudoType::Ret => {
                                    return Ok(ASTNode::new_jump_link_r(
                                        WithToken::new(JumpLinkRType::Jalr, next_node.clone()),
                                        WithToken::new(Register::X0, next_node.clone()),
                                        WithToken::new(Register::X1, next_node.clone()),
                                        WithToken::new(Imm(0), next_node.clone()),
                                    ))
                                }
                                PseudoType::Mv => {
                                    let rd = WithToken::<Register>::try_from(
                                        value.next().ok_or(UnexpectedToken)?,
                                    )
                                    .map_err(|_| ExpectedRegister)?;
                                    let rs1 = WithToken::<Register>::try_from(
                                        value.next().ok_or(UnexpectedToken)?,
                                    )
                                    .map_err(|_| ExpectedRegister)?;
                                    return Ok(ASTNode::new_arith(
                                        WithToken::new(ArithType::Add, next_node.clone()),
                                        rd,
                                        rs1,
                                        WithToken::new(Register::X0, next_node.clone()),
                                    ));
                                }
                                PseudoType::Li => {
                                    let rd = WithToken::<Register>::try_from(
                                        value.next().ok_or(UnexpectedToken)?,
                                    )
                                    .map_err(|_| ExpectedRegister)?;
                                    let imm = WithToken::<Imm>::try_from(
                                        value.next().ok_or(UnexpectedToken)?,
                                    )
                                    .map_err(|_| ExpectedImm)?;
                                    return Ok(ASTNode::new_iarith(
                                        WithToken::new(IArithType::Addi, next_node.clone()),
                                        rd,
                                        WithToken::new(Register::X0, next_node.clone()),
                                        imm,
                                    ));
                                }
                                PseudoType::La => {
                                    let rd = WithToken::<Register>::try_from(
                                        value.next().ok_or(UnexpectedToken)?,
                                    )
                                    .map_err(|_| ExpectedRegister)?;
                                    let label = WithToken::<String>::try_from(
                                        value.next().ok_or(UnexpectedToken)?,
                                    )
                                    .map_err(|_| ExpectedLabel)?;
                                    return Ok(ASTNode::new_load_addr(
                                        WithToken::new(PseudoType::La, next_node.clone()),
                                        rd,
                                        label,
                                    ));
                                }
                                PseudoType::J => {
                                    let label = WithToken::<String>::try_from(
                                        value.next().ok_or(UnexpectedToken)?,
                                    )
                                    .map_err(|_| ExpectedLabel)?;

                                    return Ok(ASTNode::new_jump_link(
                                        WithToken::new(JumpLinkType::Jal, next_node.clone()),
                                        WithToken::new(Register::X0, next_node.clone()),
                                        label,
                                    ));
                                }
                                _ => {
                                    unimplemented!("Implement pseudo instructions")
                                }
                            }
                        }
                    };
                    return node;
                }
                Err(UnexpectedToken)
            }
            Token::Label(s) => Ok(ASTNode::new_label(WithToken::new(s.clone(), next_node))),
            Token::Directive(_) => Err(Ignored),
            Token::Newline => Err(IsNewline),
            _ => unimplemented!(),
        }
    }
}
