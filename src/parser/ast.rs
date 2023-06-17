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

impl Display for LabelString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
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
            ASTNode::Directive(_x) => Inst::Nop,
            ASTNode::Branch(x) => (&x.inst.data).into(),
            ASTNode::Store(x) => (&x.inst.data).into(),
            ASTNode::Load(x) => (&x.inst.data).into(),
            ASTNode::CSR(x) => (&x.inst.data).into(),
            ASTNode::CSRImm(x) => (&x.inst.data).into(),
            ASTNode::LoadAddr(_x) => Inst::La,
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

    pub fn call_name(&self) -> Option<WithToken<LabelString>> {
        match self {
            ASTNode::JumpLink(x) => Some(x.name.to_owned()),
            _ => None,
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
