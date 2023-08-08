use std::collections::HashSet;

use crate::parser::Register;

// This file contains functions to convert register sets between bitmaps and
// hashsets. Hashsets allow for easy manipulation of register sets, while bitmaps
// are used for efficient algorithms.

pub trait ToRegBitmap {
    #[must_use]
    fn to_bitmap(&self) -> u32;
}

pub trait ToRegHashset {
    #[must_use]
    fn to_hashset(&self) -> HashSet<Register>;
}

impl<S: std::hash::BuildHasher> ToRegBitmap for HashSet<Register, S> {
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
                let num = Register::from_num(i);
                if let Ok(x) = num {
                    set.insert(x);
                }
            }
        }
        set
    }
}

pub struct RegSets;
impl RegSets {
    #[must_use]
    pub fn program_args() -> HashSet<Register> {
        vec![Register::X10, Register::X11].into_iter().collect()
    }

    #[must_use]
    pub fn temporary() -> HashSet<Register> {
        use Register::{X28, X29, X30, X31, X5, X6, X7};
        vec![X5, X6, X7, X28, X29, X30, X31].into_iter().collect()
    }

    #[must_use]
    pub fn argument() -> HashSet<Register> {
        use Register::{X10, X11, X12, X13, X14, X15, X16, X17};
        vec![X10, X11, X12, X13, X14, X15, X16, X17]
            .into_iter()
            .collect()
    }

    #[must_use]
    pub fn ret() -> HashSet<Register> {
        RegSets::argument()
    }

    #[must_use]
    pub fn saved() -> HashSet<Register> {
        use Register::{X18, X19, X20, X21, X22, X23, X24, X25, X26, X27, X8, X9};
        vec![X8, X9, X18, X19, X20, X21, X22, X23, X24, X25, X26, X27]
            .into_iter()
            .collect()
    }

    #[must_use]
    pub fn sp_ra() -> HashSet<Register> {
        use Register::{X1, X2};
        vec![X2, X1].into_iter().collect()
    }

    #[must_use]
    pub fn caller_saved() -> HashSet<Register> {
        let mut set = RegSets::temporary();
        set.extend(RegSets::argument());
        set
    }

    #[must_use]
    pub fn callee_saved() -> HashSet<Register> {
        let mut set = RegSets::saved();
        set.extend(RegSets::sp_ra());
        set
    }

    #[must_use]
    pub fn ecall_always_argument() -> HashSet<Register> {
        vec![Register::ecall_type()].into_iter().collect()
    }
}
