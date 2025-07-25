pub trait SetCombination<T>: IntoIterator<Item = T> {
    fn union(&self, other: &Self) -> Self;
    fn difference(&self, other: &Self) -> Self;
    fn intersection(&self, other: &Self) -> Self;
}

impl<T, U> SetCombination<T> for U
where
    U: IntoIterator<Item = T>,
{
    fn union(&self, other: &Self) -> Self {
        todo!()
    }

    fn difference(&self, other: &Self) -> Self {
        todo!()
    }

    fn intersection(&self, other: &Self) -> Self {
        todo!()
    }
}
