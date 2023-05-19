use crate::parser::imm::*;
use crate::parser::inst::Inst;
use crate::parser::inst::*;
use crate::parser::lexer::Lexer;

use crate::parser::register::Register;
use crate::parser::token::{LineDisplay, Range, Token, WithToken};

use std::convert::TryFrom;
use std::fmt::{self, Display};
use std::hash::{Hash, Hasher};
use std::iter::Peekable;
use std::rc::Rc;
use std::str::FromStr;
use uuid::Uuid;

// TODO make a test case with every supported RARS instruction
use super::inst::InstType;
use super::token::TokenInfo;

// Since we use equality as a way to compare uuids of nodes, this trait is a
// way to check that the contents of an ast node are equal. This is used in
// testing, mostly.
pub trait NodeData {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LabelString(pub String);

impl FromStr for LabelString {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // ensure labelstring cannot be a register
        if Register::from_str(s).is_ok() {
            return Err(());
        }

        // ensure string only starts with a letter
        if !s.chars().next().ok_or(())?.is_alphabetic() {
            return Err(());
        }

        // ensure string only contains safe characters (including numbers)
        if !s
            .chars()
            .all(|c| c.is_digit(10) || c.is_alphabetic() || c == '_' || c == '.' || c == '$')
        {
            return Err(());
        }
        Ok(LabelString(s.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct Label {
    pub name: WithToken<LabelString>,
    pub key: Uuid,
}
#[derive(Debug, Clone)]
pub struct JumpLink {
    pub inst: WithToken<JumpLinkType>,
    pub rd: WithToken<Register>,
    pub name: WithToken<LabelString>,
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
    pub name: WithToken<LabelString>,
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
    pub csr: WithToken<CSRImm>,
    pub rs1: WithToken<Register>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct CSRI {
    pub inst: WithToken<CSRIType>,
    pub rd: WithToken<Register>,
    pub csr: WithToken<CSRImm>,
    pub imm: WithToken<Imm>,
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
    pub name: WithToken<LabelString>,
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
pub struct FuncEntry {
    pub name: WithToken<LabelString>,
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub enum ASTNode {
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
    CSR(CSR),
    CSRImm(CSRI),
}

#[derive(Debug, Clone)]
pub struct EqNodeWrapper(pub ASTNode);

// TODO switch to typesafe representations
impl PartialEq for EqNodeWrapper {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (ASTNode::FuncEntry(a), ASTNode::FuncEntry(b)) => a.name == b.name,
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
            ASTNode::CSRImm(a) => a.key,
            ASTNode::LoadAddr(a) => a.key,
            ASTNode::FuncEntry(a) => a.key,
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

impl PartialEq for ASTNode {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}
impl Eq for ASTNode {}
impl Hash for ASTNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_id().hash(state);
    }
}

impl ASTNode {
    // TODO derive AST new funcs using procedural macros

    pub fn inst(&self) -> WithToken<Inst> {
        let token = match self {
            ASTNode::Arith(x) => x.inst.token.clone(),
            ASTNode::IArith(x) => x.inst.token.clone(),
            ASTNode::UpperArith(x) => x.inst.token.clone(),
            ASTNode::Label(x) => x.name.token.clone(),
            ASTNode::JumpLink(x) => x.inst.token.clone(),
            ASTNode::JumpLinkR(x) => x.inst.token.clone(),
            ASTNode::Basic(x) => x.inst.token.clone(),
            ASTNode::Directive(x) => x.dir.token.clone(),
            ASTNode::Branch(x) => x.inst.token.clone(),
            ASTNode::Store(x) => x.inst.token.clone(),
            ASTNode::Load(x) => x.inst.token.clone(),
            ASTNode::CSR(x) => x.inst.token.clone(),
            ASTNode::CSRImm(x) => x.inst.token.clone(),
            ASTNode::LoadAddr(x) => x.inst.token.clone(),
            ASTNode::FuncEntry(x) => x.name.token.clone(),
        };
        let inst: Inst = match self {
            ASTNode::Arith(x) => (&x.inst.data).into(),
            ASTNode::IArith(x) => (&x.inst.data).into(),
            ASTNode::UpperArith(x) => (&x.inst.data).into(),
            ASTNode::Label(_) => Inst::Nop,
            ASTNode::JumpLink(x) => (&x.inst.data).into(),
            ASTNode::JumpLinkR(x) => (&x.inst.data).into(),
            ASTNode::Basic(x) => (&x.inst.data).into(),
            ASTNode::Directive(x) => Inst::Nop,
            ASTNode::Branch(x) => (&x.inst.data).into(),
            ASTNode::Store(x) => (&x.inst.data).into(),
            ASTNode::Load(x) => (&x.inst.data).into(),
            ASTNode::CSR(x) => (&x.inst.data).into(),
            ASTNode::CSRImm(x) => (&x.inst.data).into(),
            ASTNode::LoadAddr(x) => Inst::La,
            ASTNode::FuncEntry(_) => Inst::Nop,
        };
        let pos = match self {
            ASTNode::Arith(x) => x.inst.pos.clone(),
            ASTNode::IArith(x) => x.inst.pos.clone(),
            ASTNode::UpperArith(x) => x.inst.pos.clone(),
            ASTNode::Label(x) => x.name.pos.clone(),
            ASTNode::JumpLink(x) => x.inst.pos.clone(),
            ASTNode::JumpLinkR(x) => x.inst.pos.clone(),
            ASTNode::Basic(x) => x.inst.pos.clone(),
            ASTNode::Directive(x) => x.dir.pos.clone(),
            ASTNode::Branch(x) => x.inst.pos.clone(),
            ASTNode::Store(x) => x.inst.pos.clone(),
            ASTNode::Load(x) => x.inst.pos.clone(),
            ASTNode::CSR(x) => x.inst.pos.clone(),
            ASTNode::CSRImm(x) => x.inst.pos.clone(),
            ASTNode::LoadAddr(x) => x.inst.pos.clone(),
            ASTNode::FuncEntry(x) => x.name.pos.clone(),
        };
        WithToken {
            token,
            data: inst,
            pos,
        }
    }

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
        name: WithToken<LabelString>,
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
        name: WithToken<LabelString>,
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
        csr: WithToken<CSRImm>,
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

    pub fn new_func_entry(name: WithToken<LabelString>) -> ASTNode {
        ASTNode::FuncEntry(FuncEntry {
            name,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_csri(
        inst: WithToken<CSRIType>,
        rd: WithToken<Register>,
        csr: WithToken<CSRImm>,
        imm: WithToken<Imm>,
    ) -> ASTNode {
        ASTNode::CSRImm(CSRI {
            inst,
            rd,
            imm,
            csr,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_label(name: WithToken<LabelString>) -> ASTNode {
        ASTNode::Label(Label {
            name,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_load_addr(
        inst: WithToken<PseudoType>,
        rd: WithToken<Register>,
        name: WithToken<LabelString>,
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

    pub fn is_func_start(&self) -> bool {
        match self {
            ASTNode::FuncEntry(_) => true,
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

    pub fn is_call(&self) -> bool {
        match self {
            ASTNode::JumpLink(_) => true,
            ASTNode::Basic(x) => x.inst == BasicType::Ecall,
            _ => false,
        }
    }

    // right now only checks if this is specific return statement
    pub fn is_return(&self) -> bool {
        match self {
            ASTNode::JumpLinkR(x) => {
                x.inst == JumpLinkRType::Jalr
                    && x.rd == Register::X0
                    && x.rs1 == Register::X1
                    && x.imm == Imm(0)
            }
            _ => false,
        }
    }

    // checks if a node jumps to another INTERNAL node
    pub fn jumps_to(&self) -> Option<WithToken<LabelString>> {
        match self {
            ASTNode::Branch(x) => Some(x.name.to_owned()),
            _ => None,
        }
    }

    pub fn calls_to(&self) -> Option<WithToken<LabelString>> {
        match self {
            ASTNode::JumpLink(x) => Some(x.name.to_owned()),
            _ => None,
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
impl Display for ASTNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match &self {
            ASTNode::FuncEntry(x) => {
                let name = x.name.data.0.to_string();
                format!("FUNC ENTRY: {}", name)
            }
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
            ASTNode::Label(x) => format!("---[{}]---", x.name.data.0),
            ASTNode::JumpLink(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let name = x.name.data.0.to_string();
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
                let name = x.name.data.0.to_string();
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
                let name = x.name.data.0.to_string();
                format!("{} {} <- [{}]", inst, rd, name)
            }
            ASTNode::CSR(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rd = x.rd.data.to_string();
                let csr = x.csr.data.0.to_string();
                let rs1 = x.rs1.data.to_string();
                format!("{} {} <- {} <- {}", inst, rd, csr, rs1)
            }
            ASTNode::CSRImm(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rd = x.rd.data.to_string();
                let csr = x.csr.data.0.to_string();
                let imm = x.imm.data.0.to_string();
                format!("{} {} <- {} <- {}", inst, rd, csr, imm)
            }
        };
        write!(f, "{}", res)
    }
}

impl<'a> fmt::Display for VecASTDisplayWrapper<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut last = false;
        for node in self.0 {
            if last {
                write!(f, "\n")?;
            }
            write!(f, "{}", node)?;
            last = true;
        }
        Ok(())
    }
}

// TODO this might differ based on how the nodes are made
impl LineDisplay for ASTNode {
    fn get_range(&self) -> Range {
        match &self {
            ASTNode::FuncEntry(x) => x.name.pos.clone(),
            ASTNode::UpperArith(x) => {
                let mut range = x.inst.pos.clone();
                range.end = x.imm.pos.end.clone();
                range
            }
            ASTNode::Label(x) => x.name.pos.clone(),
            ASTNode::Arith(arith) => {
                let mut range = arith.inst.pos.clone();
                range.end = arith.rs2.pos.end.clone();
                range
            }
            ASTNode::IArith(iarith) => {
                let mut range = iarith.inst.pos.clone();
                range.end = iarith.imm.pos.end.clone();
                range
            }
            ASTNode::JumpLink(jl) => {
                let mut range = jl.inst.pos.clone();
                range.end = jl.name.pos.end.clone();
                range
            }
            ASTNode::JumpLinkR(jlr) => {
                let mut range = jlr.inst.pos.clone();
                range.end = jlr.inst.pos.end.clone();
                range
            }
            ASTNode::Branch(branch) => {
                let mut range = branch.inst.pos.clone();
                range.end = branch.name.pos.end.clone();
                range
            }
            ASTNode::Store(store) => {
                let mut range = store.inst.pos.clone();
                range.end = store.imm.pos.end.clone();
                range
            }
            ASTNode::Load(load) => {
                let mut range = load.inst.pos.clone();
                range.end = load.imm.pos.end.clone();
                range
            }
            ASTNode::CSR(csr) => {
                let mut range = csr.inst.pos.clone();
                range.end = csr.rs1.pos.end.clone();
                range
            }
            ASTNode::CSRImm(csr) => {
                let mut range = csr.inst.pos.clone();
                range.end = csr.imm.pos.end.clone();
                range
            }
            ASTNode::Basic(x) => x.inst.pos.clone(),
            ASTNode::LoadAddr(x) => {
                let mut range = x.inst.pos.clone();
                range.end = x.name.pos.end.clone();
                range
            }
            ASTNode::Directive(directive) => directive.dir.pos.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExpectedType {
    Register,
    Imm,
    Label,
    LParen,
    RParen,
    CSRImm,
}

#[derive(Debug, Clone)]
pub enum ParseError {
    Expected(Vec<ExpectedType>, TokenInfo),
    IsNewline(TokenInfo),
    Ignored(TokenInfo),
    UnexpectedToken(TokenInfo),
    UnexpectedEOF,
    NeedTwoNodes(ASTNode, ASTNode),
}

// TODO add parse error for lw (where two nodes are needed)

impl TryFrom<TokenInfo> for LabelString {
    type Error = ();

    fn try_from(value: TokenInfo) -> Result<Self, Self::Error> {
        match value.token {
            Token::Symbol(s) => LabelString::try_from(s),
            _ => Err(()),
        }
    }
}

impl TryFrom<String> for LabelString {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        LabelString::from_str(&value)
    }
}

fn expect_lparen(value: Option<TokenInfo>) -> Result<(), ParseError> {
    let v = value.ok_or(ParseError::UnexpectedEOF)?;
    match v.token {
        Token::LParen => Ok(()),
        _ => Err(ParseError::Expected(vec![ExpectedType::LParen], v)),
    }
}

fn expect_rparen(value: Option<TokenInfo>) -> Result<(), ParseError> {
    let v = value.ok_or(ParseError::UnexpectedEOF)?;
    match v.token {
        Token::RParen => Ok(()),
        _ => Err(ParseError::Expected(vec![ExpectedType::RParen], v)),
    }
}

fn get_reg(value: Option<TokenInfo>) -> Result<WithToken<Register>, ParseError> {
    let v = value.ok_or(ParseError::UnexpectedEOF)?;
    WithToken::<Register>::try_from(v.clone())
        .map_err(|_| ParseError::Expected(vec![ExpectedType::Register], v))
}

fn get_imm(value: Option<TokenInfo>) -> Result<WithToken<Imm>, ParseError> {
    let v = value.ok_or(ParseError::UnexpectedEOF)?;
    WithToken::<Imm>::try_from(v.clone())
        .map_err(|_| ParseError::Expected(vec![ExpectedType::Imm], v))
}

fn get_label(value: Option<TokenInfo>) -> Result<WithToken<LabelString>, ParseError> {
    let v = value.ok_or(ParseError::UnexpectedEOF)?;
    WithToken::<LabelString>::try_from(v.clone())
        .map_err(|_| ParseError::Expected(vec![ExpectedType::Label], v))
}

fn get_csrimm(value: Option<TokenInfo>) -> Result<WithToken<CSRImm>, ParseError> {
    let v = value.ok_or(ParseError::UnexpectedEOF)?;
    WithToken::<CSRImm>::try_from(v.clone())
        .map_err(|_| ParseError::Expected(vec![ExpectedType::CSRImm], v))
}

impl TryFrom<&mut Peekable<Lexer>> for ASTNode {
    // TODO errors are not robust
    // TODO ensure that symbol is not a register
    // TODO how is error handling handled for non registers
    // TODO .clone() is probably not what we want
    type Error = ParseError;

    fn try_from(value: &mut Peekable<Lexer>) -> Result<Self, Self::Error> {
        use ParseError::*;
        let next_node = value.next().ok_or(UnexpectedEOF)?;
        match &next_node.token {
            Token::Symbol(s) => {
                // TODO implement loads with % syntax
                if let Ok(inst) = Inst::from_str(s) {
                    let node = match InstType::from(&inst) {
                        InstType::CSRIType(inst) => {
                            let rd = get_reg(value.next())?;
                            let csr = get_csrimm(value.next())?;
                            let imm = get_imm(value.next())?;
                            Ok(ASTNode::new_csri(
                                WithToken::new(inst, next_node),
                                rd,
                                csr,
                                imm,
                            ))
                        }
                        InstType::CSRType(inst) => {
                            let rd = get_reg(value.next())?;
                            let csr = get_csrimm(value.next())?;
                            let rs1 = get_reg(value.next())?;
                            Ok(ASTNode::new_csr(
                                WithToken::new(inst, next_node),
                                rd,
                                csr,
                                rs1,
                            ))
                        }
                        InstType::UpperArithType(inst) => {
                            let rd = get_reg(value.next())?;
                            let imm = get_imm(value.next())?;
                            Ok(ASTNode::new_upper_arith(
                                WithToken::new(inst, next_node),
                                rd,
                                imm,
                            ))
                        }
                        InstType::ArithType(inst) => {
                            let rd = get_reg(value.next())?;
                            let rs1 = get_reg(value.next())?;
                            let rs2 = get_reg(value.next())?;
                            Ok(ASTNode::new_arith(
                                WithToken::new(inst, next_node),
                                rd,
                                rs1,
                                rs2,
                            ))
                        }
                        InstType::IArithType(inst) => {
                            let rd = get_reg(value.next())?;
                            let rs1 = get_reg(value.next())?;
                            let imm = get_imm(value.next())?;
                            Ok(ASTNode::new_iarith(
                                WithToken::new(inst, next_node),
                                rd,
                                rs1,
                                imm,
                            ))
                        }

                        InstType::JumpLinkType(inst) => {
                            let name_token = value.next();

                            return if let Ok(reg) = get_reg(name_token.clone()) {
                                let name = get_label(value.next())?;
                                Ok(ASTNode::new_jump_link(
                                    WithToken::new(inst, next_node),
                                    reg,
                                    name,
                                ))
                            } else if let Ok(name) = get_label(name_token.clone()) {
                                Ok(ASTNode::new_jump_link(
                                    WithToken::new(inst, next_node.clone()),
                                    WithToken::new(Register::X1, next_node),
                                    name,
                                ))
                            } else {
                                Err(Expected(
                                    vec![ExpectedType::Register, ExpectedType::Label],
                                    name_token.ok_or(UnexpectedEOF)?,
                                ))
                            };
                        }
                        InstType::JumpLinkRType(inst) => {
                            let reg1 = get_reg(value.next())?;
                            let next = value.next();
                            return if let Ok(rs1) = get_reg(next.clone()) {
                                let imm = get_imm(value.next())?;
                                Ok(ASTNode::new_jump_link_r(
                                    WithToken::new(inst, next_node),
                                    reg1,
                                    rs1,
                                    imm,
                                ))
                            } else if let Ok(imm) = get_imm(next.clone()) {
                                if let Ok(()) = expect_lparen(value.peek().cloned()) {
                                    value.next();
                                    let rs1 = get_reg(value.next())?;
                                    expect_rparen(value.next())?;
                                    Ok(ASTNode::new_jump_link_r(
                                        WithToken::new(inst, next_node),
                                        reg1,
                                        rs1,
                                        imm,
                                    ))
                                } else {
                                    Ok(ASTNode::new_jump_link_r(
                                        WithToken::new(inst, next_node.clone()),
                                        WithToken::new(Register::X1, next_node),
                                        reg1,
                                        imm,
                                    ))
                                }
                            } else if let Ok(()) = expect_lparen(next) {
                                let rs1 = get_reg(value.next())?;
                                expect_rparen(value.next())?;
                                Ok(ASTNode::new_jump_link_r(
                                    WithToken::new(inst, next_node.clone()),
                                    reg1,
                                    rs1,
                                    WithToken::new(Imm(0), next_node),
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
                            let rd = get_reg(value.next())?;
                            let next = value.next();
                            return if let Ok(imm) = get_imm(next.clone()) {
                                if let Ok(()) = expect_lparen(value.peek().cloned()) {
                                    value.next();
                                    let rs1 = get_reg(value.next())?;
                                    expect_rparen(value.next())?;
                                    Ok(ASTNode::new_load(
                                        WithToken::new(inst, next_node),
                                        rd,
                                        rs1,
                                        imm,
                                    ))
                                } else {
                                    Ok(ASTNode::new_load(
                                        WithToken::new(inst, next_node.clone()),
                                        rd,
                                        WithToken::new(Register::X0, next_node),
                                        imm,
                                    ))
                                }
                            } else if let Ok(label) = get_label(next.clone()) {
                                Err(NeedTwoNodes(
                                    ASTNode::new_load_addr(
                                        WithToken::new(PseudoType::La, next_node.clone()),
                                        rd.clone(),
                                        label,
                                    ),
                                    ASTNode::new_load(
                                        WithToken::new(inst, next_node.clone()),
                                        rd.clone(),
                                        rd,
                                        WithToken::new(Imm(0), next_node),
                                    ),
                                ))
                            } else if let Ok(()) = expect_lparen(next.clone()) {
                                let rs1 = get_reg(value.next())?;
                                expect_rparen(value.next())?;
                                Ok(ASTNode::new_load(
                                    WithToken::new(inst, next_node.clone()),
                                    rd,
                                    rs1,
                                    WithToken::new(Imm(0), next_node),
                                ))
                            } else {
                                Err(Expected(
                                    vec![
                                        ExpectedType::Label,
                                        ExpectedType::Imm,
                                        ExpectedType::LParen,
                                    ],
                                    next.ok_or(UnexpectedEOF)?,
                                ))
                            };
                        }
                        InstType::StoreType(inst) => {
                            let rs2 = get_reg(value.next())?;
                            let next = value.next();

                            return if let Ok(imm) = get_imm(next.clone()) {
                                if let Ok(()) = expect_lparen(value.peek().cloned()) {
                                    value.next();
                                    let rs1 = get_reg(value.next())?;
                                    expect_rparen(value.next())?;
                                    Ok(ASTNode::new_store(
                                        WithToken::new(inst, next_node),
                                        rs2,
                                        rs1,
                                        imm,
                                    ))
                                } else {
                                    Ok(ASTNode::new_store(
                                        WithToken::new(inst, next_node.clone()),
                                        rs2,
                                        WithToken::new(Register::X0, next_node),
                                        imm,
                                    ))
                                }
                            } else if let Ok(label) = get_label(next.clone()) {
                                let temp_reg = get_reg(value.next())?;
                                Err(NeedTwoNodes(
                                    ASTNode::new_load_addr(
                                        WithToken::new(PseudoType::La, next_node.clone()),
                                        temp_reg.clone(),
                                        label,
                                    ),
                                    ASTNode::new_store(
                                        WithToken::new(inst, next_node.clone()),
                                        rs2,
                                        temp_reg.clone(),
                                        WithToken::new(Imm(0), next_node),
                                    ),
                                ))
                            } else if let Ok(()) = expect_lparen(next.clone()) {
                                let rs1 = get_reg(value.next())?;
                                expect_rparen(value.next())?;
                                Ok(ASTNode::new_store(
                                    WithToken::new(inst, next_node.clone()),
                                    rs2,
                                    rs1,
                                    WithToken::new(Imm(0), next_node),
                                ))
                            } else {
                                Err(Expected(
                                    vec![
                                        ExpectedType::Label,
                                        ExpectedType::Imm,
                                        ExpectedType::LParen,
                                    ],
                                    next.ok_or(UnexpectedEOF)?,
                                ))
                            };
                        }
                        InstType::BranchType(inst) => {
                            let rs1 = get_reg(value.next())?;
                            let rs2 = get_reg(value.next())?;
                            let label = get_label(value.next())?;
                            Ok(ASTNode::new_branch(
                                WithToken::new(inst, next_node),
                                rs1,
                                rs2,
                                label,
                            ))
                        }
                        InstType::IgnoreType(_) => Err(Ignored(next_node)),
                        InstType::BasicType(inst) => {
                            Ok(ASTNode::new_basic(WithToken::new(inst, next_node)))
                        }
                        InstType::PseudoType(inst) => {
                            // TODO not every pseudo instruction from rars is covered
                            // here.
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
                                    let rd = get_reg(value.next())?;
                                    let rs1 = get_reg(value.next())?;
                                    return Ok(ASTNode::new_arith(
                                        WithToken::new(ArithType::Add, next_node.clone()),
                                        rd,
                                        rs1,
                                        WithToken::new(Register::X0, next_node.clone()),
                                    ));
                                }
                                PseudoType::Li => {
                                    let rd = get_reg(value.next())?;
                                    let imm = get_imm(value.next())?;
                                    return Ok(ASTNode::new_iarith(
                                        WithToken::new(IArithType::Addi, next_node.clone()),
                                        rd,
                                        WithToken::new(Register::X0, imm.clone().into()),
                                        imm,
                                    ));
                                }
                                PseudoType::La => {
                                    let rd = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ASTNode::new_load_addr(
                                        WithToken::new(PseudoType::La, next_node.clone()),
                                        rd,
                                        label,
                                    ));
                                }
                                PseudoType::J | PseudoType::B => {
                                    let label = get_label(value.next())?;
                                    return Ok(ASTNode::new_jump_link(
                                        WithToken::new(JumpLinkType::Jal, next_node.clone()),
                                        WithToken::new(Register::X0, next_node.clone()),
                                        label,
                                    ));
                                }
                                PseudoType::Jr => {
                                    let rs1 = get_reg(value.next())?;
                                    return Ok(ASTNode::new_jump_link_r(
                                        WithToken::new(JumpLinkRType::Jalr, next_node.clone()),
                                        WithToken::new(Register::X0, next_node.clone()),
                                        rs1,
                                        WithToken::new(Imm(0), next_node.clone()),
                                    ));
                                }
                                PseudoType::Beqz => {
                                    let rs1 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ASTNode::new_branch(
                                        WithToken::new(BranchType::Beq, next_node.clone()),
                                        rs1,
                                        WithToken::new(Register::X0, next_node.clone()),
                                        label,
                                    ));
                                }
                                PseudoType::Bnez => {
                                    let rs1 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ASTNode::new_branch(
                                        WithToken::new(BranchType::Bne, next_node.clone()),
                                        rs1,
                                        WithToken::new(Register::X0, next_node.clone()),
                                        label,
                                    ));
                                }
                                PseudoType::Bltz => {
                                    let rs1 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ASTNode::new_branch(
                                        WithToken::new(BranchType::Blt, next_node.clone()),
                                        rs1,
                                        WithToken::new(Register::X0, next_node.clone()),
                                        label,
                                    ));
                                }
                                PseudoType::Neg => {
                                    let rd = get_reg(value.next())?;
                                    let rs1 = get_reg(value.next())?;
                                    return Ok(ASTNode::new_arith(
                                        WithToken::new(ArithType::Sub, next_node.clone()),
                                        rd,
                                        WithToken::new(Register::X0, next_node.clone()),
                                        rs1,
                                    ));
                                }
                                PseudoType::Not => {
                                    let rd = get_reg(value.next())?;
                                    let rs1 = get_reg(value.next())?;
                                    return Ok(ASTNode::new_iarith(
                                        WithToken::new(IArithType::Xori, next_node.clone()),
                                        rd,
                                        rs1,
                                        WithToken::new(Imm(-1), next_node.clone()),
                                    ));
                                }
                                PseudoType::Seqz => {
                                    let rd = get_reg(value.next())?;
                                    let rs1 = get_reg(value.next())?;
                                    return Ok(ASTNode::new_iarith(
                                        WithToken::new(IArithType::Sltiu, next_node.clone()),
                                        rd,
                                        rs1,
                                        WithToken::new(Imm(1), next_node.clone()),
                                    ));
                                }
                                PseudoType::Snez => {
                                    let rd = get_reg(value.next())?;
                                    let rs1 = get_reg(value.next())?;
                                    return Ok(ASTNode::new_iarith(
                                        WithToken::new(IArithType::Sltiu, next_node.clone()),
                                        rd,
                                        rs1,
                                        WithToken::new(Imm(0), next_node.clone()),
                                    ));
                                }
                                PseudoType::Nop => {
                                    return Ok(ASTNode::new_iarith(
                                        WithToken::new(IArithType::Addi, next_node.clone()),
                                        WithToken::new(Register::X0, next_node.clone()),
                                        WithToken::new(Register::X0, next_node.clone()),
                                        WithToken::new(Imm(0), next_node.clone()),
                                    ));
                                }
                                PseudoType::Bgez => {
                                    let rs1 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ASTNode::new_branch(
                                        WithToken::new(BranchType::Bge, next_node.clone()),
                                        rs1,
                                        WithToken::new(Register::X0, next_node.clone()),
                                        label,
                                    ));
                                }
                                PseudoType::Sgtz => {
                                    let rd = get_reg(value.next())?;
                                    let rs1 = get_reg(value.next())?;
                                    return Ok(ASTNode::new_arith(
                                        WithToken::new(ArithType::Slt, next_node.clone()),
                                        rd,
                                        WithToken::new(Register::X0, next_node.clone()),
                                        rs1,
                                    ));
                                }
                                PseudoType::Sltz => {
                                    let rd = get_reg(value.next())?;
                                    let rs1 = get_reg(value.next())?;
                                    return Ok(ASTNode::new_arith(
                                        WithToken::new(ArithType::Slt, next_node.clone()),
                                        rd,
                                        rs1,
                                        WithToken::new(Register::X0, next_node.clone()),
                                    ));
                                }
                                PseudoType::Sgez => {
                                    let rs1 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ASTNode::new_branch(
                                        WithToken::new(BranchType::Bge, next_node.clone()),
                                        WithToken::new(Register::X0, next_node.clone()),
                                        rs1,
                                        label,
                                    ));
                                }
                                PseudoType::Call => {
                                    let label = get_label(value.next())?;
                                    return Ok(ASTNode::new_jump_link(
                                        WithToken::new(JumpLinkType::Jal, next_node.clone()),
                                        WithToken::new(Register::X1, next_node.clone()),
                                        label,
                                    ));
                                }
                                PseudoType::Bgt => {
                                    let rs1 = get_reg(value.next())?;
                                    let rs2 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ASTNode::new_branch(
                                        WithToken::new(BranchType::Blt, next_node.clone()),
                                        rs2,
                                        rs1,
                                        label,
                                    ));
                                }
                                PseudoType::Ble => {
                                    let rs1 = get_reg(value.next())?;
                                    let rs2 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ASTNode::new_branch(
                                        WithToken::new(BranchType::Bge, next_node.clone()),
                                        rs2,
                                        rs1,
                                        label,
                                    ));
                                }
                                PseudoType::Bgtu => {
                                    let rs1 = get_reg(value.next())?;
                                    let rs2 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ASTNode::new_branch(
                                        WithToken::new(BranchType::Bltu, next_node.clone()),
                                        rs2,
                                        rs1,
                                        label,
                                    ));
                                }
                                PseudoType::Bleu => {
                                    let rs1 = get_reg(value.next())?;
                                    let rs2 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ASTNode::new_branch(
                                        WithToken::new(BranchType::Bgeu, next_node.clone()),
                                        rs2,
                                        rs1,
                                        label,
                                    ));
                                }
                                PseudoType::Bgtz => {
                                    let rs1 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ASTNode::new_branch(
                                        WithToken::new(BranchType::Blt, next_node.clone()),
                                        rs1,
                                        WithToken::new(Register::X0, next_node.clone()),
                                        label,
                                    ));
                                }
                                PseudoType::Blez => {
                                    let rs1 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ASTNode::new_branch(
                                        WithToken::new(BranchType::Bge, next_node.clone()),
                                        rs1,
                                        WithToken::new(Register::X0, next_node.clone()),
                                        label,
                                    ));
                                }
                                PseudoType::Csrci | PseudoType::Csrsi | PseudoType::Csrwi => {
                                    let csr = get_csrimm(value.next())?;
                                    let imm = get_imm(value.next())?;
                                    let inst = match inst {
                                        PseudoType::Csrci => CSRIType::Csrrci,
                                        PseudoType::Csrsi => CSRIType::Csrrsi,
                                        PseudoType::Csrwi => CSRIType::Csrrwi,
                                        _ => unreachable!(),
                                    };
                                    return Ok(ASTNode::new_csri(
                                        WithToken::new(inst, next_node.clone()),
                                        WithToken::new(Register::X0, next_node.clone()),
                                        csr,
                                        imm,
                                    ));
                                }
                                PseudoType::Csrc | PseudoType::Csrs | PseudoType::Csrw => {
                                    let rs1 = get_reg(value.next())?;
                                    let csr = get_csrimm(value.next())?;
                                    let inst = match inst {
                                        PseudoType::Csrc => CSRType::Csrrc,
                                        PseudoType::Csrs => CSRType::Csrrs,
                                        PseudoType::Csrw => CSRType::Csrrw,
                                        _ => unreachable!(),
                                    };
                                    return Ok(ASTNode::new_csr(
                                        WithToken::new(inst, next_node.clone()),
                                        WithToken::new(Register::X0, next_node.clone()),
                                        csr,
                                        rs1,
                                    ));
                                }
                                PseudoType::Csrr => {
                                    let rd = get_reg(value.next())?;
                                    let csr = get_csrimm(value.next())?;
                                    return Ok(ASTNode::new_csr(
                                        WithToken::new(CSRType::Csrrs, next_node.clone()),
                                        rd,
                                        csr,
                                        WithToken::new(Register::X0, next_node.clone()),
                                    ));
                                }
                            }
                        }
                    };
                    return node;
                }
                Err(UnexpectedToken(next_node))
            }
            Token::Label(s) => Ok(ASTNode::new_label(WithToken::new(
                LabelString::from_str(s).map_err(|_| {
                    ParseError::Expected(vec![ExpectedType::Label], next_node.clone())
                })?,
                next_node,
            ))),
            Token::Directive(_) => Err(Ignored(next_node)),
            Token::Newline => Err(IsNewline(next_node)),
            _ => unimplemented!(),
        }
    }
}
