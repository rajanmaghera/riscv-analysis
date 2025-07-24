use crate::{
    cfg::Cfg,
    parser::InstructionProperties,
    passes::{DiagnosticManager, LintError, LintPass},
};

// Check if every ecall has a known call number
// Check if there are any instructions after an ecall to terminate the program
#[non_exhaustive]
pub struct EcallPass;
impl EcallPass {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for EcallPass {
    fn default() -> Self {
        Self::new()
    }
}

impl LintPass for EcallPass {
    fn run(&self, cfg: &Cfg, errors: &mut DiagnosticManager) {
        for node in cfg {
            if node.is_ecall() && node.known_ecall().is_none() {
                errors.push(LintError::UnknownEcall(node.node().clone()));
            }
        }
    }
}
