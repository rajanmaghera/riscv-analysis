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
};

use super::{CFGError, GenerationPass, LintError, LintPass};

pub struct Manager;
impl Manager {
    pub fn run(cfg: Cfg, debug: bool) -> Result<Vec<LintError>, Box<CFGError>> {
        let mut cfg = cfg;
        let mut errors = Vec::new();

        NodeDirectionPass::run(&mut cfg)?;
        EliminateDeadCodeDirectionsPass::run(&mut cfg)?;
        FunctionMarkupPass::run(&mut cfg)?;
        AvailableValuePass::run(&mut cfg)?;
        EcallTerminationPass::run(&mut cfg)?;
        // EliminateDeadCodeDirectionsPass::run(&mut cfg)?; // to eliminate ecall terminated code
        LivenessPass::run(&mut cfg)?;
        if debug {
            println!("{}", cfg);
        }
        SaveToZeroCheck::run(&cfg, &mut errors);
        DeadValueCheck::run(&cfg, &mut errors);
        EcallCheck::run(&cfg, &mut errors);
        ControlFlowCheck::run(&cfg, &mut errors);
        GarbageInputValueCheck::run(&cfg, &mut errors);
        StackCheckPass::run(&cfg, &mut errors);
        CalleeSavedRegisterCheck::run(&cfg, &mut errors);
        CalleeSavedGarbageReadCheck::run(&cfg, &mut errors);
        LostCalleeSavedRegisterCheck::run(&cfg, &mut errors);

        Ok(errors)
    }
}
