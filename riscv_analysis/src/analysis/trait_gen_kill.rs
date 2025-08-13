use crate::cfg::RegisterSet;

pub trait HasGenKillInfo {
    #[must_use]
    fn kill_reg(&self) -> RegisterSet;

    #[must_use]
    fn gen_reg(&self) -> RegisterSet;
}
