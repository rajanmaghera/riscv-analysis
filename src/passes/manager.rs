use crate::{
    analysis::{AvailableValuePass, LivenessPass},
    cfg::{CFGWrapper, Cfg},
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

#[derive(Default)]
pub struct DebugInfo {
    pub output: bool,
    pub yaml: bool,
}

pub struct Manager;
impl Manager {
    pub fn gen_full_cfg(cfg: Cfg) -> Result<Cfg, Box<CFGError>> {
        let mut cfg = cfg;

        NodeDirectionPass::run(&mut cfg)?;
        EliminateDeadCodeDirectionsPass::run(&mut cfg)?;
        FunctionMarkupPass::run(&mut cfg)?;
        AvailableValuePass::run(&mut cfg)?;
        EcallTerminationPass::run(&mut cfg)?;
        // EliminateDeadCodeDirectionsPass::run(&mut cfg)?; // to eliminate ecall terminated code
        LivenessPass::run(&mut cfg)?;
        Ok(cfg)
    }
    pub fn run(cfg: Cfg, debug: DebugInfo) -> Result<Vec<LintError>, Box<CFGError>> {
        let mut errors = Vec::new();
        let cfg = Self::gen_full_cfg(cfg)?;
        if debug.yaml {
            println!(
                "{}",
                serde_yaml::to_string(&CFGWrapper::from(&cfg)).unwrap()
            );
        } else if debug.output {
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
