use crate::parser::imm::{CSRImm, Imm};
use crate::parser::inst::Inst;
use crate::parser::inst::{
    ArithType, BasicType, BranchType, CSRIType, CSRType, IArithType, JumpLinkRType, JumpLinkType,
    LoadType, PseudoType, StoreType,
};

use crate::parser::register::Register;
use crate::parser::token::With;

use std::collections::HashSet;

use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    Arith, Basic, Branch, Csr, CsrI, Directive, DirectiveToken, DirectiveType, FuncEntry, IArith,
    JumpLink, JumpLinkR, Label, LabelString, Load, LoadAddr, ProgramEntry, RawToken, Store,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParserNode {
    ProgramEntry(ProgramEntry),
    FuncEntry(FuncEntry),
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
    Csr(Csr),
    CsrI(CsrI),
}

impl ParserNode {
    #[must_use]
    pub fn token(&self) -> RawToken {
        match self {
            ParserNode::Arith(x) => x.token.clone(),
            ParserNode::IArith(x) => x.token.clone(),
            ParserNode::Label(x) => x.token.clone(),
            ParserNode::JumpLink(x) => x.token.clone(),
            ParserNode::JumpLinkR(x) => x.token.clone(),
            ParserNode::Basic(x) => x.token.clone(),
            ParserNode::Directive(x) => x.token.clone(),
            ParserNode::Branch(x) => x.token.clone(),
            ParserNode::Store(x) => x.token.clone(),
            ParserNode::Load(x) => x.token.clone(),
            ParserNode::Csr(x) => x.token.clone(),
            ParserNode::CsrI(x) => x.token.clone(),
            ParserNode::LoadAddr(x) => x.token.clone(),
            ParserNode::ProgramEntry(x) => x.token.clone(),
            ParserNode::FuncEntry(x) => x.token.clone(),
        }
    }

    #[must_use]
    pub fn id(&self) -> Uuid {
        match self {
            ParserNode::Arith(a) => a.key,
            ParserNode::IArith(a) => a.key,
            ParserNode::Label(a) => a.key,
            ParserNode::JumpLink(a) => a.key,
            ParserNode::JumpLinkR(a) => a.key,
            ParserNode::Basic(a) => a.key,
            ParserNode::Directive(a) => a.key,
            ParserNode::Branch(a) => a.key,
            ParserNode::Store(a) => a.key,
            ParserNode::Load(a) => a.key,
            ParserNode::Csr(a) => a.key,
            ParserNode::CsrI(a) => a.key,
            ParserNode::LoadAddr(a) => a.key,
            ParserNode::FuncEntry(a) => a.key,
            ParserNode::ProgramEntry(a) => a.key,
        }
    }
}

impl PartialEq for ParserNode {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
impl Eq for ParserNode {}
impl Hash for ParserNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl ParserNode {
    #[must_use]
    pub fn inst(&self) -> Inst {
        match self {
            ParserNode::Arith(x) => (&x.inst.data).into(),
            ParserNode::IArith(x) => (&x.inst.data).into(),
            ParserNode::JumpLink(x) => (&x.inst.data).into(),
            ParserNode::JumpLinkR(x) => (&x.inst.data).into(),
            ParserNode::Basic(x) => (&x.inst.data).into(),
            ParserNode::Branch(x) => (&x.inst.data).into(),
            ParserNode::Store(x) => (&x.inst.data).into(),
            ParserNode::Load(x) => (&x.inst.data).into(),
            ParserNode::Csr(x) => (&x.inst.data).into(),
            ParserNode::CsrI(x) => (&x.inst.data).into(),
            ParserNode::LoadAddr(_) => Inst::La,
            ParserNode::Label(_)
            | ParserNode::Directive(_)
            | ParserNode::FuncEntry(_)
            | ParserNode::ProgramEntry(_) => Inst::Nop,
        }
    }

