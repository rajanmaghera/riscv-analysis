use crate::{cfg::RegisterSet, parser::Register};

use super::{AvailableValue, MemoryLocation};

pub trait HasGenKillInfo {
    #[must_use]
    fn kill_reg(&self) -> RegisterSet;

    #[must_use]
    fn gen_reg(&self) -> RegisterSet;
}

pub trait HasGenValueInfo {
    #[must_use]
    fn gen_memory_value(&self) -> Option<(MemoryLocation, AvailableValue)>;

    #[must_use]
    fn gen_reg_value(&self) -> Option<(Register, AvailableValue)>;
}
