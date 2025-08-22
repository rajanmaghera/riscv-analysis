use crate::new_impl::{contains_basic_blocks::ContainsBasicBlocks, risc_v_implementation::RealInst};


pub trait IntrablockInstructionIterator {
    fn get_next_inst_intrablock(&self, inst: &RealInst) -> Option<&RealInst>;
    fn get_prev_inst_intrablock(&self, inst: &RealInst) -> Option<&RealInst>;
}


impl<T: ContainsBasicBlocks> IntrablockInstructionIterator for T {
    /// Get the next instruction after `inst` within `inst`'s basic block.
    ///
    /// Returns `None` if `inst` is the last instruction in its basic block
    /// and thus has no next instruction in its block.
    fn get_next_inst_intrablock(&self, inst: &RealInst) -> Option<&RealInst> {
        self.get_basic_block_of_inst(inst).get_next_inst_intrablock(inst)
    }

    /// Get the previous instruction before `inst` within `inst`'s basic block.
    ///
    /// Returns `None` if `inst` is the first instruction in its basic block
    /// and thus has no previous instruction in its block.
    fn get_prev_inst_intrablock(&self, inst: &RealInst) -> Option<&RealInst> {
        self.get_basic_block_of_inst(inst).get_prev_inst_intrablock(inst)
    }
}
