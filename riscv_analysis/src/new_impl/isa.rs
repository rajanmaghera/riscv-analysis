use crate::new_impl::instruction_like::InstructionLike;

pub trait ISA {
    type Instruction: InstructionLike;
}
