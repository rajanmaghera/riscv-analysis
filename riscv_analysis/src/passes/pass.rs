use crate::cfg::Cfg;

use super::{CfgError, LintError};

pub trait GenerationPass {
    fn run(cfg: &mut Cfg) -> Result<(), Box<CfgError>>;
}

pub trait AssertionPass {
    fn run(cfg: &Cfg) -> Result<(), Box<CfgError>>;
}

pub trait LintPass {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>);
}
