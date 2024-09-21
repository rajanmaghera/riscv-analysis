use std::collections::HashSet;
use std::hash::Hash;

use crate::cfg::{AvailableValueMap, RegisterSet};
use crate::parser::Register;

use super::{AvailableValue, MemoryLocation};

pub trait CustomIntersection {
    #[must_use]
    fn intersection(&self, other: &Self) -> Self;
}
impl<T> CustomIntersection for AvailableValueMap<T>
where
    T: Eq + Hash + Clone,
{
    fn intersection(&self, other: &Self) -> Self {
        self.iter()
            .collect::<HashSet<_>>()
            .intersection(&other.iter().collect::<HashSet<_>>())
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect()
    }
}
pub trait CustomClonedSets<T> {
    #[must_use]
    fn intersection_c(&self, other: &Self) -> Self;
    #[must_use]
    fn union_c(&self, other: &Self) -> Self;
    #[must_use]
    fn difference_c(&self, other: &Self) -> Self;
}

impl CustomClonedSets<Register> for RegisterSet {
    fn intersection_c(&self, other: &Self) -> Self {
        self.intersection(other).iter().collect()
    }

    fn union_c(&self, other: &Self) -> Self {
        self.union(other).iter().collect()
    }

    fn difference_c(&self, other: &Self) -> Self {
        self.difference(other).iter().collect()
    }
}

pub trait CustomDifference<T> {
    #[must_use]
    fn difference(&self, other: &T) -> Self;
}

impl CustomDifference<RegisterSet> for AvailableValueMap<Register> {
    fn difference(&self, other: &RegisterSet) -> Self {
        self.iter()
            .filter(|(x, _)| !other.contains(x))
            .map(|(x, y)| (x.clone(), y.clone()))
            .collect()
    }
}

pub trait CustomUnion<T>
where
    Self: Sized + Clone + Default,
    T: Eq + Clone,
{
    #[must_use]
    fn union(&self, other: &T) -> Self;

    #[must_use]
    fn union_if(&self, other: &T, cond: bool) -> Self {
        if cond {
            self.union(other)
        } else {
            self.clone()
        }
    }
}

// Trait to union two items with a map and only if value is not None in closure
pub trait CustomUnionFilterMap<T, U>
where
    Self: Sized + Clone + Default,
    T: Eq + Clone,
    U: Eq + Clone,
{
    #[must_use]
    fn union_filter_map<F>(&self, other: &T, f: F) -> Self
    where
        F: Fn(&U) -> T;

    #[must_use]
    fn clear_if(&self, cond: bool) -> Self {
        if cond {
            Self::default()
        } else {
            self.clone()
        }
    }
}

impl CustomUnion<Option<(Register, AvailableValue)>> for AvailableValueMap<Register> {
    fn union(&self, other: &Option<(Register, AvailableValue)>) -> Self {
        let mut out = self.clone();
        if let Some((reg, val)) = other {
            out.insert(*reg, val.clone());
        }
        out
    }
}

impl CustomUnion<AvailableValueMap<Register>> for AvailableValueMap<Register> {
    fn union(&self, other: &AvailableValueMap<Register>) -> Self {
        let mut out = self.clone();
        for (reg, val) in other {
            out.insert(*reg, val.clone());
        }
        out
    }
}

impl CustomUnion<i32> for i32 {
    fn union(&self, other: &i32) -> Self {
        *self | *other
    }
}

impl
    CustomUnionFilterMap<Option<(MemoryLocation, AvailableValue)>, (MemoryLocation, AvailableValue)>
    for AvailableValueMap<MemoryLocation>
{
    fn union_filter_map<F>(&self, other: &Option<(MemoryLocation, AvailableValue)>, f: F) -> Self
    where
        F: Fn(&(MemoryLocation, AvailableValue)) -> Option<(MemoryLocation, AvailableValue)>,
    {
        let mut out = self.clone();
        if let Some((off, val)) = other {
            if let Some((new_off, new_val)) = f(&(off.clone(), val.clone())) {
                out.insert(new_off, new_val);
            }
        }
        out
    }
}

pub trait CustomInto<T> {
    fn into_available(self) -> T;
}

impl CustomInto<AvailableValueMap<Register>> for RegisterSet {
    fn into_available(self) -> AvailableValueMap<Register> {
        self.into_iter()
            .map(|x| (x, AvailableValue::OriginalRegisterWithScalar(x, 0)))
            .collect()
    }
}
