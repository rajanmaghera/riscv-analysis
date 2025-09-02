use crate::analysis::Value;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, Bound};
use std::ops::Range;

/// A compact memory range representation
///
/// A range represents 32-bit values repeated.
///
/// If a value does not exist in the physical maps, it is the
/// Undefined value. All values are undefined by default.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryRangeMap {
    ranges: BTreeMap<i32, (i32, Value)>,
}

impl PartialEq for MemoryRangeMap {
    fn eq(&self, other: &Self) -> bool {
        self.ranges.iter().eq(other.ranges.iter())
    }
}
impl Eq for MemoryRangeMap {}

impl Default for MemoryRangeMap {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryRangeMap {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ranges: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn get(&self, offset: i32) -> Value {
        if let Some((_, &(end_off, ref val))) = self.ranges.range(..=offset).next_back() {
            if offset < end_off {
                return *val;
            }
        }
        Value::Undefined
    }
    /// Ensure offset is aligned to 4 bytes
    fn assert_aligned(offset: i32) {
        assert!(
            !(offset % 4 != 0 && offset != i32::MAX),
            "Offset {offset:#X} is not aligned to 4 bytes"
        );
    }

    /// Insert range.
    ///
    /// # Panics
    ///
    /// Panics if range is empty or is not increasing
    pub fn insert_range(&mut self, offset: Range<i32>, val: Value) {
        assert!(
            offset.start < offset.end,
            "Range must be non-empty and increasing"
        );
        Self::assert_aligned(offset.start);
        Self::assert_aligned(offset.end);

        let start_key = offset.start;
        let end_key = offset.end;

        // Find the range immediately before or overlapping the start
        let preceding = self
            .ranges
            .range(..=start_key)
            .next_back()
            .map(|(k, v)| (*k, *v));

        // Possibly adjust overlapping previous range
        if let Some((prev_start, (prev_end, prev_val))) = preceding {
            if offset.start < prev_end {
                self.ranges.remove(&prev_start);

                // Preserve left portion if it exists
                if prev_start < offset.start {
                    self.ranges.insert(prev_start, (offset.start, prev_val));
                }

                // Preserve right portion if it exists
                if offset.end < prev_end {
                    self.ranges.insert(offset.end, (prev_end, prev_val));
                }
            }
        }

        // Remove all fully covered overlapping ranges
        let overlapping: Vec<_> = self
            .ranges
            .range(start_key..end_key)
            .map(|(k, _)| *k)
            .collect();

        for key in overlapping {
            self.ranges.remove(&key);
        }

        // Insert the new range
        self.ranges
            .insert(start_key, (offset.end, val.canonicalize()));

        // Merge adjacent ranges with the same value
        self.merge_neighbors();
        // TODO: I think this is broken
    }
    pub fn join_range(&mut self, offset: Range<i32>, val: Value) {
        // TODO: make more efficient
        // TODO: test
        let mut other_map = MemoryRangeMap::new();
        other_map.insert_range(offset, val);
        self.join(&other_map);
    }

    pub fn insert(&mut self, offset: i32, value: Value) {
        self.insert_range(offset..offset + 4, value);
    }

    fn merge_neighbors(&mut self) {
        let mut merged = BTreeMap::new();
        let mut iter = self.ranges.iter();

        // Get the first element in the range
        let Some(mut current) = iter.next().map(|(k, v)| (*k, *v)) else {
            return;
        };

        // Loop through the rest of the array
        for next in iter {
            // If this range can be merged, set the current's end to the next's end
            if current.1 .0 == *next.0 && current.1 .1 == next.1 .1 {
                current.1 .0 = next.1 .0;
            } else {
                // Otherwise, add the current to the list
                merged.insert(current.0, current.1);
                current = (*next.0, *next.1);
            }
        }
        merged.insert(current.0, current.1);
        self.ranges = merged
            .into_iter()
            .filter(|&(_, (_, ref v))| *v != Value::Undefined)
            .collect();
    }

    pub fn dump(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (idx, (&start, &(end, val))) in self.ranges.iter().enumerate() {
            write!(
                f,
                "{}R+[{:#010X}] +{:#0X} => {}",
                if idx == 0 { "" } else { " ; " },
                start,
                end as i64 - start as i64,
                val
            )?;
        }
        Ok(())
    }

    pub fn join(&mut self, other: &MemoryRangeMap) {
        // TODO : Test
        // TODO : fix for multiple registers

        let mut split_points: BTreeSet<i32> = BTreeSet::new();

        // Collect all split points (start and end of each range) from both maps
        // We need to insert the ends of the range as well as it could signify the
        // start of an undefined section
        for (start, (end, _)) in self.ranges.iter().chain(other.ranges.iter()) {
            split_points.insert(*start);
            split_points.insert(*end);
        }

        for start in split_points.iter().copied() {
            let end = split_points
                .range((Bound::Excluded(start), Bound::Included(i32::MAX)))
                .next()
                .copied()
                .unwrap_or(i32::MAX);
            if start == end {
                continue;
            }
            let self_val = self.get(start);
            let other_val = other.get(start);
            let new_val = self_val.join(other_val);
            self.insert_range(start..end, new_val);
        }
    }

