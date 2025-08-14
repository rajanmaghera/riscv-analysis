use crate::new_impl::set_traits::SetTraits;
use crate::parser::RVRegister;
use std::collections::HashSet;

/// A trait to put on an element, where there are only N elements
///
/// This is used so that we can create a bit-set representation
/// of the set.
pub trait LimitedElementSet: std::cmp::Eq + std::hash::Hash + Clone {
    type ArrayType: SetTraits<Self>;
    fn get_idx(&self) -> usize;
}

impl LimitedElementSet for RVRegister {
    type ArrayType = HashSet<RVRegister>;
    fn get_idx(&self) -> usize {
        todo!();
    }
}
