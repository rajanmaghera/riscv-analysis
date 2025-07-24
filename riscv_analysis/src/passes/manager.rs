use crate::{
    analysis::{AvailableValuePass, LivenessPass},
    cfg::Cfg,
    gen::{
        EcallTerminationPass, EliminateDeadCodeDirectionsPass, FunctionMarkupPass,
        NodeDirectionPass,
    },
    lints::{
        CalleeSavedGarbageReadPass, CalleeSavedRegisterPass, ControlFlowPass, DeadValuePass,
        EcallPass, GarbageInputValuePass, InstructionInTextPass, LostCalleeSavedRegisterPass,
        OverlappingFunctionPass, SaveToZeroPass, StackPass,
    },
    parser::ParserNode,
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
    pub fn run_diagnostics(cfg: &Cfg, errors: &mut DiagnosticManager) {
        SaveToZeroPass::new().run(cfg, errors);
        DeadValuePass::new().run(cfg, errors);
        InstructionInTextPass::new().run(cfg, errors);
        EcallPass::new().run(cfg, errors);
        ControlFlowPass::new().run(cfg, errors);
        GarbageInputValuePass::new().run(cfg, errors);
        StackPass::new().run(cfg, errors);
        CalleeSavedRegisterPass::new().run(cfg, errors);
        CalleeSavedGarbageReadPass::new().run(cfg, errors);
        LostCalleeSavedRegisterPass::new().run(cfg, errors);
        OverlappingFunctionPass::new().run(cfg, errors);
    }
    pub fn run(cfg: Vec<ParserNode>) -> Result<DiagnosticManager, Box<CfgError>> {
        let mut errors = DiagnosticManager::new();
        let cfg = Self::gen_full_cfg(cfg)?;
        Self::run_diagnostics(&cfg, &mut errors);
        Ok(errors)
    }
}
