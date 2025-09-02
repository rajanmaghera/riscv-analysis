use crate::new_impl::instruction_like::InstructionLike;
use crate::new_impl::isa::ISA;
use crate::new_impl::limited_element_set::LimitedElementSet;

pub trait CallingConventionISA: ISA {
    fn caller_saved_registers(
    ) -> <<<Self as ISA>::Instruction as InstructionLike>::Register as LimitedElementSet>::ArrayType;

    fn callee_saved_registers(
    ) -> <<<Self as ISA>::Instruction as InstructionLike>::Register as LimitedElementSet>::ArrayType;
    fn argument_registers(
    ) -> <<<Self as ISA>::Instruction as InstructionLike>::Register as LimitedElementSet>::ArrayType;
    fn return_registers(
    ) -> <<<Self as ISA>::Instruction as InstructionLike>::Register as LimitedElementSet>::ArrayType;
    fn program_argument_registers(
    ) -> <<<Self as ISA>::Instruction as InstructionLike>::Register as LimitedElementSet>::ArrayType;

    fn is_function_call_instruction(inst: &<Self as ISA>::Instruction) -> bool;
    fn is_function_return_instruction(inst: &<Self as ISA>::Instruction) -> bool;

    // Add interrupt handler stuff here
}
