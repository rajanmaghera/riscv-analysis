use crate::cfg::Cfg;
use crate::parser::{InstructionProperties, Register};
use crate::passes::{DiagnosticManager, LintError, LintPass};

pub struct SaveToZeroCheck;

impl LintPass for SaveToZeroCheck {
    fn run(cfg: &Cfg, errors: &mut DiagnosticManager) {
        for node in cfg {
            if let Some(register) = node.writes_to() {
                if register == Register::X0 && !node.can_skip_save_checks() {
                    errors.push(LintError::SaveToZero(register.clone()));
                }
            }
        }
    }
}
