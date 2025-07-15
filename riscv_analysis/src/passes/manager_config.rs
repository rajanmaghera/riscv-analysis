use crate::{lints::{
    CalleeSavedGarbageReadCheckConfiguration,
    CalleeSavedRegisterCheckConfiguration,
    ControlFlowCheckConfiguration,
    DeadValueCheckConfiguration,
    DotCFGGenerationPassConfiguration,
    EcallCheckConfiguration,
    GarbageInputValueCheckConfiguration,
    InstructionInTextCheckConfiguration,
    LostCalleeSavedRegisterCheckConfiguration,
    OverlappingFunctionCheckConfiguration,
    SaveToZeroCheckConfiguration,
    StackCheckPassConfiguration,
}, passes::Manager};

#[derive(Default)]
pub struct ManagerConfiguration {
    callee_saved_garbage_read_check_config: CalleeSavedGarbageReadCheckConfiguration,
    callee_saved_register_check_config: CalleeSavedRegisterCheckConfiguration,
    control_flow_check_config: ControlFlowCheckConfiguration,
    dead_value_check_config: DeadValueCheckConfiguration,
    dot_cfg_generation_pass_config: DotCFGGenerationPassConfiguration,
    ecall_check_config: EcallCheckConfiguration,
    garbage_input_value_check_config: GarbageInputValueCheckConfiguration,
    instruction_in_text_check_config: InstructionInTextCheckConfiguration,
    lost_callee_saved_register_check_config: LostCalleeSavedRegisterCheckConfiguration,
    overlapping_function_check_config: OverlappingFunctionCheckConfiguration,
    save_to_zero_check_config: SaveToZeroCheckConfiguration,
    stack_check_pass_config: StackCheckPassConfiguration,
}
impl ManagerConfiguration {
    // CalleeSavedGarbageReadCheck
    pub fn get_callee_saved_garbage_read_check_config(&self) -> &CalleeSavedGarbageReadCheckConfiguration {
        &self.callee_saved_garbage_read_check_config
    }
    pub fn get_mut_callee_saved_garbage_read_check_config(&mut self) -> &mut CalleeSavedGarbageReadCheckConfiguration {
        &mut self.callee_saved_garbage_read_check_config
    }

    // CalleeSavedRegisterCheck
    pub fn get_callee_saved_register_check_config(&self) -> &CalleeSavedRegisterCheckConfiguration {
        &self.callee_saved_register_check_config
    }
    pub fn get_mut_callee_saved_register_check_config(&mut self) -> &mut CalleeSavedRegisterCheckConfiguration {
        &mut self.callee_saved_register_check_config
    }

    // ControlFlowCheck
    pub fn get_control_flow_check_config(&self) -> &ControlFlowCheckConfiguration {
        &self.control_flow_check_config
    }
    pub fn get_mut_control_flow_check_config(&mut self) -> &mut ControlFlowCheckConfiguration {
        &mut self.control_flow_check_config
    }

    // DeadValueCheck
    pub fn get_dead_value_check_config(&self) -> &DeadValueCheckConfiguration {
        &self.dead_value_check_config
    }
    pub fn get_mut_dead_value_check_config(&mut self) -> &mut DeadValueCheckConfiguration {
        &mut self.dead_value_check_config
    }

    // DotCFGGenerationPass
    pub fn get_dot_cfg_generation_pass_config(&self) -> &DotCFGGenerationPassConfiguration {
        &self.dot_cfg_generation_pass_config
    }
    pub fn get_mut_dot_cfg_generation_pass_config(&mut self) -> &mut DotCFGGenerationPassConfiguration {
        &mut self.dot_cfg_generation_pass_config
    }

    // EcallCheck
    pub fn get_ecall_check_config(&self) -> &EcallCheckConfiguration {
        &self.ecall_check_config
    }
    pub fn get_mut_ecall_check_config(&mut self) -> &mut EcallCheckConfiguration {
        &mut self.ecall_check_config
    }

    // GarbageInputValueCheck
    pub fn get_garbage_input_value_check_config(&self) -> &GarbageInputValueCheckConfiguration {
        &self.garbage_input_value_check_config
    }
    pub fn get_mut_garbage_input_value_check_config(&mut self) -> &mut GarbageInputValueCheckConfiguration {
        &mut self.garbage_input_value_check_config
    }

    // InstructionInTextCheck
    pub fn get_instruction_in_text_check_config(&self) -> &InstructionInTextCheckConfiguration {
        &self.instruction_in_text_check_config
    }
    pub fn get_mut_instruction_in_text_check_config(&mut self) -> &mut InstructionInTextCheckConfiguration {
        &mut self.instruction_in_text_check_config
    }

    // LostCalleeSavedRegisterCheck
    pub fn get_lost_callee_saved_register_check_config(&self) -> &LostCalleeSavedRegisterCheckConfiguration {
        &self.lost_callee_saved_register_check_config
    }
    pub fn get_mut_lost_callee_saved_register_check_config(&mut self) -> &mut LostCalleeSavedRegisterCheckConfiguration {
        &mut self.lost_callee_saved_register_check_config
    }

    // OverlappingFunctionCheck
    pub fn get_overlapping_function_check_config(&self) -> &OverlappingFunctionCheckConfiguration {
        &self.overlapping_function_check_config
    }
    pub fn get_mut_overlapping_function_check_config(&mut self) -> &mut OverlappingFunctionCheckConfiguration {
        &mut self.overlapping_function_check_config
    }

    // SaveToZeroCheck
    pub fn get_save_to_zero_check_config(&self) -> &SaveToZeroCheckConfiguration {
        &self.save_to_zero_check_config
    }
    pub fn get_mut_save_to_zero_check_config(&mut self) -> &mut SaveToZeroCheckConfiguration {
        &mut self.save_to_zero_check_config
    }

    // StackCheckPass
    pub fn get_stack_check_pass_config(&self) -> &StackCheckPassConfiguration {
        &self.stack_check_pass_config
    }
    pub fn get_mut_stack_check_pass_config(&mut self) -> &mut StackCheckPassConfiguration {
        &mut self.stack_check_pass_config
    }
}

pub trait ToManagerConfiguration {
    fn to_manager_configuration(&self) -> ManagerConfiguration;
}
