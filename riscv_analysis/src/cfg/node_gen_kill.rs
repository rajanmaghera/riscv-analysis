use crate::analysis::HasGenKillInfo;

use super::{CfgNode, RegisterSet};

impl HasGenKillInfo for CfgNode {
    fn kill_reg(&self) -> RegisterSet {
        self.node().kill_reg()
    }

    fn gen_reg(&self) -> RegisterSet {
        self.node().gen_reg()
    }
}
