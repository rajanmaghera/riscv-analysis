use crate::passes::LintPassDefaultOptions;
use crate::{
    cfg::Cfg,
    parser::InstructionProperties,
    passes::{DiagnosticManager, LintError, LintPass},
};

// Check if every ecall has a known call number
// Check if there are any instructions after an ecall to terminate the program
#[non_exhaustive]
pub struct EcallPass {
    default_options: LintPassDefaultOptions,
}
impl EcallPass {
    pub fn new() -> Self {
        Self {
            default_options: LintPassDefaultOptions::default(),
        }
    }
}

impl Default for EcallPass {
    fn default() -> Self {
        Self::new()
    }
}

impl LintPass for EcallPass {
    fn get_default_options(&self) -> &LintPassDefaultOptions {
        &self.default_options
    }

    fn get_default_options_mut(&mut self) -> &mut LintPassDefaultOptions {
        &mut self.default_options
    }
    fn run(&self, cfg: &Cfg, errors: &mut DiagnosticManager) {
        for node in cfg {
            if node.is_ecall() && node.known_ecall().is_none() {
                errors.push(LintError::UnknownEcall(node.node().clone()));
            }
        }
    }
}
