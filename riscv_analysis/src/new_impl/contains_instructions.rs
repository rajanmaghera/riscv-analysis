use crate::new_impl::risc_v_implementation::RealInst;

pub trait ContainsInstructions {
    fn get_next_insts_intraprocedural(&self, inst: &RealInst) -> Option<impl Iterator<Item = &RealInst>>;
    fn get_prev_insts_intraprocedural(&self, inst: &RealInst) -> Option<impl Iterator<Item = &RealInst>>;
}
