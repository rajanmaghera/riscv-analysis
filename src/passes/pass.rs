use crate::cfg::CFG;

use super::{CFGError, LintError};

pub trait GenerationPass {
    fn run(cfg: &mut CFG) -> Result<(), Box<CFGError>>;
}

pub trait AssertionPass {
    fn run(cfg: &CFG) -> Result<(), Box<CFGError>>;
}

pub trait LintPass {
    fn run(cfg: &CFG, errors: &mut Vec<LintError>);
}