    #[must_use]
    pub fn new_arith(
        inst: With<ArithType>,
        rd: With<Register>,
        rs1: With<Register>,
        rs2: With<Register>,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::Arith(Arith {
            inst,
            rd,
            rs1,
            rs2,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_iarith(
        inst: With<IArithType>,
        rd: With<Register>,
        rs1: With<Register>,
        imm: With<Imm>,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::IArith(IArith {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_jump_link(
        inst: With<JumpLinkType>,
        rd: With<Register>,
        name: With<LabelString>,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::JumpLink(JumpLink {
            inst,
            rd,
            name,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_jump_link_r(
        inst: With<JumpLinkRType>,
        rd: With<Register>,
        rs1: With<Register>,
        imm: With<Imm>,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::JumpLinkR(JumpLinkR {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_basic(inst: With<BasicType>, token: RawToken) -> ParserNode {
        ParserNode::Basic(Basic {
            inst,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_directive(
        dir_token: With<DirectiveToken>,
        dir: DirectiveType,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::Directive(Directive {
            dir_token,
            dir,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_branch(
        inst: With<BranchType>,
        rs1: With<Register>,
        rs2: With<Register>,
        name: With<LabelString>,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::Branch(Branch {
            inst,
            rs1,
            rs2,
            name,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_store(
        inst: With<StoreType>,
        rs1: With<Register>,
        rs2: With<Register>,
        imm: With<Imm>,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::Store(Store {
            inst,
            rs1,
            rs2,
            imm,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_load(
        inst: With<LoadType>,
        rd: With<Register>,
        rs1: With<Register>,
        imm: With<Imm>,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::Load(Load {
            inst,
            rd,
            rs1,
            imm,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_csr(
        inst: With<CSRType>,
        rd: With<Register>,
        csr: With<CSRImm>,
        rs1: With<Register>,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::Csr(Csr {
            inst,
            rd,
            rs1,
            csr,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_func_entry(file: Uuid, token: RawToken) -> ParserNode {
        ParserNode::FuncEntry(FuncEntry {
            key: Uuid::new_v4(),
            file,
            token,
        })
    }

    #[must_use]
    pub fn new_program_entry(file: Uuid, token: RawToken) -> ParserNode {
        ParserNode::ProgramEntry(ProgramEntry {
            key: Uuid::new_v4(),
            file,
            token,
        })
    }

    #[must_use]
    pub fn new_csri(
        inst: With<CSRIType>,
        rd: With<Register>,
        csr: With<CSRImm>,
        imm: With<Imm>,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::CsrI(CsrI {
            inst,
            rd,
            imm,
            csr,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_label(name: With<LabelString>, token: RawToken) -> ParserNode {
        ParserNode::Label(Label {
            name,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn new_load_addr(
        inst: With<PseudoType>,
        rd: With<Register>,
        name: With<LabelString>,
        token: RawToken,
    ) -> ParserNode {
        ParserNode::LoadAddr(LoadAddr {
            inst,
            rd,
            name,
            key: Uuid::new_v4(),
            token,
        })
    }

    #[must_use]
    pub fn is_return(&self) -> bool {
        match self {
            ParserNode::JumpLinkR(x) => {
                x.inst == JumpLinkRType::Jalr
                    && x.rd == Register::X0
                    && x.rs1 == Register::X1
                    && x.imm == Imm(0)
            }
            ParserNode::Basic(x) => x.inst == BasicType::Uret,
            _ => false,
        }
    }

    /// Checks if a instruction is a return from an interrupt handler.
    ///
    /// This function returns to the value in uepc.
    #[must_use]
    pub fn is_ureturn(&self) -> bool {
        match self {
            ParserNode::Basic(x) => x.inst == BasicType::Uret,
            _ => false,
        }
    }

    /// Checks if a instruction is meant to be saved to zero
    ///
    /// Some instructions save to zero as part of their design. For example,
    /// jumps that link to zero. However, some have no effect even while
    /// saving to zero. For example, `addi x0, x0, 0` is a no-op.
    /// This function determines if an instruction is meant to be saved to zero
    /// or if it is a no-op. No-ops are treated as warnings, not errors.
    #[must_use]
    pub fn can_skip_save_checks(&self) -> bool {
        matches!(
            self,
            ParserNode::ProgramEntry(_)
                | ParserNode::FuncEntry(_)
                | ParserNode::JumpLink(_)
                | ParserNode::JumpLinkR(_)
                | ParserNode::Csr(_)
                | ParserNode::CsrI(_)
        )
    }

    /// Checks if a instruction is a function call
    #[must_use]
    pub fn calls_to(&self) -> Option<With<LabelString>> {
        match self {
            ParserNode::JumpLink(x) if x.rd == Register::X1 => Some(x.name.clone()),
            _ => None,
        }
    }

    /// Checks if a instruction is an environment call
    #[must_use]
    pub fn is_ecall(&self) -> bool {
        match self {
            ParserNode::Basic(x) => x.inst == BasicType::Ecall,
            _ => false,
        }
    }

    /// Checks if a instruction is a potential jump
    #[must_use]
    pub fn jumps_to(&self) -> Option<With<LabelString>> {
        match self {
            ParserNode::JumpLink(x) if x.rd != Register::X1 => Some(x.name.clone()),
            ParserNode::Branch(x) => Some(x.name.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub fn reads_address_of(&self) -> Option<With<LabelString>> {
        match self {
            ParserNode::LoadAddr(x) => Some(x.name.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_any_entry(&self) -> bool {
        matches!(self, ParserNode::ProgramEntry(_) | ParserNode::FuncEntry(_))
    }

    #[must_use]
    pub fn is_function_entry(&self) -> bool {
        matches!(self, ParserNode::FuncEntry(_))
    }

    #[must_use]
    pub fn is_program_entry(&self) -> bool {
        matches!(self, ParserNode::ProgramEntry(_))
    }

    /// Check if a node is an instruction.
    #[must_use]
    pub fn is_instruction(&self) -> bool {
        matches!(
            self,
            ParserNode::Arith(_)
                | ParserNode::IArith(_)
                | ParserNode::JumpLink(_)
                | ParserNode::JumpLinkR(_)
                | ParserNode::Basic(_)
                | ParserNode::Branch(_)
                | ParserNode::Store(_)
                | ParserNode::Load(_)
                | ParserNode::LoadAddr(_)
                | ParserNode::Csr(_)
                | ParserNode::CsrI(_)
        )
    }

    /// Either loads or stores to a memory location
    #[must_use]
    pub fn uses_memory_location(&self) -> Option<(Register, Imm)> {
        match self {
            ParserNode::Store(s) => Some((s.rs1.data, s.imm.data.clone())),
            ParserNode::Load(l) => Some((l.rs1.data, l.imm.data.clone())),
            _ => None,
        }
    }

    #[must_use]
    /// Checks whether a jump is unconditional with no side effects
    ///
    /// Some jumps have side effects, like jumping to a function which sets
    /// the return address. This function checks if a jump is unconditional
    /// and has no side effects.
    pub fn is_unconditional_jump(&self) -> bool {
        match self {
            ParserNode::JumpLink(x) if x.rd == Register::X0 => true,
            ParserNode::JumpLinkR(x) if x.rd == Register::X0 => true,
            ParserNode::Branch(x) => {
                x.rs1 == Register::X0
                    && x.rs2 == Register::X0
                    && (x.inst == BranchType::Beq
                        || x.inst == BranchType::Bge
                        || x.inst == BranchType::Bgeu)
            }
            _ => false,
        }
    }

    // NOTE: This is in context to a register store, not a memory store
    #[must_use]
    pub fn stores_to(&self) -> Option<With<Register>> {
        match self {
            ParserNode::Load(load) => Some(load.rd.clone()),
            ParserNode::LoadAddr(load) => Some(load.rd.clone()),
            ParserNode::Arith(arith) => Some(arith.rd.clone()),
            ParserNode::IArith(iarith) => Some(iarith.rd.clone()),
            ParserNode::JumpLink(jump_link) => Some(jump_link.rd.clone()),
            ParserNode::JumpLinkR(jump_link_r) => Some(jump_link_r.rd.clone()),
            ParserNode::Csr(csr) => Some(csr.rd.clone()),
            ParserNode::CsrI(csri) => Some(csri.rd.clone()),
            ParserNode::ProgramEntry(_)
            | ParserNode::FuncEntry(_)
            | ParserNode::Label(_)
            | ParserNode::Basic(_)
            | ParserNode::Directive(_)
            | ParserNode::Branch(_)
            | ParserNode::Store(_) => None,
        }
    }

    #[must_use]
    pub fn reads_from(&self) -> HashSet<With<Register>> {
        let vector = match self {
            ParserNode::Arith(x) => vec![x.rs1.clone(), x.rs2.clone()],
            ParserNode::IArith(x) => vec![x.rs1.clone()],
            ParserNode::JumpLinkR(x) => vec![x.rs1.clone()],
            ParserNode::Branch(x) => vec![x.rs1.clone(), x.rs2.clone()],
            ParserNode::Store(x) => vec![x.rs1.clone(), x.rs2.clone()],
            ParserNode::Load(x) => vec![x.rs1.clone()],
            ParserNode::Csr(x) => vec![x.rs1.clone()],
            ParserNode::ProgramEntry(_)
            | ParserNode::FuncEntry(_)
            | ParserNode::Label(_)
            | ParserNode::JumpLink(_)
            | ParserNode::Basic(_)
            | ParserNode::Directive(_)
            | ParserNode::LoadAddr(_)
            | ParserNode::CsrI(_) => vec![],
        };
        vector.into_iter().collect()
    }

    pub fn set_uuid(&mut self, uuid: Uuid) {
        match self {
            ParserNode::Arith(x) => x.key = uuid,
            ParserNode::IArith(x) => x.key = uuid,
            ParserNode::Label(x) => x.key = uuid,
            ParserNode::JumpLink(x) => x.key = uuid,
            ParserNode::JumpLinkR(x) => x.key = uuid,
            ParserNode::Basic(x) => x.key = uuid,
            ParserNode::Directive(x) => x.key = uuid,
            ParserNode::Branch(x) => x.key = uuid,
            ParserNode::Store(x) => x.key = uuid,
            ParserNode::Load(x) => x.key = uuid,
            ParserNode::Csr(x) => x.key = uuid,
            ParserNode::CsrI(x) => x.key = uuid,
            ParserNode::LoadAddr(x) => x.key = uuid,
            ParserNode::ProgramEntry(_) | ParserNode::FuncEntry(_) => (),
        }
    }
}
