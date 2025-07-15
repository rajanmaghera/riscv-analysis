use crate::cfg::Cfg;
use crate::parser::{InstructionProperties, Register};
use crate::passes::{DiagnosticManager, LintError, LintPass, PassConfiguration};

pub struct SaveToZeroCheck;
impl LintPass<SaveToZeroCheckConfiguration> for SaveToZeroCheck {
    fn run(cfg: &Cfg, errors: &mut DiagnosticManager, config: &SaveToZeroCheckConfiguration) {
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
pub struct SaveToZeroCheckConfiguration {
    /// Is the pass enabled?
    enabled: bool,
}
impl Default for SaveToZeroCheckConfiguration {
    fn default() -> Self {
        SaveToZeroCheckConfiguration { enabled: true }
    }
}
impl PassConfiguration for SaveToZeroCheckConfiguration {
    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled
    }
}
