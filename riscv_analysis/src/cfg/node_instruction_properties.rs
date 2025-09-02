use crate::parser::{
    Imm, InstructionProperties, JumpTarget, LabelStringToken, RVRegister, RegisterToken,
};
use std::collections::HashSet;

use super::CfgNode;

impl InstructionProperties for CfgNode {
    fn is_return(&self) -> bool {
        self.node().is_return()
    }

    fn might_terminate(&self) -> bool {
        self.node().might_terminate()
    }

    fn is_ureturn(&self) -> bool {
        self.node().is_ureturn()
    }

    fn stores_to_memory(&self) -> Option<(RVRegister, (RVRegister, Imm))> {
        self.node().stores_to_memory()
    }

    fn reads_from_memory(&self) -> Option<((RVRegister, Imm), RVRegister)> {
        self.node().reads_from_memory()
    }

    fn can_skip_save_checks(&self) -> bool {
        self.node().can_skip_save_checks()
    }

    fn calls_to(&self) -> Option<JumpTarget> {
        self.node().calls_to()
    }

    fn is_ecall(&self) -> bool {
        self.node().is_ecall()
    }

    fn jumps_to(&self) -> Option<JumpTarget> {
        self.node().jumps_to()
    }

    fn is_instruction(&self) -> bool {
        self.node().is_instruction()
    }

    fn uses_memory_location(&self) -> Option<(RVRegister, Imm)> {
        self.node().uses_memory_location()
    }

    fn is_unconditional_jump(&self) -> bool {
        self.node().is_unconditional_jump()
    }

    fn writes_to(&self) -> HashSet<RegisterToken> {
        self.node().writes_to()
    }

    fn reads_from(&self) -> std::collections::HashSet<RegisterToken> {
        self.node().reads_from()
    }

    fn reads_address_of(&self) -> Option<LabelStringToken> {
        self.node().reads_address_of()
    }

    fn is_indirect_uncond_jump(&self) -> bool {
        self.node().is_indirect_uncond_jump()
    }

    fn is_direct_uncond_jump(&self) -> bool {
        self.node().is_direct_uncond_jump()
    }

    fn is_cond_branch(&self) -> bool {
        self.node().is_cond_branch()
    }
}
