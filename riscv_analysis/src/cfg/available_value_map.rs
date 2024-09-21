use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::hash::Hash;
use std::ops::{BitAndAssign, SubAssign};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{analysis::AvailableValue, parser::Register};

use super::RegisterSet;

/// A map from a generic item to an available value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableValueMap<T: PartialEq + Eq + Hash> {
    /// The map from the item to the available value.
    map: HashMap<T, AvailableValue>,
}

impl<T: PartialEq + Eq + Hash> AvailableValueMap<T> {
    /// Create a new `AvailableValueMap` with no values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Iterate over all values
    pub fn iter(&self) -> impl Iterator<Item = (&T, &AvailableValue)> {
        self.map.iter()
    }

    /// Get the available value for the given key.
    pub fn get(&self, item: &T) -> Option<&AvailableValue> {
        self.map.get(item)
    }

    /// Insert or replace the given item and available value into the map.
    pub fn insert(&mut self, key: T, value: AvailableValue) {
        self.map.insert(key, value);
    }

    /// Check if the available value map is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Get the number of items in the available value map.
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Extend the available value map with another available value map.
    pub fn extend(&mut self, other: Self) {
        self.map.extend(other.map);
    }
}

impl<T: PartialEq + Eq + Hash> IntoIterator for AvailableValueMap<T> {
    type Item = (T, AvailableValue);
    type IntoIter = std::collections::hash_map::IntoIter<T, AvailableValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<'a, T: PartialEq + Eq + Hash> IntoIterator for &'a AvailableValueMap<T> {
    type Item = (&'a T, &'a AvailableValue);
    type IntoIter = std::collections::hash_map::Iter<'a, T, AvailableValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
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

impl<T: PartialEq + Eq + Hash> Default for AvailableValueMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Since pure subtraction would require a clone we disable it for just `SubAssign`.

impl<T: Iterator<Item = Register>> SubAssign<T> for AvailableValueMap<Register> {
    fn sub_assign(&mut self, other: T) {
        for register in other {
            self.map.remove(&register);
        }
    }
}

impl<T: PartialEq + Eq + Hash> BitAndAssign<&AvailableValueMap<T>> for AvailableValueMap<T> {
    fn bitand_assign(&mut self, other: &AvailableValueMap<T>) {
        self.map
            .retain(|key, value| other.map.get(key) == Some(value));
    }
}

impl<T: PartialEq + Eq + Hash + Display> std::fmt::Display for AvailableValueMap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let values = self
            .map
            .iter()
            .map(|(reg, val)| format!("{reg}: {val}"))
            .sorted()
            .join(", ");
        write!(f, "[{values}]")
    }
}

impl<'a, T: PartialEq + Eq + Hash + Deserialize<'a>> Deserialize<'a> for AvailableValueMap<T> {
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

impl<T: PartialEq + Eq + Hash + Serialize + Ord> Serialize for AvailableValueMap<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.map
            .iter()
            .sorted_by_key(|(key, _)| *key)
            .collect::<BTreeMap<_, _>>()
            .serialize(serializer)
    }
}

impl<T: PartialEq + Eq + Hash> FromIterator<(T, AvailableValue)> for AvailableValueMap<T> {
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
                new.insert(*reg, value.clone());
            }
            new
        } else {
            self.clone()
        }
    }
}

impl<T: PartialEq + Eq + Hash + Display> AvailableValueMap<T> {
    #[must_use]
    pub fn str(&self) -> String {
        self.to_string()
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
                &Register::X1,
                &AvailableValue::OriginalRegisterWithScalar(Register::X1, 0)
            ))
        );
        assert_eq!(
            map_iter.next(),
            Some((&Register::X2, &AvailableValue::Constant(18)))
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