    pub fn join_all_inner_values(&self) -> Value {
        self.ranges
            .iter()
            .map(|(_, (_, val))| *val)
            .reduce(Value::join)
            .unwrap_or(Value::Undefined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dump(map: &MemoryRangeMap) -> String {
        let mut s = String::new();
        map.dump(&mut s).unwrap();
        s
    }

    #[test]
    fn insert_single_const() {
        let mut mem = MemoryRangeMap::new();
        mem.insert(0x00, Value::Const(123));
        assert_eq!(dump(&mem), "R+[0x00000000] +0x4 => 123");
    }

    #[test]
    fn insert_single_const_negative() {
        let mut mem = MemoryRangeMap::new();
        mem.insert(-8, Value::Const(123));
        assert_eq!(dump(&mem), "R+[0xFFFFFFF8] +0x4 => 123");
    }

    #[test]
    fn insert_multiple_same_value_merge() {
        let mut mem = MemoryRangeMap::new();
        mem.insert(0x00, Value::Unknown);
        mem.insert(0x04, Value::Unknown);
        mem.insert(0x08, Value::Unknown);
        assert_eq!(dump(&mem), "R+[0x00000000] +0xC => T");
    }

    #[test]
    fn insert_different_values_no_merge() {
        let mut mem = MemoryRangeMap::new();
        mem.insert(0x00, Value::Unknown);
        mem.insert(0x04, Value::Const(1));
        mem.insert(0x08, Value::Unknown);
        assert_eq!(
            dump(&mem),
            "R+[0x00000000] +0x4 => T ; R+[0x00000004] +0x4 => 1 ; R+[0x00000008] +0x4 => T"
        );
    }

    #[test]
    fn insert_range_values() {
        let mut mem = MemoryRangeMap::new();
        mem.insert(0x00, Value::Unknown);
        mem.insert(0x04, Value::Const(1));
        mem.insert(0x08, Value::Unknown);
        assert_eq!(
            dump(&mem),
            "R+[0x00000000] +0x4 => T ; R+[0x00000004] +0x4 => 1 ; R+[0x00000008] +0x4 => T"
        );

        mem.insert_range(0..8, Value::Const(2));
        assert_eq!(
            dump(&mem),
            "R+[0x00000000] +0x8 => 2 ; R+[0x00000008] +0x4 => T"
        );
        mem.insert_range(4..12, Value::Const(3));
        assert_eq!(
            dump(&mem),
            "R+[0x00000000] +0x4 => 2 ; R+[0x00000004] +0x8 => 3"
        );
    }
    #[test]
    fn remove_middle_value_and_merge() {
        let mut mem = MemoryRangeMap::new();
        mem.insert(0x00, Value::Unknown);
        mem.insert(0x04, Value::Unknown);
        mem.insert(0x08, Value::Unknown);
        assert_eq!(dump(&mem), "R+[0x00000000] +0xC => T");

        mem.insert(0x04, Value::Const(42));
        assert_eq!(
            dump(&mem),
            "R+[0x00000000] +0x4 => T ; R+[0x00000004] +0x4 => 42 ; R+[0x00000008] +0x4 => T"
        );

        mem.insert(0x04, Value::Unknown); // Back to Unknown
        assert_eq!(dump(&mem), "R+[0x00000000] +0xC => T");

        mem.insert(0x04, Value::Undefined); // To undefined
        assert_eq!(
            dump(&mem),
            "R+[0x00000000] +0x4 => T ; R+[0x00000008] +0x4 => T"
        );
    }

    #[test]
    fn query_default_undefined() {
        let mem = MemoryRangeMap::new();
        assert_eq!(mem.get(0x00), Value::Undefined);
        assert_eq!(dump(&mem), "");
    }

    #[test]
    fn insert_and_get() {
        let mut mem = MemoryRangeMap::new();
        mem.insert(0x20, Value::Const(999));
        assert_eq!(mem.get(0x20), Value::Const(999));
        assert_eq!(dump(&mem), "R+[0x00000020] +0x4 => 999");
    }

    #[test]
    fn insert_and_overwrite() {
        let mut mem = MemoryRangeMap::new();
        mem.insert(0x00, Value::Unknown);
        mem.insert(0x00, Value::Const(5));
        assert_eq!(dump(&mem), "R+[0x00000000] +0x4 => 5");
    }

    #[test]
    fn insert_non_contiguous_values() {
        let mut mem = MemoryRangeMap::new();
        mem.insert(0x00, Value::Const(1));
        mem.insert(0x10, Value::Const(2));
        mem.insert(0x20, Value::Const(3));
        assert_eq!(
            dump(&mem),
            "R+[0x00000000] +0x4 => 1 ; R+[0x00000010] +0x4 => 2 ; R+[0x00000020] +0x4 => 3"
        );
    }

    #[test]
    fn remove_all_values() {
        let mut mem = MemoryRangeMap::new();
        for i in 0..4 {
            mem.insert(i * 4, Value::Const(i));
        }
        for i in 0..4 {
            mem.insert(i * 4, Value::Undefined);
        }
        assert_eq!(dump(&mem), "");
    }

    #[test]
    fn alignment_check_panics() {
        let result = std::panic::catch_unwind(|| {
            let mut mem = MemoryRangeMap::new();
            mem.insert(3, Value::Unknown); // Not aligned
        });
        assert!(result.is_err());
    }

    fn make_map(pairs: Vec<(i32, Value)>) -> MemoryRangeMap {
        let mut map = MemoryRangeMap::new();
        for (offset, val) in pairs {
            map.insert(offset, val);
        }
        map
    }

    #[test]
    fn join_with_equal_ranges() {
        let mut a = make_map(vec![(0, Value::Const(42))]);
        let b = make_map(vec![(0, Value::Const(42))]);
        a.join(&b);
        assert_eq!(a.get(0), Value::Const(42));
        assert_eq!(dump(&a), "R+[0x00000000] +0x4 => 42");
    }

    #[test]
    fn join_with_undefined_keeps_other_value() {
        let mut a = make_map(vec![(0, Value::Undefined)]);
        let b = make_map(vec![(0, Value::Const(99))]);
        a.join(&b);
        assert_eq!(a.get(0), Value::Const(99));
        assert_eq!(dump(&a), "R+[0x00000000] +0x4 => 99");
    }

    #[test]
    fn join_with_unknown_dominates() {
        let mut a = make_map(vec![(0, Value::Unknown)]);
        let b = make_map(vec![(0, Value::Const(1))]);
        a.join(&b);
        assert_eq!(a.get(0), Value::Unknown);
        assert_eq!(dump(&a), "R+[0x00000000] +0x4 => T");
    }

    #[test]
    fn join_const_conflict_becomes_unknownconst() {
        let mut a = make_map(vec![(0, Value::Const(1))]);
        let b = make_map(vec![(0, Value::Const(2))]);
        a.join(&b);
        assert_eq!(a.get(0), Value::UnknownConst);
        assert_eq!(dump(&a), "R+[0x00000000] +0x4 => Tc");
    }

    #[test]
    fn join_different_offsets_partial_overlap() {
        let mut a = make_map(vec![(0, Value::Const(10)), (4, Value::Const(20))]);
        let b = make_map(vec![(4, Value::Const(20)), (8, Value::Const(30))]);
        a.join(&b);

        assert_eq!(a.get(0), Value::Const(10));
        assert_eq!(a.get(4), Value::Const(20));
        assert_eq!(a.get(8), Value::Const(30));

        let expected =
            "R+[0x00000000] +0x4 => 10 ; R+[0x00000004] +0x4 => 20 ; R+[0x00000008] +0x4 => 30";
        assert_eq!(dump(&a), expected);
    }

    #[test]
    fn join_gap_filled_with_undefined() {
        let mut a = make_map(vec![(0, Value::Const(10)), (8, Value::Const(30))]);
        let b = make_map(vec![(4, Value::Const(20))]);
        a.join(&b);

        assert_eq!(a.get(0), Value::Const(10));
        assert_eq!(a.get(4), Value::Const(20));
        assert_eq!(a.get(8), Value::Const(30));

        let expected =
            "R+[0x00000000] +0x4 => 10 ; R+[0x00000004] +0x4 => 20 ; R+[0x00000008] +0x4 => 30";
        assert_eq!(dump(&a), expected);
    }

    #[test]
    fn join_creates_non_contiguous_blocks() {
        let mut a = make_map(vec![(0, Value::Const(1)), (4, Value::Const(2))]);
        let b = make_map(vec![(4, Value::Unknown)]);

        a.join(&b);
        assert_eq!(a.get(0), Value::Const(1));
        assert_eq!(a.get(4), Value::Unknown);

        let expected = "R+[0x00000000] +0x4 => 1 ; R+[0x00000004] +0x4 => T";
        assert_eq!(dump(&a), expected);
    }

    #[test]
    fn join_stack_pointer_with_other_returns_unknown() {
        let mut a = make_map(vec![(0, Value::InitialStackPointer(0))]);
        let b = make_map(vec![(0, Value::Const(10))]);
        a.join(&b);
        assert_eq!(a.get(0), Value::Unknown);
        assert_eq!(dump(&a), "R+[0x00000000] +0x4 => T");
    }

    #[test]
    fn join_merges_when_result_values_equal() {
        let mut a = make_map(vec![(0, Value::Const(42)), (4, Value::Const(42))]);
        let b = make_map(vec![(0, Value::Undefined), (4, Value::Undefined)]);
        a.join(&b);

        // After join, both ranges are still Const(42), so should be merged
        assert_eq!(a.get(0), Value::Const(42));
        assert_eq!(a.get(4), Value::Const(42));
        assert_eq!(dump(&a), "R+[0x00000000] +0x8 => 42");
    }
}
