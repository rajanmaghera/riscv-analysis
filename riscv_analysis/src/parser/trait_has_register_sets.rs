use crate::cfg::RegisterSet;

use super::RegisterProperties;

pub trait HasRegisterSets: RegisterProperties {
    #[must_use]
    fn program_args_set() -> RegisterSet;

    #[must_use]
    fn temporary_set() -> RegisterSet;

    #[must_use]
    fn argument_set() -> RegisterSet;

    #[must_use]
    fn return_set() -> RegisterSet;

    #[must_use]
    fn all_writable_set() -> RegisterSet;

    #[must_use]
    fn saved_set() -> RegisterSet;

    #[must_use]
    fn sp_ra_set() -> RegisterSet;

    #[must_use]
    fn return_addr_set() -> RegisterSet;

    #[must_use]
    fn caller_saved_set() -> RegisterSet;

    #[must_use]
    fn callee_saved_set() -> RegisterSet;

    #[must_use]
    fn ecall_always_argument_set() -> RegisterSet;

    #[must_use]
    fn const_zero_set() -> RegisterSet;
}
