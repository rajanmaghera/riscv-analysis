use crate::parser::imm::{CSRImm, Imm};
use crate::parser::inst::Inst;
use crate::parser::inst::{
    ArithType, BasicType, BranchType, CSRIType, CSRType, IArithType, IgnoreType, JumpLinkRType,
    JumpLinkType, LoadType, PseudoType, StoreType, UpperArithType,
};

use crate::parser::register::Register;
use crate::parser::token::{LineDisplay, Range, Token, With};

use std::collections::HashSet;

use std::fmt::{self, Display};
use std::hash::{Hash, Hasher};

use std::rc::Rc;
use std::str::FromStr;
use uuid::Uuid;

// TODO make a test case with every supported RARS instruction
// TODO set all returns to a jump to a special label

use super::token::Position;

// Since we use equality as a way to compare uuids of nodes, this trait is a
// way to check that the contents of an ast node are equal. This is used in
// testing, mostly.
pub trait NodeData {
    fn get_id(&self) -> Uuid;
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectiveType {
    Nop, // Include(WithToken<String>),
         // Align(WithToken<i32>),
         // Space(WithToken<i32>),
         // Text,
         // Data, // TODO include more
}

impl Display for DirectiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LabelString(pub String);

impl PartialEq<str> for LabelString {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for LabelString {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

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
            .all(|c| c.is_ascii_digit() || c.is_alphabetic() || c == '_' || c == '.' || c == '$')
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

#[derive(Debug, Clone)]
pub struct Directive {
    pub dir: With<DirectiveType>,
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
// TODO add names here?
pub struct FuncEntry {
    pub key: Uuid,
}

#[derive(Debug, Clone)]
pub struct ProgramEntry {
    pub key: Uuid,
}

// TODO change optional fields to Option<T> instead of T
// then, implement TokenInfo for the options on a case by case basis

// TODO add FuncExit/ProgramExit nodes?
// Rename to ParseNode
#[derive(Debug, Clone)]
pub enum Node {
    ProgramEntry(ProgramEntry),
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
    Csr(Csr),
    CsrI(CsrI),
}

#[derive(Debug, Clone)]
pub struct EqNodeWrapper(pub Node);

// TODO switch to typesafe representations
impl PartialEq for EqNodeWrapper {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Node::FuncEntry(a), Node::FuncEntry(b)) => true,
            (Node::ProgramEntry(_a), Node::ProgramEntry(_b)) => true,
            (Node::Arith(a), Node::Arith(b)) => {
                a.inst == b.inst && a.rd == b.rd && a.rs1 == b.rs1 && a.rs2 == b.rs2
            }
            (Node::IArith(a), Node::IArith(b)) => {
                a.inst == b.inst && a.rd == b.rd && a.rs1 == b.rs1 && a.imm == b.imm
            }
            (Node::Label(a), Node::Label(b)) => a.name == b.name,
            (Node::JumpLink(a), Node::JumpLink(b)) => a.inst == b.inst && a.name == b.name,
            (Node::JumpLinkR(a), Node::JumpLinkR(b)) => {
                a.inst == b.inst && a.rd == b.rd && a.rs1 == b.rs1 && a.imm == b.imm
            }
            (Node::Basic(a), Node::Basic(b)) => a.inst == b.inst,
            (Node::Directive(a), Node::Directive(b)) => a.dir == b.dir,
            (Node::Branch(a), Node::Branch(b)) => {
                a.inst == b.inst && a.rs1 == b.rs1 && a.rs2 == b.rs2 && a.name == b.name
            }
            (Node::Store(a), Node::Store(b)) => {
                a.inst == b.inst && a.rs1 == b.rs1 && a.rs2 == b.rs2 && a.imm == b.imm
            }
            (Node::Load(a), Node::Load(b)) => {
                a.inst == b.inst && a.rd == b.rd && a.rs1 == b.rs1 && a.imm == b.imm
            }
            (Node::Csr(a), Node::Csr(b)) => {
                a.inst == b.inst && a.rd == b.rd && a.csr == b.csr && a.rs1 == b.rs1
            }
            (Node::LoadAddr(a), Node::LoadAddr(b)) => {
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

impl EqNodeData for Node {
    fn data(&self) -> EqNodeWrapper {
        EqNodeWrapper(self.clone())
    }
}

impl EqNodeDataVec for Vec<Node> {
    fn data(&self) -> Vec<EqNodeWrapper> {
        self.iter()
            .map(crate::parser::ast::EqNodeData::data)
            .collect()
    }
}

impl EqNodeDataVec for Vec<Rc<Node>> {
    fn data(&self) -> Vec<EqNodeWrapper> {
        self.iter().map(|x| x.data()).collect()
    }
}

impl NodeData for Node {
    fn get_id(&self) -> Uuid {
        match self {
            Node::Arith(a) => a.key,
            Node::IArith(a) => a.key,
            Node::UpperArith(a) => a.key,
            Node::Label(a) => a.key,
            Node::JumpLink(a) => a.key,
            Node::JumpLinkR(a) => a.key,
            Node::Basic(a) => a.key,
            Node::Directive(a) => a.key,
            Node::Branch(a) => a.key,
            Node::Store(a) => a.key,
            Node::Load(a) => a.key,
            Node::Csr(a) => a.key,
            Node::CsrI(a) => a.key,
            Node::LoadAddr(a) => a.key,
            Node::FuncEntry(a) => a.key,
            Node::ProgramEntry(a) => a.key,
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

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}
impl Eq for Node {}
impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_id().hash(state);
    }
}

impl Node {
    // TODO derive AST new funcs using procedural macros

    pub fn inst(&self) -> With<Inst> {
        let token = match self {
            Node::Arith(x) => x.inst.token.clone(),
            Node::IArith(x) => x.inst.token.clone(),
            Node::UpperArith(x) => x.inst.token.clone(),
            Node::Label(x) => x.name.token.clone(),
            Node::JumpLink(x) => x.inst.token.clone(),
            Node::JumpLinkR(x) => x.inst.token.clone(),
            Node::Basic(x) => x.inst.token.clone(),
            Node::Directive(x) => x.dir.token.clone(),
            Node::Branch(x) => x.inst.token.clone(),
            Node::Store(x) => x.inst.token.clone(),
            Node::Load(x) => x.inst.token.clone(),
            Node::Csr(x) => x.inst.token.clone(),
            Node::CsrI(x) => x.inst.token.clone(),
            Node::LoadAddr(x) => x.inst.token.clone(),
            Node::ProgramEntry(_) | Node::FuncEntry(_) => Token::Symbol(String::new()),
        };
        let inst: Inst = match self {
            Node::Arith(x) => (&x.inst.data).into(),
            Node::IArith(x) => (&x.inst.data).into(),
            Node::UpperArith(x) => (&x.inst.data).into(),
            Node::JumpLink(x) => (&x.inst.data).into(),
            Node::JumpLinkR(x) => (&x.inst.data).into(),
            Node::Basic(x) => (&x.inst.data).into(),
            Node::Branch(x) => (&x.inst.data).into(),
            Node::Store(x) => (&x.inst.data).into(),
            Node::Load(x) => (&x.inst.data).into(),
            Node::Csr(x) => (&x.inst.data).into(),
            Node::CsrI(x) => (&x.inst.data).into(),
            Node::LoadAddr(_) => Inst::La,
            Node::Label(_) | Node::Directive(_) | Node::FuncEntry(_) | Node::ProgramEntry(_) => {
                Inst::Nop
            }
        };
        let pos = match self {
            Node::Arith(x) => x.inst.pos.clone(),
            Node::IArith(x) => x.inst.pos.clone(),
            Node::UpperArith(x) => x.inst.pos.clone(),
            Node::Label(x) => x.name.pos.clone(),
            Node::JumpLink(x) => x.inst.pos.clone(),
            Node::JumpLinkR(x) => x.inst.pos.clone(),
            Node::Basic(x) => x.inst.pos.clone(),
            Node::Directive(x) => x.dir.pos.clone(),
            Node::Branch(x) => x.inst.pos.clone(),
            Node::Store(x) => x.inst.pos.clone(),
            Node::Load(x) => x.inst.pos.clone(),
            Node::Csr(x) => x.inst.pos.clone(),
            Node::CsrI(x) => x.inst.pos.clone(),
            Node::LoadAddr(x) => x.inst.pos.clone(),
            Node::ProgramEntry(_) | Node::FuncEntry(_) => Range {
                // TODO add position to ASTNodes
                start: Position { line: 0, column: 0 },
                end: Position { line: 0, column: 0 },
            },
        };
        With {
            token,
            data: inst,
            pos,
        }
    }

    pub fn new_arith(
        inst: With<ArithType>,
        rd: With<Register>,
        rs1: With<Register>,
        rs2: With<Register>,
    ) -> Node {
        Node::Arith(Arith {
            inst,
            rd,
            rs1,
            rs2,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_iarith(
        inst: With<IArithType>,
        rd: With<Register>,
        rs1: With<Register>,
        imm: With<Imm>,
    ) -> Node {
        Node::IArith(IArith {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_upper_arith(inst: With<UpperArithType>, rd: With<Register>, imm: With<Imm>) -> Node {
        Node::UpperArith(UpperArith {
            inst,
            rd,
            imm,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_jump_link(
        inst: With<JumpLinkType>,
        rd: With<Register>,
        name: With<LabelString>,
    ) -> Node {
        Node::JumpLink(JumpLink {
            inst,
            rd,
            name,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_jump_link_r(
        inst: With<JumpLinkRType>,
        rd: With<Register>,
        rs1: With<Register>,
        imm: With<Imm>,
    ) -> Node {
        Node::JumpLinkR(JumpLinkR {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_basic(inst: With<BasicType>) -> Node {
        Node::Basic(Basic {
            inst,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_directive(dir: With<DirectiveType>) -> Node {
        Node::Directive(Directive {
            dir,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_branch(
        inst: With<BranchType>,
        rs1: With<Register>,
        rs2: With<Register>,
        name: With<LabelString>,
    ) -> Node {
        Node::Branch(Branch {
            inst,
            rs1,
            rs2,
            name,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_store(
        inst: With<StoreType>,
        rs1: With<Register>,
        rs2: With<Register>,
        imm: With<Imm>,
    ) -> Node {
        Node::Store(Store {
            inst,
            rs1,
            rs2,
            imm,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_load(
        inst: With<LoadType>,
        rd: With<Register>,
        rs1: With<Register>,
        imm: With<Imm>,
    ) -> Node {
        Node::Load(Load {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_csr(
        inst: With<CSRType>,
        rd: With<Register>,
        csr: With<CSRImm>,
        rs1: With<Register>,
    ) -> Node {
        Node::Csr(Csr {
            inst,
            rd,
            rs1,
            csr,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_func_entry() -> Node {
        Node::FuncEntry(FuncEntry {
            key: Uuid::new_v4(),
        })
    }

    pub fn new_program_entry() -> Node {
        Node::ProgramEntry(ProgramEntry {
            key: Uuid::new_v4(),
        })
    }

    pub fn new_csri(
        inst: With<CSRIType>,
        rd: With<Register>,
        csr: With<CSRImm>,
        imm: With<Imm>,
    ) -> Node {
        Node::CsrI(CsrI {
            inst,
            rd,
            imm,
            csr,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_label(name: With<LabelString>) -> Node {
        Node::Label(Label {
            name,
            key: Uuid::new_v4(),
        })
    }

    pub fn new_load_addr(
        inst: With<PseudoType>,
        rd: With<Register>,
        name: With<LabelString>,
    ) -> Node {
        Node::LoadAddr(LoadAddr {
            inst,
            rd,
            name,
            key: Uuid::new_v4(),
        })
    }

    // right now only checks if this is specific return statement
    // TODO attach to jumplinkr only?
    pub fn is_return(&self) -> bool {
        match self {
            Node::JumpLinkR(x) => {
                x.inst == JumpLinkRType::Jalr
                    && x.rd == Register::X0
                    && x.rs1 == Register::X1
                    && x.imm == Imm(0)
            }
            _ => false,
        }
    }

    // TODO move into Load/Store ast node info
    pub fn _is_stack_access(&self) -> bool {
        match self {
            Node::Load(x) => x.rs1 == Register::X2,
            Node::Store(x) => x.rs1 == Register::X2,
            _ => false,
        }
    }

    pub fn is_memory_access(&self) -> bool {
        matches!(self, Node::Load(_) | Node::Store(_))
    }

    pub fn calls_to(&self) -> Option<With<LabelString>> {
        match self {
            Node::JumpLink(x) if x.rd == Register::X1 => Some(x.name.clone()),
            _ => None,
        }
    }

    pub fn is_ecall(&self) -> bool {
        match self {
            Node::Basic(x) => x.inst == BasicType::Ecall,
            _ => false,
        }
    }

    pub fn jumps_to(&self) -> Option<With<LabelString>> {
        match self {
            Node::JumpLink(x) if x.rd != Register::X1 => Some(x.name.clone()),
            Node::Branch(x) => Some(x.name.clone()),
            _ => None,
        }
    }

    pub fn is_any_entry(&self) -> bool {
        matches!(self, Node::ProgramEntry(_) | Node::FuncEntry(_))
    }

    pub fn is_function_entry(&self) -> bool {
        matches!(self, Node::FuncEntry(_))
    }

    pub fn is_program_entry(&self) -> bool {
        matches!(self, Node::ProgramEntry(_))
    }

    // NOTE: This is in context to a register store, not a memory store
    pub fn stores_to(&self) -> Option<With<Register>> {
        match self {
            Node::Load(load) => Some(load.rd.clone()),
            Node::LoadAddr(load) => Some(load.rd.clone()),
            Node::Arith(arith) => Some(arith.rd.clone()),
            Node::IArith(iarith) => Some(iarith.rd.clone()),
            Node::UpperArith(upper_arith) => Some(upper_arith.rd.clone()),
            Node::JumpLink(jump_link) => Some(jump_link.rd.clone()),
            Node::JumpLinkR(jump_link_r) => Some(jump_link_r.rd.clone()),
            Node::Csr(csr) => Some(csr.rd.clone()),
            Node::CsrI(csri) => Some(csri.rd.clone()),
            Node::ProgramEntry(_)
            | Node::FuncEntry(_)
            | Node::Label(_)
            | Node::Basic(_)
            | Node::Directive(_)
            | Node::Branch(_)
            | Node::Store(_) => None,
        }
    }

    pub fn reads_from(&self) -> HashSet<With<Register>> {
        let vector = match self {
            Node::Arith(x) => vec![x.rs1.clone(), x.rs2.clone()],
            Node::IArith(x) => vec![x.rs1.clone()],
            Node::JumpLinkR(x) => vec![x.rs1.clone()],
            Node::Branch(x) => vec![x.rs1.clone(), x.rs2.clone()],
            Node::Store(x) => vec![x.rs1.clone(), x.rs2.clone()],
            Node::Load(x) => vec![x.rs1.clone()],
            Node::Csr(x) => vec![x.rs1.clone()],
            Node::ProgramEntry(_)
            | Node::FuncEntry(_)
            | Node::UpperArith(_)
            | Node::Label(_)
            | Node::JumpLink(_)
            | Node::Basic(_)
            | Node::Directive(_)
            | Node::LoadAddr(_)
            | Node::CsrI(_) => vec![],
        };
        vector.into_iter().collect()
    }
}

// -- PRETTY PRINTING --
pub struct VecASTDisplayWrapper<'a>(&'a Vec<Node>);
pub trait ToDisplayForVecASTNode {
    fn to_display(&self) -> VecASTDisplayWrapper;
}
impl ToDisplayForVecASTNode for Vec<Node> {
    fn to_display(&self) -> VecASTDisplayWrapper {
        VecASTDisplayWrapper(self)
    }
}
impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match &self {
            Node::ProgramEntry(_) => "--- [PROGRAM ENTRY] ---".to_string(),
            Node::FuncEntry(x) => {
                format!("--- FUNCTION ENTRY ---")
            }
            Node::UpperArith(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rd = x.rd.data.to_string();
                let imm = x.imm.data.0.to_string();
                format!("{inst} {rd} <- {imm}")
            }
            Node::Arith(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rd = x.rd.data.to_string();
                let rs1 = x.rs1.data.to_string();
                let rs2 = x.rs2.data.to_string();
                format!("{inst} {rd} <- {rs1}, {rs2}")
            }
            Node::IArith(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rd = x.rd.data.to_string();
                let rs1 = x.rs1.data.to_string();
                let imm = x.imm.data.0.to_string();
                format!("{inst} {rd} <- {rs1}, {imm}")
            }
            Node::Label(x) => format!("---[{}]---", x.name.data.0),
            Node::JumpLink(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let name = x.name.data.0.clone();
                let rd = x.rd.data.to_string();
                format!("{inst} [{name}] | {rd} <- PC")
            }
            Node::JumpLinkR(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rs1 = x.rs1.data.to_string();
                format!("{inst} [{rs1}]")
            }
            Node::Basic(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                format!("{inst}")
            }
            Node::Directive(x) => {
                let dir = x.dir.data.to_string();
                format!("-<{dir}>-")
            }
            Node::Branch(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rs1 = x.rs1.data.to_string();
                let rs2 = x.rs2.data.to_string();
                let name = x.name.data.0.clone();
                format!("{inst} {rs1}--{rs2}, [{name}]")
            }
            Node::Store(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rs1 = x.rs1.data.to_string();
                let rs2 = x.rs2.data.to_string();
                let imm = x.imm.data.0.to_string();
                format!("{inst} {rs2} -> {imm}({rs1})")
            }
            Node::Load(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rd = x.rd.data.to_string();
                let rs1 = x.rs1.data.to_string();
                let imm = x.imm.data.0.to_string();
                format!("{inst} {rd} <- {imm}({rs1})")
            }
            // TODO don't use the pseudo type here
            Node::LoadAddr(x) => {
                let inst = "la";
                let rd = x.rd.data.to_string();
                let name = x.name.data.0.clone();
                format!("{inst} {rd} <- [{name}]")
            }
            Node::Csr(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rd = x.rd.data.to_string();
                let csr = x.csr.data.0.to_string();
                let rs1 = x.rs1.data.to_string();
                format!("{inst} {rd} <- {csr} <- {rs1}")
            }
            Node::CsrI(x) => {
                let inst: Inst = Inst::from(&x.inst.data);
                let rd = x.rd.data.to_string();
                let csr = x.csr.data.0.to_string();
                let imm = x.imm.data.0.to_string();
                format!("{inst} {rd} <- {csr} <- {imm}")
            }
        };
        write!(f, "{res}")
    }
}

impl<'a> fmt::Display for VecASTDisplayWrapper<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut last = false;
        for node in self.0 {
            if last {
                writeln!(f)?;
            }
            write!(f, "{node}")?;
            last = true;
        }
        Ok(())
    }
}

impl Node {
    pub fn get_store_range(&self) -> Range {
        if let Some(item) = self.stores_to() {
            item.pos
        } else {
            self.get_range()
        }
    }
}

// TODO this might differ based on how the nodes are made
// TODO store whole range for each ASTNode as field
// TODO only annotate CFGNode, not ASTNode
impl LineDisplay for Node {
    fn get_range(&self) -> Range {
        match &self {
            Node::ProgramEntry(_) => Range {
                start: Position { line: 0, column: 0 },
                end: Position { line: 0, column: 0 },
            },
            Node::UpperArith(x) => {
                let mut range = x.inst.pos.clone();
                range.end = x.imm.pos.end;
                range
            }
            Node::Label(x) => x.name.pos.clone(),
            Node::Arith(arith) => {
                let mut range = arith.inst.pos.clone();
                range.end = arith.rs2.pos.end;
                range
            }
            Node::IArith(iarith) => {
                let mut range = iarith.inst.pos.clone();
                range.end = iarith.imm.pos.end;
                range
            }
            Node::JumpLink(jl) => {
                let mut range = jl.inst.pos.clone();
                range.end = jl.name.pos.end;
                range
            }
            Node::JumpLinkR(jlr) => {
                let mut range = jlr.inst.pos.clone();
                range.end = jlr.inst.pos.end;
                range
            }
            Node::Branch(branch) => {
                let mut range = branch.inst.pos.clone();
                range.end = branch.name.pos.end;
                range
            }
            Node::Store(store) => {
                let mut range = store.inst.pos.clone();
                range.end = store.imm.pos.end;
                range
            }
            Node::Load(load) => {
                let mut range = load.inst.pos.clone();
                range.end = load.imm.pos.end;
                range
            }
            Node::Csr(csr) => {
                let mut range = csr.inst.pos.clone();
                range.end = csr.rs1.pos.end;
                range
            }
            Node::CsrI(csr) => {
                let mut range = csr.inst.pos.clone();
                range.end = csr.imm.pos.end;
                range
            }
            Node::Basic(x) => x.inst.pos.clone(),
            Node::LoadAddr(x) => {
                let mut range = x.inst.pos.clone();
                range.end = x.name.pos.end;
                range
            }
            Node::Directive(directive) => directive.dir.pos.clone(),
            Node::FuncEntry(_) => Range {
                start: Position { line: 0, column: 0 },
                end: Position { line: 0, column: 0 },
            },
        }
    }
}
