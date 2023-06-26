use crate::cfg::BaseCFG;

use super::{CFGError, LintError};

pub trait GenerationPass {
    fn run(cfg: &mut BaseCFG) -> Result<(), CFGError>;
}

pub trait AssertionPass {
    fn run(cfg: &BaseCFG) -> Result<(), CFGError>;
}

pub trait LintPass {
    fn run(cfg: &BaseCFG, errors: &mut Vec<LintError>);
}
