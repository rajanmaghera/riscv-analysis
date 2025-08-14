use crate::new_impl::set_combination::SetCombination;
use crate::new_impl::set_traits::SetTraits;

pub(crate) trait SetCombinationFold<T, U: SetTraits<T>>: IntoIterator<Item = U> {
    fn union_all(self) -> Self::Item
    where
        Self: Sized,
    {
        self.into_iter()
            .reduce(|acc, x| acc.union(&x))
            .unwrap_or_else(|| U::new())
    }

    fn intersection_all(self) -> Self::Item
    where
        Self: Sized,
    {
        self.into_iter()
            .reduce(|acc, x| acc.intersection(&x))
            .unwrap_or_else(|| U::new())
    }
}

impl<T, U: SetTraits<T>, V: Iterator<Item = U>> SetCombinationFold<T, U> for V {}
