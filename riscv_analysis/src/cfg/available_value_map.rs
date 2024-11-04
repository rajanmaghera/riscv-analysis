use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;
use std::ops::{BitAndAssign, SubAssign};
use itertools::Itertools;

use serde::{Deserialize, Serialize};

use crate::{analysis::AvailableValue, parser::Register};

use super::RegisterSet;

/// A set that is either a finite set, or the universe subtract a finite set.
#[derive(Debug, Clone, PartialEq, Eq)]
enum MaybeUniverse<T: PartialEq + Eq + Hash> {
    Finite(HashMap<T, AvailableValue>),
    Universe(HashSet<T>),
}

impl<T: PartialEq + Eq + Hash> MaybeUniverse<T> {
    /// Return a new set with value V - {}.
    pub fn universe() -> Self {
        Self::Universe(HashSet::new())
    }

    pub fn empty() -> Self {
        Self::Finite(HashMap::new())
    }
}

/// A map from a generic item to an available value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableValueMap<T: PartialEq + Eq + Hash> {
    /// The map from the item to the available value.
    map: MaybeUniverse<T>,
}

impl<T: PartialEq + Eq + Hash + Clone> AvailableValueMap<T> {
    /// Create a new `AvailableValueMap` with no values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: MaybeUniverse::empty(),
        }
    }

    #[must_use]
    pub fn universe() -> Self {
        Self {
            map: MaybeUniverse::universe(),
        }
    }

    /// Iterate over all values
    #[must_use]
    pub fn iter(&self) -> AvailableValueIterator<T> {
        AvailableValueIterator::new(self.clone())
    }

    /// Get the available value for the given key.
    pub fn get(&self, item: &T) -> Option<&AvailableValue> {
        match &self.map {
            MaybeUniverse::Finite(map) => map.get(item),
            MaybeUniverse::Universe(_) => None,
        }
    }

    /// Insert or replace the given item and available value into the map.
    pub fn insert(&mut self, key: T, value: AvailableValue) {
        // self.map.insert(key, value);
        match &mut self.map {
            MaybeUniverse::Finite(map) => { map.insert(key, value); },
            MaybeUniverse::Universe(set) => { set.remove(&key); },
        };
    }

    /// Check if the available value map is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match &self.map {
            MaybeUniverse::Finite(map) => map.is_empty(),
            MaybeUniverse::Universe(_) => false,
        }
    }

    /// Get the number of items in the available value map.
    #[must_use]
    pub fn len(&self) -> usize {
        match &self.map {
            MaybeUniverse::Finite(map) => map.len(),
            // FIXME: Length should not be zero
            MaybeUniverse::Universe(_) => 0,
        }
    }

    /// Extend the available value map with another available value map.
    pub fn extend(&mut self, other: Self) {
        match &mut self.map {
            MaybeUniverse::Finite(map) => {
                match other.map {
                    // A | B
                    MaybeUniverse::Finite(o_map) => {
                        map.extend(o_map.clone());
                    },
                    // A | (V - B) = V - (B - A)
                    MaybeUniverse::Universe(mut o_set) => {
                        o_set.retain(|key| !map.contains_key(key));
                        self.map = MaybeUniverse::Universe(o_set.clone());
                    },
                };
            },
            MaybeUniverse::Universe(set) => {
                match &other.map {
                    // (V - A) | B = V - (A - B)
                    MaybeUniverse::Finite(o_map) => {
                        set.retain(|key| !o_map.contains_key(key));
                    },
                    // (V - A) | (V - B) = V - (A & B)
                    MaybeUniverse::Universe(o_set) => {
                        set.retain(|key| o_set.contains(key));
                    },
                };
            },
        }
    }
}

pub struct AvailableValueIterator<T: PartialEq + Eq + Hash> {
    values: Vec<(T, AvailableValue)>,
}

