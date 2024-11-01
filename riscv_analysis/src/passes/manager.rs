use crate::{
    analysis::{AvailableValuePass, LivenessPass},
    cfg::Cfg,
    gen::{
        EcallTerminationPass, EliminateDeadCodeDirectionsPass, FunctionMarkupPass,
        NodeDirectionPass,
    },
    lints::{
        CalleeSavedGarbageReadCheck, CalleeSavedRegisterCheck, ControlFlowCheck, DeadValueCheck,
        EcallCheck, GarbageInputValueCheck, InstructionInTextCheck, LostCalleeSavedRegisterCheck,
        OverlappingFunctionCheck, SaveToZeroCheck, StackCheckPass,
    },
    parser::ParserNode,
};

use super::{CfgError, GenerationPass, LintError, LintPass};

#[derive(Default)]
pub struct DebugInfo {
    pub output: bool,
    pub yaml: bool,
}

pub struct Manager;
impl Manager {
    pub fn gen_full_cfg(nodes: Vec<ParserNode>) -> Result<Cfg, Box<CfgError>> {
        // Stage 1: Generate names of interrupt handler functions
        let mut cfg = Cfg::new(nodes.clone())?;
        NodeDirectionPass::run(&mut cfg)?;
        AvailableValuePass::run(&mut cfg)?;
        let interrupt_call_names = cfg.get_names_of_interrupt_handler_functions();

        // Stage 2: Generate full CFG
        let mut cfg = Cfg::new_with_predefined_call_names(nodes, Some(interrupt_call_names))?;
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
    pub fn run_diagnostics(cfg: &Cfg, errors: &mut Vec<LintError>) {
        SaveToZeroCheck::run(cfg, errors);
        DeadValueCheck::run(cfg, errors);
        InstructionInTextCheck::run(cfg, errors);
        EcallCheck::run(cfg, errors);
        ControlFlowCheck::run(cfg, errors);
        GarbageInputValueCheck::run(cfg, errors);
        StackCheckPass::run(cfg, errors);
        CalleeSavedRegisterCheck::run(cfg, errors);
        CalleeSavedGarbageReadCheck::run(cfg, errors);
        LostCalleeSavedRegisterCheck::run(cfg, errors);
        OverlappingFunctionCheck::run(cfg, errors);
    }
    pub fn run(cfg: Vec<ParserNode>) -> Result<Vec<LintError>, Box<CfgError>> {
        let mut errors = Vec::new();
        let cfg = Self::gen_full_cfg(cfg)?;
        Self::run_diagnostics(&cfg, &mut errors);
        Ok(errors)
    }
}
