use crate::lints::{
    CalleeSavedGarbageReadPassConfiguration, CalleeSavedRegisterPassConfiguration,
    ControlFlowPassConfiguration, DeadValuePassConfiguration, DotCFGGenerationPassConfiguration,
    EcallPassConfiguration, GarbageInputValuePassConfiguration, InstructionInTextPassConfiguration,
    LostCalleeSavedRegisterPassConfiguration, OverlappingFunctionPassConfiguration,
    SaveToZeroPassConfiguration, StackPassConfiguration,
};

#[derive(Default)]
pub struct ManagerConfiguration {
    callee_saved_garbage_read_pass_config: CalleeSavedGarbageReadPassConfiguration,
    callee_saved_register_pass_config: CalleeSavedRegisterPassConfiguration,
    control_flow_pass_config: ControlFlowPassConfiguration,
    dead_value_pass_config: DeadValuePassConfiguration,
    dot_cfg_generation_pass_config: DotCFGGenerationPassConfiguration,
    ecall_pass_config: EcallPassConfiguration,
    garbage_input_value_pass_config: GarbageInputValuePassConfiguration,
    instruction_in_text_pass_config: InstructionInTextPassConfiguration,
    lost_callee_saved_register_pass_config: LostCalleeSavedRegisterPassConfiguration,
    overlapping_function_pass_config: OverlappingFunctionPassConfiguration,
    save_to_zero_pass_config: SaveToZeroPassConfiguration,
    stack_pass_config: StackPassConfiguration,
}
impl ManagerConfiguration {
    // CalleeSavedGarbageReadPass
    #[must_use]
    pub fn get_callee_saved_garbage_read_pass_config(
        &self,
    ) -> &CalleeSavedGarbageReadPassConfiguration {
        &self.callee_saved_garbage_read_pass_config
    }
    pub fn get_mut_callee_saved_garbage_read_pass_config(
        &mut self,
    ) -> &mut CalleeSavedGarbageReadPassConfiguration {
        &mut self.callee_saved_garbage_read_pass_config
    }

    // CalleeSavedRegisterPass
    #[must_use]
    pub fn get_callee_saved_register_pass_config(&self) -> &CalleeSavedRegisterPassConfiguration {
        &self.callee_saved_register_pass_config
    }
    pub fn get_mut_callee_saved_register_pass_config(
        &mut self,
    ) -> &mut CalleeSavedRegisterPassConfiguration {
        &mut self.callee_saved_register_pass_config
    }

    // ControlFlowPass
    #[must_use]
    pub fn get_control_flow_pass_config(&self) -> &ControlFlowPassConfiguration {
        &self.control_flow_pass_config
    }
    pub fn get_mut_control_flow_pass_config(&mut self) -> &mut ControlFlowPassConfiguration {
        &mut self.control_flow_pass_config
    }

    // DeadValuePass
    #[must_use]
    pub fn get_dead_value_pass_config(&self) -> &DeadValuePassConfiguration {
        &self.dead_value_pass_config
    }
    pub fn get_mut_dead_value_pass_config(&mut self) -> &mut DeadValuePassConfiguration {
        &mut self.dead_value_pass_config
    }

    // DotCFGGenerationPass
    #[must_use]
    pub fn get_dot_cfg_generation_pass_config(&self) -> &DotCFGGenerationPassConfiguration {
        &self.dot_cfg_generation_pass_config
    }
    pub fn get_mut_dot_cfg_generation_pass_config(
        &mut self,
    ) -> &mut DotCFGGenerationPassConfiguration {
        &mut self.dot_cfg_generation_pass_config
    }

    // EcallPass
    #[must_use]
    pub fn get_ecall_pass_config(&self) -> &EcallPassConfiguration {
        &self.ecall_pass_config
    }
    pub fn get_mut_ecall_pass_config(&mut self) -> &mut EcallPassConfiguration {
        &mut self.ecall_pass_config
    }

    // GarbageInputValuePass
    #[must_use]
    pub fn get_garbage_input_value_pass_config(&self) -> &GarbageInputValuePassConfiguration {
        &self.garbage_input_value_pass_config
    }
    pub fn get_mut_garbage_input_value_pass_config(
        &mut self,
    ) -> &mut GarbageInputValuePassConfiguration {
        &mut self.garbage_input_value_pass_config
    }

    // InstructionInTextPass
    #[must_use]
    pub fn get_instruction_in_text_pass_config(&self) -> &InstructionInTextPassConfiguration {
        &self.instruction_in_text_pass_config
    }
    pub fn get_mut_instruction_in_text_pass_config(
        &mut self,
    ) -> &mut InstructionInTextPassConfiguration {
        &mut self.instruction_in_text_pass_config
    }

    // LostCalleeSavedRegisterPass
    #[must_use]
    pub fn get_lost_callee_saved_register_pass_config(
        &self,
    ) -> &LostCalleeSavedRegisterPassConfiguration {
        &self.lost_callee_saved_register_pass_config
    }
    pub fn get_mut_lost_callee_saved_register_pass_config(
        &mut self,
    ) -> &mut LostCalleeSavedRegisterPassConfiguration {
        &mut self.lost_callee_saved_register_pass_config
    }

    // OverlappingFunctionPass
    #[must_use]
    pub fn get_overlapping_function_pass_config(&self) -> &OverlappingFunctionPassConfiguration {
        &self.overlapping_function_pass_config
    }
    pub fn get_mut_overlapping_function_pass_config(
        &mut self,
    ) -> &mut OverlappingFunctionPassConfiguration {
        &mut self.overlapping_function_pass_config
    }

    // SaveToZeroPass
    #[must_use]
    pub fn get_save_to_zero_pass_config(&self) -> &SaveToZeroPassConfiguration {
        &self.save_to_zero_pass_config
    }
    pub fn get_mut_save_to_zero_pass_config(&mut self) -> &mut SaveToZeroPassConfiguration {
        &mut self.save_to_zero_pass_config
    }

    // StackPass
    #[must_use]
    pub fn get_stack_pass_config(&self) -> &StackPassConfiguration {
        &self.stack_pass_config
    }
    pub fn get_mut_stack_pass_config(&mut self) -> &mut StackPassConfiguration {
        &mut self.stack_pass_config
    }
}

pub trait ToManagerConfiguration {
    fn to_manager_configuration(&self) -> ManagerConfiguration;
}