impl<T: PartialEq + Eq + Hash> AvailableValueIterator<T> {
    #[must_use]
    pub fn new(map: AvailableValueMap<T>) -> Self {
        let values = match map.map {
            MaybeUniverse::Finite(map) => map,
            MaybeUniverse::Universe(_set) => HashMap::new(),
        }.into_iter().collect::<Vec<_>>();

        Self {
            values,
        }
    }

}

impl<T: PartialEq + Eq + Hash> Iterator for AvailableValueIterator<T> {
    type Item = (T, AvailableValue);

    fn next(&mut self) -> Option<Self::Item> {
        self.values.pop()
    }
}

impl<T: PartialEq + Eq + Hash + Clone> IntoIterator for &AvailableValueMap<T> {
    type IntoIter = AvailableValueIterator<T>;
    type Item = (T, AvailableValue);
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl AvailableValueMap<Register> {
    /// Returns the offset of the stack pointer if it is known.
    ///
    /// This is used to determine the offset of the stack pointer in relation
    /// to the value it was at the beginning of the function or graph.
    #[must_use]
    pub fn stack_offset(&self) -> Option<i32> {
        if let Some(AvailableValue::OriginalRegisterWithScalar(reg, off)) = self.get(&Register::X2)
        {
            if *reg == Register::X2 {
                return Some(*off);
            }
        }
        None
    }

    /// Determine if a register has the same value it had at the beginning of
    /// a function.
    #[must_use]
    pub fn is_original_value(&self, reg: Register) -> bool {
        if let Some(AvailableValue::OriginalRegisterWithScalar(register, offset)) = self.get(&reg) {
            reg == *register && *offset == 0
        } else {
            false
        }
    }
}

impl<T: PartialEq + Eq + Hash + Clone> Default for AvailableValueMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Since pure subtraction would require a clone we disable it for just `SubAssign`.

impl<T: Iterator<Item = Register>> SubAssign<T> for AvailableValueMap<Register> {
    fn sub_assign(&mut self, other: T) {
        for register in other {
            match &mut self.map {
                MaybeUniverse::Finite(map) => { map.remove(&register); },
                MaybeUniverse::Universe(set) => { set.insert(register); },
            }
        }
    }
}

impl<T: PartialEq + Eq + Hash + Clone> BitAndAssign<&AvailableValueMap<T>> for AvailableValueMap<T> {
    fn bitand_assign(&mut self, other: &AvailableValueMap<T>) {
        match &mut self.map {
            MaybeUniverse::Finite(map) => {
                match &other.map {
                    // A & B
                    MaybeUniverse::Finite(o_map) => {
                        map.retain(|key, value| o_map.get(key) == Some(value));
                    },
                    // A & (V - B) = A - B
                    MaybeUniverse::Universe(o_set) => {
                        map.retain(|key, _value| !o_set.contains(key));
                    },
                };
            },
            MaybeUniverse::Universe(set) => {
                match &other.map {
                    // (V - B) & A = A - B
                    MaybeUniverse::Finite(o_map) => {
                        let mut map = o_map.clone();
                        map.retain(|key, _value| !set.contains(key));
                        self.map = MaybeUniverse::Finite(map);
                    },
                    // (V - A) & (V - B) = V - (A | B)
                    MaybeUniverse::Universe(o_set) => {
                        set.extend(o_set.iter().cloned());
                    },
                };
            },
        }
    }
}

impl<T: PartialEq + Eq + Hash + Clone + Display> std::fmt::Display for AvailableValueMap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let values = self
            .iter()
            .map(|(reg, val)| format!("{reg}: {val}"))
            .sorted()
            .join(", ");
        write!(f, "[{values}]")
    }
}

impl<'a, T: PartialEq + Eq + Hash + Clone + Deserialize<'a>> Deserialize<'a> for AvailableValueMap<T> {
    fn deserialize<D>(deserializer: D) -> Result<AvailableValueMap<T>, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let map = HashMap::<T, AvailableValue>::deserialize(deserializer)?;
        let mut available_value_map = AvailableValueMap::new();
        for (key, value) in map {
            available_value_map.insert(key, value);
        }
        Ok(available_value_map)
    }
}

