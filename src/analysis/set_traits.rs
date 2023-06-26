use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;

use crate::parser::Register;

use super::AvailableValue;

trait CustomDefault {
    fn def() -> Self;
}

impl<U, T> CustomDefault for std::collections::hash_map::IntoIter<U, T>
where
    T: Eq + Hash + Clone,
    U: Eq + Hash + Clone + Display,
{
    fn def() -> Self {
        HashMap::new().into_iter()
    }
}

pub trait CustomIntersection {
    fn intersection(&self, other: &Self) -> Self;
}
impl<T, U> CustomIntersection for HashMap<T, U>
where
    T: Eq + Hash + Clone,
    U: Eq + Hash + Clone,
{
    fn intersection(&self, other: &Self) -> Self {
        self.into_iter()
            .collect::<HashSet<_>>()
            .intersection(&other.into_iter().collect::<HashSet<_>>())
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect::<HashMap<_, _>>()
    }
}

// trait CustomBlankIterator<T>
// where
//     T: Sized + Eq + Hash + Clone,
//     Self: Sized + IntoIterator<Item = T>,
// {
//     fn blank() -> Box<dyn Iterator<Item = T> + Default + CustomIntersection + CustomBlankIterator>;
// }

// impl<U, T> CustomIntersection for std::collections::hash_map::IntoIter<U, T>
// where
//     T: Eq + Hash + Clone,
//     U: Eq + Hash + Clone + Display,
// {
//     fn intersection(&self, other: &Self) -> Self {
//         let a = self.clone().collect::<HashSet<_>>();
//         let b = other.clone().collect::<HashSet<_>>();
//         a.intersection(&b)
//             .cloned()
//             .collect::<HashMap<_, _>>()
//             .into_iter()
//     }
// }

// impl CustomBlankIterator<Register> for HashSet<Register> {
//     fn blank() -> Box<dyn Iterator<Item = Register>> {
//         Box::new(HashSet::new().into_iter())
//     }
// }

// pub trait ListIntersection<T> {
//     fn intersection_all(&mut self) -> T;
// }

// impl<T, I, U> ListIntersection<HashMap<U, T>> for I
// where
//     T: Eq + Hash + Clone,
//     I: Iterator<Item = HashMap<U, T>>,
//     U: Eq + Hash + Clone + Display,
// {
//     fn intersection_all(&mut self) -> HashMap<U, T> {
//         let mut list = self.map(|x| x.into_iter()).into_iter();
//         intersection_list(&mut list).collect()
//     }
// }

pub trait CustomClonedSets<T> {
    fn intersection_c(&self, other: &Self) -> Self;
    fn union_c(&self, other: &Self) -> Self;
    fn difference_c(&self, other: &Self) -> Self;
}

impl<T> CustomClonedSets<T> for HashSet<T>
where
    T: Eq + Hash + Clone,
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
    fn difference(&self, other: &T) -> Self;
}

impl<T, U> CustomDifference<HashSet<T>> for HashMap<T, U>
where
    T: Eq + Hash + Clone,
    U: Eq + Hash + Clone,
{
    fn difference(&self, other: &HashSet<T>) -> Self {
        self.into_iter()
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
    fn union(&self, other: &T) -> Self;

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
    fn union_filter_map<F>(&self, other: &T, f: F) -> Self
    where
        F: Fn(&U) -> T;
    fn clear_if(&self, cond: bool) -> Self {
        if cond {
            Self::default()
        } else {
            self.clone()
        }
    }
}

impl CustomUnion<Option<(Register, AvailableValue)>> for HashMap<Register, AvailableValue> {
    fn union(&self, other: &Option<(Register, AvailableValue)>) -> Self {
        let mut out = self.clone();
        if let Some((reg, val)) = other {
            out.insert(*reg, val.clone());
        }
        out
    }
}

impl CustomUnion<HashMap<Register, AvailableValue>> for HashMap<Register, AvailableValue> {
    fn union(&self, other: &HashMap<Register, AvailableValue>) -> Self {
        let mut out = self.clone();
        for (reg, val) in other {
            out.insert(*reg, val.clone());
        }
        out
    }
}

impl CustomUnion<i32> for i32 {
    #[inline(always)]
    fn union(&self, other: &i32) -> Self {
        *self | *other
    }
}

impl CustomUnionFilterMap<Option<(i32, AvailableValue)>, (i32, AvailableValue)>
    for HashMap<i32, AvailableValue>
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

impl CustomInto<HashMap<Register, AvailableValue>> for HashSet<Register> {
    fn into_available(self) -> HashMap<Register, AvailableValue> {
        self.into_iter()
            .map(|x| (x, AvailableValue::OriginalRegisterWithScalar(x, 0)))
            .collect()
    }
}
