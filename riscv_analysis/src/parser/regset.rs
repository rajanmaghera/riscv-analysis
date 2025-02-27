use crate::{cfg::RegisterSet, parser::Register};

pub struct RegSets;
impl RegSets {
    #[must_use]
    pub fn program_args() -> RegisterSet {
        use Register::{X10, X11};
        [X10, X11].into_iter().collect()
    }

    #[must_use]
    pub fn temporary() -> RegisterSet {
        use Register::{X28, X29, X30, X31, X5, X6, X7};
        [X5, X6, X7, X28, X29, X30, X31].into_iter().collect()
    }

    #[must_use]
    pub fn argument() -> RegisterSet {
        use Register::{X10, X11, X12, X13, X14, X15, X16, X17};
        [X10, X11, X12, X13, X14, X15, X16, X17]
            .into_iter()
            .collect()
    }

    #[must_use]
    pub fn ret() -> RegisterSet {
        RegSets::argument()
    }

    #[must_use]
    pub fn all() -> RegisterSet {
        use Register::{
            X1, X10, X11, X12, X13, X14, X15, X16, X17, X18, X19, X2, X20, X21, X22, X23, X24, X25,
            X26, X27, X28, X29, X3, X30, X31, X4, X5, X6, X7, X8, X9,
        };
        [
            X1, X2, X3, X4, X5, X6, X7, X8, X9, X10, X11, X12, X13, X14, X15, X16, X17, X18, X19,
            X20, X21, X22, X23, X24, X25, X26, X27, X28, X29, X30, X31,
        ]
        .into_iter()
        .collect()
    }

    #[must_use]
    pub fn saved() -> RegisterSet {
        use Register::{X18, X19, X20, X21, X22, X23, X24, X25, X26, X27, X8, X9};
        [X8, X9, X18, X19, X20, X21, X22, X23, X24, X25, X26, X27]
            .into_iter()
            .collect()
    }

    #[must_use]
    pub fn sp_ra() -> RegisterSet {
        use Register::{X1, X2};
        [X2, X1].into_iter().collect()
    }

    #[must_use]
    pub fn return_addr() -> RegisterSet {
        use Register::X1;
        [X1].into_iter().collect()
    }

    #[must_use]
    pub fn caller_saved() -> RegisterSet {
        RegSets::temporary() | RegSets::argument()
    }

    #[must_use]
    pub fn callee_saved() -> RegisterSet {
        RegSets::saved() | RegSets::sp_ra()
    }

    #[must_use]
    pub fn ecall_always_argument() -> RegisterSet {
        [Register::ecall_type()].into_iter().collect()
    }
}
