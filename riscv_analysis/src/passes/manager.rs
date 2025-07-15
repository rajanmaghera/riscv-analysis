use crate::{
    analysis::{AvailableValuePass, LivenessPass},
    cfg::Cfg,
    gen::{
        EcallTerminationPass,
        EliminateDeadCodeDirectionsPass,
        FunctionMarkupPass,
        NodeDirectionPass,
    },
    lints::{
        CalleeSavedGarbageReadCheck,
        CalleeSavedRegisterCheck,
        ControlFlowCheck,
        DeadValueCheck,
        DotCFGGenerationPass,
        EcallCheck,
        GarbageInputValueCheck,
        InstructionInTextCheck,
        LostCalleeSavedRegisterCheck,
        OverlappingFunctionCheck,
        SaveToZeroCheck,
        StackCheckPass,
    },
    parser::ParserNode,
    passes::ManagerConfiguration,
};

use super::{CfgError, DiagnosticManager, GenerationPass, LintPass};

#[derive(Default)]
pub struct DebugInfo {
    pub output: bool,
    pub yaml: bool,
}

pub struct Manager;
impl Manager {
    pub fn gen_full_cfg(nodes: Vec<ParserNode>) -> Result<Cfg, Box<CfgError>> {
        // Stage 1: Generate names of interrupt handler functions
        let interrupt_call_names = {
            let mut cfg = Cfg::new(nodes.clone())?;
            NodeDirectionPass::run(&mut cfg)?;
            AvailableValuePass::run(&mut cfg)?;
            cfg.get_names_of_interrupt_handler_functions()
        };

        // Stage 2: Generate full CFG
        let mut cfg = Cfg::new_with_predefined_call_names(nodes, &Some(interrupt_call_names))?;
        NodeDirectionPass::run(&mut cfg)?;
        EliminateDeadCodeDirectionsPass::run(&mut cfg)?;
        AvailableValuePass::run(&mut cfg)?;
        EcallTerminationPass::run(&mut cfg)?;
        FunctionMarkupPass::run(&mut cfg)?;

        AvailableValuePass::run(&mut cfg)?;
        EcallTerminationPass::run(&mut cfg)?;
        // EliminateDeadCodeDirectionsPass::run(&mut cfg)?; // to eliminate ecall terminated code
        LivenessPass::run(&mut cfg)?;
        Ok(cfg)
    }
    pub fn run_diagnostics(cfg: &Cfg, errors: &mut DiagnosticManager, config: &ManagerConfiguration) {
        DotCFGGenerationPass::run(cfg, errors, config.get_dot_cfg_generation_pass_config());
        SaveToZeroCheck::run(cfg, errors, config.get_save_to_zero_check_config());
        DeadValueCheck::run(cfg, errors, config.get_dead_value_check_config());
        InstructionInTextCheck::run(cfg, errors, config.get_instruction_in_text_check_config());
        EcallCheck::run(cfg, errors, config.get_ecall_check_config());
        ControlFlowCheck::run(cfg, errors, config.get_control_flow_check_config());
        GarbageInputValueCheck::run(cfg, errors, config.get_garbage_input_value_check_config());
        StackCheckPass::run(cfg, errors, config.get_stack_check_pass_config());
        CalleeSavedRegisterCheck::run(cfg, errors, config.get_callee_saved_register_check_config());
        CalleeSavedGarbageReadCheck::run(cfg, errors, config.get_callee_saved_garbage_read_check_config());
        LostCalleeSavedRegisterCheck::run(cfg, errors, config.get_lost_callee_saved_register_check_config());
        OverlappingFunctionCheck::run(cfg, errors, config.get_overlapping_function_check_config());
    }
    pub fn run(cfg: Vec<ParserNode>, config: &ManagerConfiguration) -> Result<DiagnosticManager, Box<CfgError>> {
        let mut errors = DiagnosticManager::new();
        let cfg = Self::gen_full_cfg(cfg)?;
        Self::run_diagnostics(&cfg, &mut errors, &config);
        Ok(errors)
    }
    pub fn run_with_default_config(cfg: Vec<ParserNode>) -> Result<DiagnosticManager, Box<CfgError>> {
        Self::run(cfg, &ManagerConfiguration::default())
    }
}
