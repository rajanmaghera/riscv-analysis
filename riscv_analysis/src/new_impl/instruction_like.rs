use crate::new_impl::register_like::RegisterLike;
use crate::new_impl::risc_v_implementation::RealInst;
use crate::parser::{HasIdentity, Register};

pub trait InstructionLike {
    type Register: RegisterLike;
}

impl InstructionLike for RealInst {
    type Register = Register;
}

pub trait InstructionLikeInProg: InstructionLike + HasIdentity {}
