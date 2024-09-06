use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::parser::Register;

use super::AvailableValue;

pub trait CustomIntersection {
    #[must_use]
    fn intersection(&self, other: &Self) -> Self;
}
impl<T, U, S> CustomIntersection for HashMap<T, U, S>
where
    T: Eq + Hash + Clone,
    U: Eq + Hash + Clone,
    S: std::hash::BuildHasher + Default,
{
    fn intersection(&self, other: &Self) -> Self {
        self.iter()
            .collect::<HashSet<_>>()
            .intersection(&other.iter().collect::<HashSet<_>>())
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect::<HashMap<_, _, _>>()
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

impl<T, S> CustomClonedSets<T> for HashSet<T, S>
where
    T: Eq + Hash + Clone,
    S: std::hash::BuildHasher + Default,
{
    fn intersection_c(&self, other: &Self) -> Self {
        self.intersection(other).cloned().collect()
    }

    fn union_c(&self, other: &Self) -> Self {
        self.union(other).cloned().collect()
    }

    fn difference_c(&self, other: &Self) -> Self {
        self.difference(other).cloned().collect()
    }
}

pub trait CustomDifference<T> {
    #[must_use]
    fn difference(&self, other: &T) -> Self;
}

impl<T, U, S> CustomDifference<HashSet<T>> for HashMap<T, U, S>
where
    T: Eq + Hash + Clone,
    U: Eq + Hash + Clone,
    S: std::hash::BuildHasher + Default,
{
    fn difference(&self, other: &HashSet<T>) -> Self {
        self.iter()
            .filter(|(x, _)| !other.contains(x))
            .map(|(x, y)| (x.clone(), y.clone()))
            .collect()

        // for reg in other{
        //     out_n.remove(&reg);
        // }
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

impl<S> CustomUnion<Option<(Register, AvailableValue)>> for HashMap<Register, AvailableValue, S>
where
    S: std::hash::BuildHasher + Default + Clone,
{
    fn union(&self, other: &Option<(Register, AvailableValue)>) -> Self {
        let mut out = self.clone();
        if let Some((reg, val)) = other {
            out.insert(*reg, val.clone());
        }
        out
    }
}

impl<S> CustomUnion<HashMap<Register, AvailableValue>> for HashMap<Register, AvailableValue, S>
where
    S: std::hash::BuildHasher + Default + Clone,
{
    fn union(&self, other: &HashMap<Register, AvailableValue>) -> Self {
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

impl<S> CustomUnionFilterMap<Option<(i32, AvailableValue)>, (i32, AvailableValue)>
    for HashMap<i32, AvailableValue, S>
where
    S: std::hash::BuildHasher + Default + Clone,
{
    fn union_filter_map<F>(&self, other: &Option<(i32, AvailableValue)>, f: F) -> Self
    where
        F: Fn(&(i32, AvailableValue)) -> Option<(i32, AvailableValue)>,
    {
        let mut out = self.clone();
        if let Some((off, val)) = other {
            if let Some((new_off, new_val)) = f(&(*off, val.clone())) {
                out.insert(new_off, new_val);
            }
        }
        out
    }
}

pub trait CustomInto<T> {
    fn into_available(self) -> T;
}

impl<S> CustomInto<HashMap<Register, AvailableValue>> for HashSet<Register, S>
where
    S: std::hash::BuildHasher + Default + Clone,
{
    fn into_available(self) -> HashMap<Register, AvailableValue> {
        self.into_iter()
            .map(|x| (x, AvailableValue::OriginalRegisterWithScalar(x, 0)))
            .collect()
    }
}
