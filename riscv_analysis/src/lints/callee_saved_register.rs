use crate::analysis::AvailableValue;
use crate::cfg::Cfg;
use crate::parser::{HasRegisterSets, Register};
use crate::passes::{DiagnosticManager, LintError, LintPass, PassConfiguration};

// Check if the values of callee-saved registers are restored to the original value at the end of the function
pub struct CalleeSavedRegisterCheck;
impl LintPass<CalleeSavedRegisterCheckConfiguration> for CalleeSavedRegisterCheck {
    fn run(cfg: &Cfg, errors: &mut DiagnosticManager, config: &CalleeSavedRegisterCheckConfiguration) {
        if !config.get_enabled() {
            return;
        }
        for func in cfg.functions().values() {
            let exit_vals = func.exit().reg_values_in();
            for reg in &Register::callee_saved_set() {
                match exit_vals.get(&reg) {
                    Some(AvailableValue::OriginalRegisterWithScalar(reg2, offset))
                        if reg2 == &reg && offset == &0 =>
                    {
                        // GOOD!
                    }
                    _ => {
                        // TODO combine with lost register check

                        // This means that we are overwriting a callee-saved register
                        // We will traverse the function to find the first time
                        // from the return point that that register was overwritten.
                        let ranges = Cfg::error_ranges_for_first_store(&func.exit(), reg);
                        for range in ranges {
                            errors.push(LintError::OverwriteCalleeSavedRegister(range));
                        }
                    }
                }
            }
        }
    }
}
pub struct CalleeSavedRegisterCheckConfiguration {
    /// Is the pass enabled?
    enabled: bool,
}
impl Default for CalleeSavedRegisterCheckConfiguration {
    fn default() -> Self {
        CalleeSavedRegisterCheckConfiguration { enabled: true }
    }
}
impl PassConfiguration for CalleeSavedRegisterCheckConfiguration {
    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled
    }
}
