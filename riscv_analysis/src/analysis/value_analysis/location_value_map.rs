use crate::analysis::{Location, MemoryRangeMap, Value};
use crate::parser::RVRegister;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fmt::Display;
use std::ops::Range;

/// A compact in-memory representation for value analysis.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LocValueMap {
    register_map: BTreeMap<RVRegister, Value>,
    memory_range_map: MemoryRangeMap,
}

impl Default for LocValueMap {
    fn default() -> Self {
        Self::new()
    }
}

impl LocValueMap {
    #[must_use]
    pub fn new() -> Self {
        Self {
            register_map: BTreeMap::from_iter([(RVRegister::X0, Value::Const(0))]),
            memory_range_map: MemoryRangeMap::new(),
        }
    }

    #[must_use]
    pub fn get(&self, location: &Location) -> Value {
        match location {
            Location::Register(r) => self
                .register_map
                .get(r)
                .copied()
                .unwrap_or(Value::Undefined),
            Location::StackPointerOffset32(offset) => self.memory_range_map.get(*offset),
        }
    }
    pub fn insert(&mut self, location: &Location, value: Value) {
        match location {
            Location::Register(r) => {
                if r != &RVRegister::X0 {
                    if value == Value::Undefined {
                        self.register_map.remove(r);
                    } else {
                        self.register_map.insert(*r, value.canonicalize());
                    }
                }
            }
            Location::StackPointerOffset32(offset) => {
                self.memory_range_map.insert(*offset, value);
            }
        }
    }

    pub fn insert_memory_range_into_stack(&mut self, offset_range: Range<i32>, value: Value) {
        self.memory_range_map.insert_range(offset_range, value);
    }

    pub fn join_memory_range_into_stack(&mut self, offset_range: Range<i32>, value: Value) {
        self.memory_range_map.join_range(offset_range, value);
    }

    pub fn dump(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (reg, value) in &self.register_map {
            write!(f, "{reg} => {value};")?;
        }
        self.memory_range_map.dump(f)?;
        Ok(())
    }

    // TODO: Test this function

    /// Join for when we only need to join register values
    pub fn join_registers(&mut self, others: impl IntoIterator<Item = (RVRegister, Value)>) {
        for (reg, value) in others {
            self.register_map
                .entry(reg)
                .and_modify(|v| *v = v.join(value))
                .or_insert(value);
        }
    }

    pub fn join(&mut self, other: &LocValueMap) {
        // Find keys in both
        let keys_a: HashSet<_> = self.register_map.keys().copied().collect();
        let keys_b: HashSet<_> = other.register_map.keys().copied().collect();

        // Split into three sets
        // Don't need to do anything to those only in a (undef in b)
        let in_both = keys_a.intersection(&keys_b);
        let only_in_b = keys_b.difference(&keys_a);

        // For the keys in both, call join
        for key in in_both {
            if let (Some(a), Some(b)) = (self.register_map.get(key), other.register_map.get(key)) {
                self.register_map.insert(*key, a.join(*b));
            }
        }

        // For keys in only_b, extend to a
        for key in only_in_b {
            if let Some(b) = other.register_map.get(key) {
                self.register_map.insert(*key, *b);
            }
        }

        // Join memory map
        self.memory_range_map.join(&other.memory_range_map);
    }
}

#[cfg(test)]
mod jointests {
    use super::*;

    #[test]
    fn join_works_both_ways() {
        let map_a = LocValueMap::from_iter([
            (RVRegister::X1, Value::Const(1)),
            (RVRegister::X2, Value::Const(2)),
            (RVRegister::X3, Value::Const(3)),
        ]);
        let map_b = LocValueMap::from_iter([(RVRegister::X1, Value::Const(1))]);
        let mut tmp_map_a = map_a.clone();
        tmp_map_a.join(&map_b);
        assert_eq!(tmp_map_a, map_a);
        assert_eq!(tmp_map_a.to_string(), "ra => 1;sp => 2;gp => 3;");
        let mut tmp_map_b = map_b.clone();
        tmp_map_b.join(&map_a);
        assert_eq!(tmp_map_b, map_a);
        assert_eq!(tmp_map_b.to_string(), "ra => 1;sp => 2;gp => 3;");
    }
}

impl Display for LocValueMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.dump(f)
    }
}

impl FromIterator<(RVRegister, Value)> for LocValueMap {
    fn from_iter<I: IntoIterator<Item = (RVRegister, Value)>>(iter: I) -> Self {
        Self {
            register_map: iter
                .into_iter()
                .filter(|&(reg, _)| reg != RVRegister::X0)
                .map(|(reg, value)| (reg, value.canonicalize()))
                .collect(),
            memory_range_map: MemoryRangeMap::new(),
        }
    }
}
