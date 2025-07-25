use crate::new_impl::limited_element_set::LimitedElementSet;
use crate::parser::Register;

/// There is a well-defined number of registers
pub trait RegisterLike: LimitedElementSet {}

impl RegisterLike for Register {}
