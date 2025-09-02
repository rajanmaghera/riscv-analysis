use crate::new_impl::instruction_like::InstructionLike;
use crate::new_impl::limited_element_set::LimitedElementSet;
use crate::new_impl::risc_v_implementation::RealInst;
use std::collections::HashSet;

pub trait HasGenKill: InstructionLike {
    fn get_gen(&self) -> <<Self as InstructionLike>::Register as LimitedElementSet>::ArrayType;
    fn get_kill(&self) -> <<Self as InstructionLike>::Register as LimitedElementSet>::ArrayType;
}

impl HasGenKill for RealInst {
    fn get_gen(&self) -> <<Self as InstructionLike>::Register as LimitedElementSet>::ArrayType {
        todo!()
    }

    fn get_kill(&self) -> HashSet<<Self as InstructionLike>::Register> {
        todo!()
    }
}
