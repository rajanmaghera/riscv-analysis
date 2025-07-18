use crate::cfg::Cfg;
use crate::parser::InstructionProperties;
use crate::passes::{DiagnosticManager, LintError, LintPass, PassConfiguration};

// Check if every ecall has a known call number
// Check if there are any instructions after an ecall to terminate the program
pub struct EcallPass;
impl LintPass<EcallPassConfiguration> for EcallPass {
    fn run(cfg: &Cfg, errors: &mut DiagnosticManager, config: &EcallPassConfiguration) {
        if !config.get_enabled() {
            return;
        }
        for node in cfg {
            if node.is_ecall() && node.known_ecall().is_none() {
                errors.push(LintError::UnknownEcall(node.node().clone()));
            }
        }
    }
}
pub struct EcallPassConfiguration {
    /// Is the pass enabled?
    enabled: bool,
}
impl Default for EcallPassConfiguration {
    fn default() -> Self {
        EcallPassConfiguration { enabled: true }
    }
}
impl PassConfiguration for EcallPassConfiguration {
    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
