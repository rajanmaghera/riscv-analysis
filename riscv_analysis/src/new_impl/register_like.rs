use crate::new_impl::limited_element_set::LimitedElementSet;
use crate::parser::RVRegister;

/// There is a well-defined number of registers
pub trait RegisterLike: LimitedElementSet {}

impl RegisterLike for RVRegister {}
