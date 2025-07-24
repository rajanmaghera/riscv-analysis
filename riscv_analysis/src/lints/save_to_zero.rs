use crate::cfg::Cfg;
use crate::parser::{InstructionProperties, Register};
use crate::passes::{DiagnosticManager, LintError, LintPass, LintPassDefaultOptions};

#[non_exhaustive]
pub struct SaveToZeroPass {
    default_options: LintPassDefaultOptions,
}
impl SaveToZeroPass {
    pub fn new() -> Self {
        Self {
            default_options: LintPassDefaultOptions::default(),
        }
    }
}

impl Default for SaveToZeroPass {
    fn default() -> Self {
        Self::new()
    }
}

impl LintPass for SaveToZeroPass {
    fn get_default_options(&self) -> &LintPassDefaultOptions {
        &self.default_options
    }

    fn get_default_options_mut(&mut self) -> &mut LintPassDefaultOptions {
        &mut self.default_options
    }
    fn run(&self, cfg: &Cfg, errors: &mut DiagnosticManager) {
        for node in cfg {
            if let Some(register) = node.writes_to() {
                if register == Register::X0 && !node.can_skip_save_checks() {
                    errors.push(LintError::SaveToZero(register.clone()));
                }
            }
        }
    }
}
