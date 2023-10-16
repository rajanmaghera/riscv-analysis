use crate::{
    analysis::{AvailableValuePass, LivenessPass},
    cfg::Cfg,
    gen::{
        EcallTerminationPass, EliminateDeadCodeDirectionsPass, FunctionMarkupPass,
        NodeDirectionPass,
    },
    lints::{
        CalleeSavedGarbageReadCheck, CalleeSavedRegisterCheck, ControlFlowCheck, DeadValueCheck,
        EcallCheck, GarbageInputValueCheck, LostCalleeSavedRegisterCheck, SaveToZeroCheck,
        StackCheckPass,
    },
    parser::ParserNode,
};

use super::{CFGError, GenerationPass, LintError, LintPass};

#[derive(Default)]
pub struct DebugInfo {
    pub output: bool,
    pub yaml: bool,
}

pub struct Manager;
impl Manager {
    pub fn gen_full_cfg(cfg: Vec<ParserNode>) -> Result<Cfg, Box<CFGError>> {
        let mut cfg = Cfg::new(cfg)?;

        NodeDirectionPass::run(&mut cfg)?;
        EliminateDeadCodeDirectionsPass::run(&mut cfg)?;
        // AvailableValuePass::run(&mut cfg)?;
        // EcallTerminationPass::run(&mut cfg)?;
        // TODO big issue, we need to run all of these
        // at the same time, otherwise some function terminations
        // do not register.
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
        EcallCheck::run(cfg, errors);
        ControlFlowCheck::run(cfg, errors);
        GarbageInputValueCheck::run(cfg, errors);
        StackCheckPass::run(cfg, errors);
        CalleeSavedRegisterCheck::run(cfg, errors);
        CalleeSavedGarbageReadCheck::run(cfg, errors);
        LostCalleeSavedRegisterCheck::run(cfg, errors);
    }
    pub fn run(cfg: Vec<ParserNode>) -> Result<Vec<LintError>, Box<CFGError>> {
        let mut errors = Vec::new();
        let cfg = Self::gen_full_cfg(cfg)?;
        Self::run_diagnostics(&cfg, &mut errors);
        Ok(errors)
    }
}
