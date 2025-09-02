use crate::new_impl::limited_element_set::LimitedElementSet;
use crate::new_impl::set_combination::SetCombination;

pub trait SetTraits<T>: SetCombination<T> + Clone + Eq {
    fn new() -> Self;
    // fn insert(&mut self, item: T);
    // fn remove(&mut self, item: &T);
    // fn contains(&self, item: &T) -> bool;
    // fn clear(&mut self);
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
}

impl<T: LimitedElementSet, U> SetTraits<T> for U
where
    U: Clone + Eq + SetCombination<T>,
{
    fn new() -> Self {
        todo!()
    }

    fn is_empty(&self) -> bool {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }
}
