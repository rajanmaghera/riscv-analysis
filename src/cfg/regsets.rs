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
        convert_to_bitmap(self.clone())
    }
}

impl ToRegHashset for u32 {
    fn to_hashset(&self) -> HashSet<Register> {
        convert_to_hashset(*self)
    }
}

fn convert_to_hashset(bitmap: u32) -> HashSet<Register> {
    let mut set = HashSet::new();
    for i in 0..32 {
        if bitmap & (1 << i) != 0 {
            set.insert(Register::from_num(i));
        }
    }
    set
}

fn convert_to_bitmap(set: HashSet<Register>) -> u32 {
    let mut bitmap = 0;
    for reg in set {
        bitmap |= 1 << reg.to_num();
    }
    bitmap
}

pub fn caller_saved_registers() -> u32 {
    convert_to_bitmap(
        vec![
            Register::X10,
            Register::X11,
            Register::X12,
            Register::X13,
            Register::X14,
            Register::X15,
            Register::X16,
            Register::X17,
            Register::X5,
            Register::X6,
            Register::X7,
            Register::X28,
            Register::X29,
            Register::X30,
            Register::X31,
        ]
        .into_iter()
        .collect(),
    )
}

fn argument_registers() -> HashSet<Register> {
    vec![
        Register::X10,
        Register::X11,
        Register::X12,
        Register::X13,
        Register::X14,
        Register::X15,
        Register::X16,
        Register::X17,
    ]
    .into_iter()
    .collect()
}
