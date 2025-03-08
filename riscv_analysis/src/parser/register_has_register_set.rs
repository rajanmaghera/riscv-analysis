use crate::{cfg::RegisterSet, parser::Register};
use Register::{
    X0, X1, X10, X11, X12, X13, X14, X15, X16, X17, X18, X19, X2, X20, X21, X22, X23, X24, X25,
    X26, X27, X28, X29, X3, X30, X31, X4, X5, X6, X7, X8, X9,
};

use super::HasRegisterSets;

trait AsSet {
    fn set(self) -> RegisterSet;
}

impl<const N: usize> AsSet for [Register; N] {
    fn set(self) -> RegisterSet {
        self.into_iter().collect()
    }
}

impl HasRegisterSets for Register {
    fn program_args_set() -> RegisterSet {
        [X10, X11].set()
    }

    fn temporary_set() -> RegisterSet {
        [X5, X6, X7, X28, X29, X30, X31].set()
    }

    fn argument_set() -> RegisterSet {
        [X10, X11, X12, X13, X14, X15, X16, X17].set()
    }

    fn return_set() -> RegisterSet {
        Self::argument_set()
    }

    fn all_writable_set() -> RegisterSet {
        [
            X1, X2, X3, X4, X5, X6, X7, X8, X9, X10, X11, X12, X13, X14, X15, X16, X17, X18, X19,
            X20, X21, X22, X23, X24, X25, X26, X27, X28, X29, X30, X31,
        ]
        .set()
    }

    fn saved_set() -> RegisterSet {
        [X8, X9, X18, X19, X20, X21, X22, X23, X24, X25, X26, X27].set()
    }

    fn sp_ra_set() -> RegisterSet {
        [X2, X1].set()
    }

    fn return_addr_set() -> RegisterSet {
        [X1].set()
    }

    fn caller_saved_set() -> RegisterSet {
        Self::temporary_set() | Self::argument_set()
    }

    fn const_zero_set() -> RegisterSet {
        [X0].set()
    }

    fn callee_saved_set() -> RegisterSet {
        Self::saved_set() | Self::sp_ra_set()
    }

    fn ecall_always_argument_set() -> RegisterSet {
        [Register::ecall_type()].set()
    }
}
