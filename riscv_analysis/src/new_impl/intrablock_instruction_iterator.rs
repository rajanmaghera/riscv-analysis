use crate::new_impl::risc_v_implementation::RealInst;


pub trait IntrablockInstructionIterator {
    fn get_next_inst_intrablock(&self, inst: &RealInst) -> Option<&RealInst>;
    fn get_prev_inst_intrablock(&self, inst: &RealInst) -> Option<&RealInst>;
}
