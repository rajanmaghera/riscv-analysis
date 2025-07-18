use crate::cfg::Cfg;
use crate::parser::{HasRegisterSets, InstructionProperties, Register};
use crate::passes::{DiagnosticManager, LintError, LintPass, PassConfiguration};

// TODO deprecate
// Check if there are any in values to the start of functions that are not args or saved registers
// Check if there are any in values at the start of a program
pub struct GarbageInputValuePass;
impl LintPass<GarbageInputValuePassConfiguration> for GarbageInputValuePass {
    fn run(cfg: &Cfg, errors: &mut DiagnosticManager, config: &GarbageInputValuePassConfiguration) {
        if !config.get_enabled() {
            return;
        }
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
pub struct GarbageInputValuePassConfiguration {
    /// Is the pass enabled?
    enabled: bool,
}
impl Default for GarbageInputValuePassConfiguration {
    fn default() -> Self {
        GarbageInputValuePassConfiguration { enabled: true }
    }
}
impl PassConfiguration for GarbageInputValuePassConfiguration {
    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
