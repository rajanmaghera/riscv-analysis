use uuid::Uuid;

use crate::new_impl::risc_v_implementation::RealInst;

pub trait ContainsInstructions {
    fn get_inst_by_id(&self, inst_id: &Uuid) -> &RealInst;
}
