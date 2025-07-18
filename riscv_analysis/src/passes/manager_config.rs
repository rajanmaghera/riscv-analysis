use crate::lints::{
    CalleeSavedGarbageReadPassConfiguration, CalleeSavedRegisterPassConfiguration,
    ControlFlowPassConfiguration, DeadValuePassConfiguration, DotCFGGenerationPassConfiguration,
    EcallPassConfiguration, GarbageInputValuePassConfiguration, InstructionInTextPassConfiguration,
    LostCalleeSavedRegisterPassConfiguration, OverlappingFunctionPassConfiguration,
    SaveToZeroPassConfiguration, StackPassConfiguration,
};

#[derive(Default)]
pub struct ManagerConfiguration {
    callee_saved_garbage_read: CalleeSavedGarbageReadPassConfiguration,
    callee_saved_register: CalleeSavedRegisterPassConfiguration,
    control_flow: ControlFlowPassConfiguration,
    dead_value: DeadValuePassConfiguration,
    dot_cfg_generation: DotCFGGenerationPassConfiguration,
    ecall: EcallPassConfiguration,
    garbage_input_value: GarbageInputValuePassConfiguration,
    instruction_in_text: InstructionInTextPassConfiguration,
    lost_callee_saved_register: LostCalleeSavedRegisterPassConfiguration,
    overlapping_function: OverlappingFunctionPassConfiguration,
    save_to_zero: SaveToZeroPassConfiguration,
    stack: StackPassConfiguration,
}
impl ManagerConfiguration {
    // CalleeSavedGarbageReadPass
    #[must_use]
    pub fn get_callee_saved_garbage_read_pass_config(
        &self,
    ) -> &CalleeSavedGarbageReadPassConfiguration {
        &self.callee_saved_garbage_read
    }
    pub fn get_mut_callee_saved_garbage_read_pass_config(
        &mut self,
    ) -> &mut CalleeSavedGarbageReadPassConfiguration {
        &mut self.callee_saved_garbage_read
    }

    // CalleeSavedRegisterPass
    #[must_use]
    pub fn get_callee_saved_register_pass_config(&self) -> &CalleeSavedRegisterPassConfiguration {
        &self.callee_saved_register
    }
    pub fn get_mut_callee_saved_register_pass_config(
        &mut self,
    ) -> &mut CalleeSavedRegisterPassConfiguration {
        &mut self.callee_saved_register
    }

    // ControlFlowPass
    #[must_use]
    pub fn get_control_flow_pass_config(&self) -> &ControlFlowPassConfiguration {
        &self.control_flow
    }
    pub fn get_mut_control_flow_pass_config(&mut self) -> &mut ControlFlowPassConfiguration {
        &mut self.control_flow
    }

    // DeadValuePass
    #[must_use]
    pub fn get_dead_value_pass_config(&self) -> &DeadValuePassConfiguration {
        &self.dead_value
    }
    pub fn get_mut_dead_value_pass_config(&mut self) -> &mut DeadValuePassConfiguration {
        &mut self.dead_value
    }

    // DotCFGGenerationPass
    #[must_use]
    pub fn get_dot_cfg_generation_pass_config(&self) -> &DotCFGGenerationPassConfiguration {
        &self.dot_cfg_generation
    }
    pub fn get_mut_dot_cfg_generation_pass_config(
        &mut self,
    ) -> &mut DotCFGGenerationPassConfiguration {
        &mut self.dot_cfg_generation
    }

    // EcallPass
    #[must_use]
    pub fn get_ecall_pass_config(&self) -> &EcallPassConfiguration {
        &self.ecall
    }
    pub fn get_mut_ecall_pass_config(&mut self) -> &mut EcallPassConfiguration {
        &mut self.ecall
    }

    // GarbageInputValuePass
    #[must_use]
    pub fn get_garbage_input_value_pass_config(&self) -> &GarbageInputValuePassConfiguration {
        &self.garbage_input_value
    }
    pub fn get_mut_garbage_input_value_pass_config(
        &mut self,
    ) -> &mut GarbageInputValuePassConfiguration {
        &mut self.garbage_input_value
    }

    // InstructionInTextPass
    #[must_use]
    pub fn get_instruction_in_text_pass_config(&self) -> &InstructionInTextPassConfiguration {
        &self.instruction_in_text
    }
    pub fn get_mut_instruction_in_text_pass_config(
        &mut self,
    ) -> &mut InstructionInTextPassConfiguration {
        &mut self.instruction_in_text
    }

    // LostCalleeSavedRegisterPass
    #[must_use]
    pub fn get_lost_callee_saved_register_pass_config(
        &self,
    ) -> &LostCalleeSavedRegisterPassConfiguration {
        &self.lost_callee_saved_register
    }
    pub fn get_mut_lost_callee_saved_register_pass_config(
        &mut self,
    ) -> &mut LostCalleeSavedRegisterPassConfiguration {
        &mut self.lost_callee_saved_register
    }

    // OverlappingFunctionPass
    #[must_use]
    pub fn get_overlapping_function_pass_config(&self) -> &OverlappingFunctionPassConfiguration {
        &self.overlapping_function
    }
    pub fn get_mut_overlapping_function_pass_config(
        &mut self,
    ) -> &mut OverlappingFunctionPassConfiguration {
        &mut self.overlapping_function
    }

    // SaveToZeroPass
    #[must_use]
    pub fn get_save_to_zero_pass_config(&self) -> &SaveToZeroPassConfiguration {
        &self.save_to_zero
    }
    pub fn get_mut_save_to_zero_pass_config(&mut self) -> &mut SaveToZeroPassConfiguration {
        &mut self.save_to_zero
    }

    // StackPass
    #[must_use]
    pub fn get_stack_pass_config(&self) -> &StackPassConfiguration {
        &self.stack
    }
    pub fn get_mut_stack_pass_config(&mut self) -> &mut StackPassConfiguration {
        &mut self.stack
    }
}

pub trait ToManagerConfiguration {
    fn to_manager_configuration(&self) -> ManagerConfiguration;
}
