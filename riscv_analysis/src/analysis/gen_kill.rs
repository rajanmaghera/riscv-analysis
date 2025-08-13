use super::HasGenKillInfo;
use crate::{
    cfg::RegisterSet,
    parser::{InstructionProperties, RVInstructionNode, RegisterProperties},
};

impl HasGenKillInfo for RVInstructionNode {
    fn kill_reg(&self) -> RegisterSet {
        self.writes_to()
            .into_iter()
            .map(|x| *x.get())
            .filter(|x| !x.is_const_zero())
            .collect()
    }

    fn gen_reg(&self) -> RegisterSet {
        self.reads_from()
            .into_iter()
            .map(|x| *x.get())
            .filter(|x| !x.is_const_zero())
            .collect()
    }
}
