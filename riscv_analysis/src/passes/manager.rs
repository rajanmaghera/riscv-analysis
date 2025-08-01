use super::{CfgError, DiagnosticManager, GenerationPass, LintPass};
use crate::cfg::RegisterSet;
use crate::parser::{LabelString, RVParserOutput, With};
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
};

#[derive(Default)]
pub struct DebugInfo {
    pub output: bool,
    pub yaml: bool,
}

pub struct Manager;
impl Manager {
    pub fn gen_full_cfg(
        parser_output: RVParserOutput,
        additional_function_information: Option<Vec<(String, RegisterSet)>>,
    ) -> Result<Cfg, Box<CfgError>> {
        // Stage 1: Generate names of interrupt handler functions
        let mut predefined = {
            let mut cfg = Cfg::new(parser_output.clone())?;
            NodeDirectionPass::run(&mut cfg)?;
            AvailableValuePass::run(&mut cfg)?;
            cfg.get_names_of_interrupt_handler_functions()
        };

        // Combine interrupt call names and input function call names
        predefined.extend(parser_output.extra_labels.clone());
        let injected = additional_function_information
            .unwrap_or_default()
            .into_iter()
            .map(|(name, regs)| (With::blank(LabelString::new(name)), regs))
            .collect::<Vec<(With<LabelString>, RegisterSet)>>();
        predefined.extend(injected.iter().map(|(name, _)| name.clone()));

        // Stage 2: Generate full CFG
        let mut cfg = Cfg::new_with_predefined_call_names(parser_output, Some(&predefined))?;
        NodeDirectionPass::run(&mut cfg)?;
        EliminateDeadCodeDirectionsPass::run(&mut cfg)?;
        AvailableValuePass::run(&mut cfg)?;
        EcallTerminationPass::run(&mut cfg)?;
        FunctionMarkupPass::run(&mut cfg)?;

        AvailableValuePass::run(&mut cfg)?;
        EcallTerminationPass::run(&mut cfg)?;
        // EliminateDeadCodeDirectionsPass::run(&mut cfg)?; // to eliminate ecall terminated code
        LivenessPass::inject_return_registers_into_function(&mut cfg, injected.into_iter());
        LivenessPass::run(&mut cfg)?;
        Ok(cfg)
    }
    pub fn run_diagnostics(cfg: &Cfg, errors: &mut DiagnosticManager) {
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
    pub fn run(parser_output: RVParserOutput) -> Result<DiagnosticManager, Box<CfgError>> {
        let mut errors = DiagnosticManager::new();
        let cfg = Self::gen_full_cfg(parser_output, None)?;
        Self::run_diagnostics(&cfg, &mut errors);
        Ok(errors)
    }
}
