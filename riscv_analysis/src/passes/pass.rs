use crate::{cfg::Cfg, parser::ParserNode};

use super::{CfgError, LintError};

pub trait GenerationPass {
    fn run(cfg: &mut Cfg) -> Result<(), Box<CfgError>>;
}

pub trait AssertionPass {
    fn run(cfg: &Cfg) -> Result<(), Box<CfgError>>;
}

pub trait LintPass {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>);

    /// Run a single pass along a set of `ParserNode`s and return the errors.
    ///
    /// # Example
    ///
    /// ```
    /// use riscv_analysis::passes::{LintPass, LintError};
    /// use riscv_analysis::parser::ParserNode;
    /// use riscv_analysis::{arith, iarith};
    /// use riscv_analysis::cfg::Cfg;
    ///
    /// struct MyPass;
    /// impl LintPass for MyPass {
    ///    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
    ///       for node in cfg {
    ///         errors.push(LintError::InvalidStackPointer(node.node()));
    ///      }
    ///   }
    /// }
    ///
    /// let nodes = &[iarith!(Addi X1 X0 0)];
    /// let errors = MyPass::run_single_pass_along_nodes(nodes);
    /// assert_eq!(errors.len(), 1);
    /// assert!(matches!(errors[0], LintError::InvalidStackPointer(_)));
    /// ```
    #[must_use]
    fn run_single_pass_along_nodes(nodes: &[ParserNode]) -> Vec<LintError> {
        let cfg = Cfg::new(nodes.into()).unwrap();
        Self::run_single_pass_along_cfg(&cfg)
    }

    #[must_use]
    fn run_single_pass_along_cfg(cfg: &Cfg) -> Vec<LintError> {
        let mut errors = Vec::new();
        Self::run(cfg, &mut errors);
        errors
    }
}
