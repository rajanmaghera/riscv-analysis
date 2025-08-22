use std::collections::HashSet;

use crate::new_impl::risc_v_implementation::RealInst;


pub trait InterproceduralInstructionIterator {
    fn get_next_insts_interprocedural(&self, inst: &RealInst) -> HashSet<&RealInst>;
    fn get_prev_insts_interprocedural(&self, inst: &RealInst) -> HashSet<&RealInst>;
}
