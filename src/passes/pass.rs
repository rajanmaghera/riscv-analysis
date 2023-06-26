use crate::cfg::Cfg;

use super::{CFGError, LintError};

pub trait GenerationPass {
    fn run(cfg: &mut Cfg) -> Result<(), Box<CFGError>>;
}

pub trait AssertionPass {
    fn run(cfg: &Cfg) -> Result<(), Box<CFGError>>;
}

pub trait LintPass {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>);
}
