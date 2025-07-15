use crate::cfg::Cfg;
use crate::parser::{InstructionProperties, Register};
use crate::passes::{DiagnosticManager, LintError, LintPass, PassConfiguration};

pub struct SaveToZeroPass;
impl LintPass<SaveToZeroPassConfiguration> for SaveToZeroPass {
    fn run(cfg: &Cfg, errors: &mut DiagnosticManager, config: &SaveToZeroPassConfiguration) {
        if !config.get_enabled() {
            return;
        }
        for node in cfg {
            if let Some(register) = node.writes_to() {
                if register == Register::X0 && !node.can_skip_save_checks() {
                    errors.push(LintError::SaveToZero(register.clone()));
                }
            }
        }
    }
}
pub struct SaveToZeroPassConfiguration {
    /// Is the pass enabled?
    enabled: bool,
}
impl Default for SaveToZeroPassConfiguration {
    fn default() -> Self {
        SaveToZeroPassConfiguration { enabled: true }
    }
}
impl PassConfiguration for SaveToZeroPassConfiguration {
    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled
    }
}
