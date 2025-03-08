use std::collections::HashSet;

use super::{Imm, LabelStringToken, Register, RegisterToken};

pub trait InstructionProperties {
    fn is_return(&self) -> bool;

    /// Checks if an insturction might terminate the program.
    ///
    /// This is used to determine if a node might be a terminating instruction.
    /// Some forms of environment calls might terminate the program. In early
    /// stages, we might not know if a function will or will not terminate, thus
    /// this function is used.
    fn might_terminate(&self) -> bool;

    fn is_ureturn(&self) -> bool;

    fn stores_to_memory(&self) -> Option<(Register, (Register, Imm))>;

    fn reads_from_memory(&self) -> Option<((Register, Imm), Register)>;

    /// Checks if a instruction is meant to be saved to zero
    ///
    /// Some instructions save to zero as part of their design. For example,
    /// jumps that link to zero. However, some have no effect even while
    /// saving to zero. For example, `addi x0, x0, 0` is a no-op.
    /// This function determines if an instruction is meant to be saved to zero
    /// or if it is a no-op. No-ops are treated as warnings, not errors.
    #[must_use]
    fn can_skip_save_checks(&self) -> bool;

    /// Checks if a instruction is a function call
    #[must_use]
    fn calls_to(&self) -> Option<LabelStringToken>;

    /// Checks if a instruction is an environment call
    #[must_use]
    fn is_ecall(&self) -> bool;

    /// Checks if a instruction is a potential jump
    #[must_use]
    fn jumps_to(&self) -> Option<LabelStringToken>;

    #[must_use]
    fn is_any_entry(&self) -> bool;

    #[must_use]
    fn is_function_entry(&self) -> bool;

    #[must_use]
    fn is_handler_function_entry(&self) -> bool;

    #[must_use]
    fn is_program_entry(&self) -> bool;

    /// Check if a node is an instruction.
    #[must_use]
    fn is_instruction(&self) -> bool;

    /// Either loads or stores to a memory location
    #[must_use]
    fn uses_memory_location(&self) -> Option<(Register, Imm)>;

    /// Checks whether a jump is unconditional with no side effects
    ///
    /// Some jumps have side effects, like jumping to a function which sets
    /// the return address. This function checks if a jump is unconditional
    /// and has no side effects.
    #[must_use]
    fn is_unconditional_jump(&self) -> bool;

    /// Checks whether this is some jump to a known label with no side effects.
    #[must_use]
    fn is_some_jump_to_label(&self) -> Option<LabelStringToken>;

    /// Checks whether this instruction writes to a register, and which register it writes to.
    #[must_use]
    fn writes_to(&self) -> Option<RegisterToken>;

    /// Checks whether this instruction reads from a register, and which registers it reads from.
    #[must_use]
    fn reads_from(&self) -> HashSet<RegisterToken>;

    fn reads_address_of(&self) -> Option<LabelStringToken>;
}