impl<T: PartialEq + Eq + Hash + Serialize + Ord + Clone> Serialize for AvailableValueMap<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.iter()
            .sorted_by_key(|(key, _)| key.clone())
            .collect::<BTreeMap<_, _>>()
            .serialize(serializer)
    }
}

impl<T: PartialEq + Eq + Hash + Clone> FromIterator<(T, AvailableValue)> for AvailableValueMap<T> {
    fn from_iter<I: IntoIterator<Item = (T, AvailableValue)>>(iter: I) -> Self {
        let mut map = AvailableValueMap::new();
        for (key, value) in iter {
            map.insert(key, value);
        }
        map
    }
}

/// LEGACY METHODS:

impl AvailableValueMap<Register> {
    #[must_use]
    pub fn difference(&self, other: &RegisterSet) -> Self {
        let mut new_map = self.clone();
        new_map -= other.iter();
        new_map
    }

    #[must_use]
    pub fn union(&self, other: &Option<(Register, AvailableValue)>) -> Self {
        let mut new_map = self.clone();
        if let Some((reg, value)) = other {
            new_map.insert(*reg, value.clone());
        }
        new_map
    }

    #[must_use]
    pub fn union_if(&self, other: &Self, condition: bool) -> Self {
        if condition {
            let mut new = self.clone();
            for (reg, value) in other {
                new.insert(reg, value.clone());
            }
            new
        } else {
            self.clone()
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn can_create_an_empty_map() {
        let map = AvailableValueMap::<Register>::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        let mut map_iter = map.iter();
        assert!(map_iter.next().is_none(), "Map should be empty");
    }

    #[test]
    fn can_use_map_methods() {
        let mut map = AvailableValueMap::<Register>::new();
        map.insert(
            Register::X1,
            AvailableValue::OriginalRegisterWithScalar(Register::X1, 0),
        );
        map.insert(Register::X2, AvailableValue::Constant(18));
        assert!(!map.is_empty());
        assert_eq!(map.len(), 2);
        let mut map_iter = map.iter().sorted_by_key(|(reg, _)| *reg);
        assert_eq!(
            map_iter.next(),
            Some((
                Register::X1,
                AvailableValue::OriginalRegisterWithScalar(Register::X1, 0)
            ))
        );
        assert_eq!(
            map_iter.next(),
            Some((Register::X2, AvailableValue::Constant(18)))
        );
        assert!(map_iter.next().is_none(), "Map should be empty");
    }

    #[test]
    fn can_serialize_a_basic_map() {
        let mut map = AvailableValueMap::<Register>::new();
        map.insert(
            Register::X1,
            AvailableValue::OriginalRegisterWithScalar(Register::X1, 0),
        );
        map.insert(Register::X2, AvailableValue::Constant(18));

        let serialized = serde_yaml::to_string(&map).unwrap();
        assert_eq!(serialized, "1: !ors\n- 1\n- 0\n2: !c 18\n");
    }

    #[test]
    fn can_deserialize_a_basic_map() {
        let serialized = "1: !ors\n- 1\n- 0\n2: !ors\n- 2\n- 18\n";
        let map = serde_yaml::from_str::<AvailableValueMap<Register>>(serialized).unwrap();
        assert_eq!(
            map.get(&Register::X1),
            Some(&AvailableValue::OriginalRegisterWithScalar(Register::X1, 0))
        );
        assert_eq!(
            map.get(&Register::X2),
            Some(&AvailableValue::OriginalRegisterWithScalar(
                Register::X2,
                18
            ))
        );
        assert_eq!(map.get(&Register::X3), None);
        assert_eq!(map.len(), 2);
    }
}
