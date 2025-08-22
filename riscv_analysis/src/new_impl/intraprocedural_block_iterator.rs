use std::collections::HashSet;

use crate::new_impl::{contains_functions::ContainsFunctions, risc_v_implementation::RealBasicBlock};


pub trait IntraproceduralBlockIterator<'a> {
    fn get_next_blocks_intraprocedural(&'a self, block: &'a RealBasicBlock) -> HashSet<&'a RealBasicBlock<'a>>;
    fn get_prev_blocks_intraprocedural(&'a self, block: &'a RealBasicBlock) -> HashSet<&'a RealBasicBlock<'a>>;
}

impl<'a, T: ContainsFunctions> IntraproceduralBlockIterator<'a> for T {
    fn get_next_blocks_intraprocedural(&'a self, block: &'a RealBasicBlock) -> HashSet<&'a RealBasicBlock<'a>> {
        self.get_function_of_basic_block(block).get_next_blocks_intraprocedural(block)
    }

    fn get_prev_blocks_intraprocedural(&'a self, block: &'a RealBasicBlock) -> HashSet<&'a RealBasicBlock<'a>> {
        self.get_function_of_basic_block(block).get_prev_blocks_intraprocedural(block)
    }
}
