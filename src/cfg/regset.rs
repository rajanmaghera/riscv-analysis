use std::collections::HashSet;

use crate::parser::register::Register;

// This file contains functions to convert register sets between bitmaps and
// hashsets. Hashsets allow for easy manipulation of register sets, while bitmaps
// are used for efficient algorithms.

// TODO in the future, we should use a set of macros that efficiently generate
// the register sets

// TODO static sets should be generated at compile time

pub trait ToRegBitmap {
    fn to_bitmap(&self) -> u32;
}

pub trait ToRegHashset {
    fn to_hashset(&self) -> HashSet<Register>;
}

impl ToRegBitmap for HashSet<Register> {
    fn to_bitmap(&self) -> u32 {
        let mut bitmap = 0;
        for reg in self {
            bitmap |= 1 << reg.to_num();
        }
        bitmap
    }
}

impl ToRegHashset for u32 {
    fn to_hashset(&self) -> HashSet<Register> {
        let mut set = HashSet::new();
        for i in 0..32 {
            if self & (1 << i) != 0 {
                set.insert(Register::from_num(i));
            }
        }
        set
    }
}

