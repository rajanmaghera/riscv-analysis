use crate::passes::LintPassDefaultOptions;
use crate::{
    cfg::Cfg,
    parser::{HasRegisterSets, InstructionProperties, Register},
    passes::{DiagnosticManager, LintError, LintPass},
};

// TODO deprecate
// Check if there are any in values to the start of functions that are not args or saved registers
// Check if there are any in values at the start of a program
#[non_exhaustive]
pub struct GarbageInputValuePass {
    default_options: LintPassDefaultOptions,
}
impl GarbageInputValuePass {
    pub fn new() -> Self {
        Self {
            default_options: LintPassDefaultOptions::default(),
        }
    }
}

impl Default for GarbageInputValuePass {
    fn default() -> Self {
        Self::new()
    }
}

impl LintPass for GarbageInputValuePass {
    fn get_default_options(&self) -> &LintPassDefaultOptions {
        &self.default_options
    }

    fn get_default_options_mut(&mut self) -> &mut LintPassDefaultOptions {
        &mut self.default_options
    }
    fn run(&self, cfg: &Cfg, errors: &mut DiagnosticManager) {
        for node in cfg {
            if node.is_program_entry() {
                // get registers
                let garbage = node.live_in() - Register::program_args_set();
                if !garbage.is_empty() {
                    let mut ranges = Vec::new();
                    for reg in &garbage {
                        let mut ranges_tmp = Cfg::error_ranges_for_first_usage(&node, reg);
                        ranges.append(&mut ranges_tmp);
                    }
                    for range in ranges {
                        errors.push(LintError::InvalidUseBeforeAssignment(range.clone()));
                    }
                }
            } else if let Some(func) = node.is_function_entry_with_func() {
                let args = func.arguments();
                let garbage = node.live_in() - args - Register::callee_saved_set();
                if !garbage.is_empty() {
                    let mut ranges = Vec::new();
                    for reg in &garbage {
                        let mut ranges_tmp = Cfg::error_ranges_for_first_usage(&node, reg);
                        ranges.append(&mut ranges_tmp);
                    }
                    for range in ranges {
                        errors.push(LintError::InvalidUseBeforeAssignment(range.clone()));
                    }
                }
            }
        }
    }
}
