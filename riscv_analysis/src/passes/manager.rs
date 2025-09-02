use super::{CfgError, DiagnosticManager, GenerationPass, LintPass};
use crate::cfg::RegisterSet;
use crate::parser::{LabelString, ProgramEntryType, RVParserOutput, With};
use crate::{
    analysis::{AvailableValuePass, LivenessPass},
    cfg::Cfg,
    gen::{FunctionMarkupPass, NodeDirectionPass},
    lints::{
        CalleeSavedGarbageReadPass, CalleeSavedRegisterPass, ControlFlowPass, DeadValuePass,
        EcallPass, GarbageInputValuePass, LostCalleeSavedRegisterPass, OverlappingFunctionPass,
        SaveToZeroPass, StackPass,
    },
};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct DebugInfo {
    pub output: bool,
    pub yaml: bool,
}

pub struct Manager {
    passes: HashMap<&'static str, LintPassWrapper>,
}

struct LintPassWrapper {
    pass: Box<dyn LintPass>,
    enabled: bool,
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}

impl Manager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            passes: HashMap::new(),
        }
    }

    pub fn gen_empty_cfg(
        parser_output: &RVParserOutput,
        external_functions: &Option<HashSet<(With<LabelString>, RegisterSet, RegisterSet)>>,
        program_entry: &ProgramEntryType,
    ) -> Result<Cfg, Box<CfgError>> {
        // Combine interrupt call names and input function call names
        let predefined = parser_output.extra_labels.clone();
        Cfg::new(
            parser_output.clone(),
            Some(&predefined),
            &external_functions,
            program_entry,
        )
    }

    pub fn gen_full_cfg(
        parser_output: &RVParserOutput,
        external_functions: Option<HashSet<(With<LabelString>, RegisterSet, RegisterSet)>>,
        program_entry: &ProgramEntryType,
    ) -> Result<Cfg, Box<CfgError>> {
        let mut cfg = Self::gen_empty_cfg(parser_output, &external_functions, program_entry)?;
        NodeDirectionPass::run(&mut cfg)?;
        FunctionMarkupPass::run(&mut cfg)?;
        AvailableValuePass::run(&mut cfg)?;
        LivenessPass::inject_return_registers_into_function(
            &mut cfg,
            external_functions
                .into_iter()
                .flat_map(std::iter::IntoIterator::into_iter),
        );
        LivenessPass::run(&mut cfg)?;
        Ok(cfg)
    }

    /// Register many passes with the pass manager.
    pub fn register_passes(&mut self, passes: impl IntoIterator<Item = Box<dyn LintPass>>) {
        self.passes.extend(passes.into_iter().map(|x| {
            (
                x.get_pass_name(),
                LintPassWrapper {
                    pass: x,
                    enabled: false,
                },
            )
        }));
    }

    /// Register a pass with the pass manager.
    pub fn register_pass(&mut self, pass: Box<dyn LintPass>) {
        self.register_passes([pass]);
    }

    /// Register all built in passes.
    pub fn register_and_enable_built_in_passes(&mut self) {
        let diags: [Box<dyn LintPass>; 10] = [
            Box::new(SaveToZeroPass::new()),
            Box::new(DeadValuePass::new()),
            // Box::new(InstructionInTextPass::new()),
            Box::new(EcallPass::new()),
            Box::new(ControlFlowPass::new()),
            Box::new(GarbageInputValuePass::new()),
            Box::new(StackPass::new()),
            Box::new(CalleeSavedRegisterPass::new()),
            Box::new(CalleeSavedGarbageReadPass::new()),
            Box::new(LostCalleeSavedRegisterPass::new()),
            Box::new(OverlappingFunctionPass::new()),
        ];
        let keys = diags
            .iter()
            .map(|x| x.get_pass_name())
            .collect::<Vec<&'static str>>();
        self.register_passes(diags);
        for key in &keys {
            self.enable_pass(key);
        }
    }

    /// Enable a pass.
    ///
    /// Function returns true if the pass name exists, false
    /// otherwise.
    pub fn enable_pass(&mut self, pass_name: &'static str) -> bool {
        if let Some(pass) = self.passes.get_mut(pass_name) {
            pass.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable a pass.
    ///
    /// Function returns true if the pass name exists, false
    /// otherwise.
    pub fn disable_pass(&mut self, pass_name: &'static str) -> bool {
        if let Some(pass) = self.passes.get_mut(pass_name) {
            pass.enabled = false;
            true
        } else {
            false
        }
    }

    /// Run lint passes and generate diagnostics.
    pub fn run_diagnostics(&self, cfg: &Cfg, errors: &mut DiagnosticManager) {
        for diag in self.passes.values() {
            diag.pass.run(cfg, errors);
        }
    }

    pub fn run(
        parser_output: &RVParserOutput,
        program_entry: &ProgramEntryType,
    ) -> Result<DiagnosticManager, Box<CfgError>> {
        let mut errors = DiagnosticManager::new();
        let cfg = Self::gen_full_cfg(parser_output, None, program_entry)?;
        let manager = Self::new();
        manager.run_diagnostics(&cfg, &mut errors);
        Ok(errors)
    }
}
