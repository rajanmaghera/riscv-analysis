use crate::{cfg::RegisterSet, parser::Register};

// This file contains functions to convert register sets between bitmaps and
// hashsets. Hashsets allow for easy manipulation of register sets, while bitmaps
// are used for efficient algorithms.

pub struct RegSets;
impl RegSets {
    #[must_use]
    pub fn program_args() -> RegisterSet {
        vec![Register::X10, Register::X11].into_iter().collect()
    }

    #[must_use]
    pub fn temporary() -> RegisterSet {
        use Register::{X28, X29, X30, X31, X5, X6, X7};
        vec![X5, X6, X7, X28, X29, X30, X31].into_iter().collect()
    }

    #[must_use]
    pub fn argument() -> RegisterSet {
        use Register::{X10, X11, X12, X13, X14, X15, X16, X17};
        vec![X10, X11, X12, X13, X14, X15, X16, X17]
            .into_iter()
            .collect()
    }

    #[must_use]
    pub fn ret() -> RegisterSet {
        RegSets::argument()
    }

    #[must_use]
    pub fn saved() -> RegisterSet {
        use Register::{X18, X19, X20, X21, X22, X23, X24, X25, X26, X27, X8, X9};
        vec![X8, X9, X18, X19, X20, X21, X22, X23, X24, X25, X26, X27]
            .into_iter()
            .collect()
    }

    #[must_use]
    pub fn sp_ra() -> RegisterSet {
        use Register::{X1, X2};
        vec![X2, X1].into_iter().collect()
    }

    #[must_use]
    pub fn caller_saved() -> RegisterSet {
        let mut set = RegSets::temporary();
        set.extend(RegSets::argument());
        set
    }

    #[must_use]
    pub fn callee_saved() -> RegisterSet {
        let mut set = RegSets::saved();
        set.extend(RegSets::sp_ra());
        set
    }

    #[must_use]
    pub fn ecall_always_argument() -> RegisterSet {
        vec![Register::ecall_type()].into_iter().collect()
    }
}
