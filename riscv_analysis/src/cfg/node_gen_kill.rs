use crate::{
    analysis::{AvailableValue, HasGenKillInfo, HasGenValueInfo, MemoryLocation},
    parser::Register,
};

use super::{CfgNode, RegisterSet};

impl HasGenKillInfo for CfgNode {
    fn kill_reg(&self) -> RegisterSet {
        self.node().kill_reg()
    }

    fn gen_reg(&self) -> RegisterSet {
        self.node().gen_reg()
    }
}

impl HasGenValueInfo for CfgNode {
    fn gen_memory_value(&self) -> Option<(MemoryLocation, AvailableValue)> {
        self.node().gen_memory_value()
    }

    fn gen_reg_value(&self) -> Option<(Register, AvailableValue)> {
        self.node().gen_reg_value()
    }
}
