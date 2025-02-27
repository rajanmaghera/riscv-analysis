use crate::parser::{Imm, InstructionProperties, LabelStringToken, Register, With};

use super::CfgNode;

impl InstructionProperties for CfgNode {
    fn is_return(&self) -> bool {
        self.node().is_return()
    }

    fn is_ureturn(&self) -> bool {
        self.node().is_ureturn()
    }

    fn stores_to_memory(&self) -> Option<(Register, (Register, Imm))> {
        self.node().stores_to_memory()
    }

    fn reads_from_memory(&self) -> Option<((Register, Imm), Register)> {
        self.node().reads_from_memory()
    }

    fn can_skip_save_checks(&self) -> bool {
        self.node().can_skip_save_checks()
    }

    fn calls_to(&self) -> Option<LabelStringToken> {
        self.node().calls_to()
    }

    fn is_ecall(&self) -> bool {
        self.node().is_ecall()
    }

    fn jumps_to(&self) -> Option<LabelStringToken> {
        self.node().jumps_to()
    }

    fn is_any_entry(&self) -> bool {
        self.node().is_any_entry()
    }

    fn is_function_entry(&self) -> bool {
        self.node().is_function_entry()
    }

    fn is_handler_function_entry(&self) -> bool {
        self.node().is_handler_function_entry()
    }

    fn is_program_entry(&self) -> bool {
        self.node().is_program_entry()
    }

    fn is_instruction(&self) -> bool {
        self.node().is_instruction()
    }

    fn uses_memory_location(&self) -> Option<(Register, Imm)> {
        self.node().uses_memory_location()
    }

    fn is_unconditional_jump(&self) -> bool {
        self.node().is_unconditional_jump()
    }

    fn is_some_jump_to_label(&self) -> Option<LabelStringToken> {
        self.node().is_some_jump_to_label()
    }

    fn writes_to(&self) -> Option<With<Register>> {
        self.node().writes_to()
    }

    fn reads_from(&self) -> std::collections::HashSet<With<Register>> {
        self.node().reads_from()
    }

    fn reads_address_of(&self) -> Option<LabelStringToken> {
        self.node().reads_address_of()
    }
}
