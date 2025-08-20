use std::collections::HashSet;

use crate::new_impl::risc_v_implementation::RealInst;


pub trait IntraproceduralInstructionIterator {
    fn get_next_insts_intraprocedural(&self, inst: &RealInst) -> Option<HashSet<&RealInst>>;
    fn get_prev_insts_intraprocedural(&self, inst: &RealInst) -> Option<HashSet<&RealInst>>;
}
