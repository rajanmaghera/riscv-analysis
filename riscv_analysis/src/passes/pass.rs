use crate::{cfg::Cfg, parser::ParserNode};

use super::{CfgError, DiagnosticManager};

pub trait GenerationPass {
    fn run(cfg: &mut Cfg) -> Result<(), Box<CfgError>>;
}

pub trait AssertionPass {
    fn run(cfg: &Cfg) -> Result<(), Box<CfgError>>;
}

pub struct LintPassDefaultOptions {
    enabled: bool,
}

impl Default for LintPassDefaultOptions {
    fn default() -> Self {
        Self { enabled: true }
    }
}

pub trait LintPass {
    /// Run a single pass along a set of `ParserNode`s and return the errors.
    ///
    /// # Example
    ///
    /// ```
    /// use riscv_analysis::passes::{LintPass, LintError, DiagnosticManager, LintPassDefaultOptions};
    /// use riscv_analysis::parser::ParserNode;
    /// use riscv_analysis::{arith, iarith};
    /// use riscv_analysis::cfg::Cfg;
    ///
    /// struct MyPass {
    ///     default_options: LintPassDefaultOptions
    /// }
    /// impl MyPass {
    ///     fn new() -> Self {
    ///         Self {
    ///             default_options: LintPassDefaultOptions::default()
    ///         }
    ///     }
    /// }
    /// impl LintPass for MyPass {
    ///    fn get_default_options(&self) -> &LintPassDefaultOptions {
    ///       &self.default_options
    ///    }
    ///    fn get_default_options_mut(&mut self) -> &mut LintPassDefaultOptions {
    ///       &mut self.default_options
    ///    }
    ///    fn run(&self, cfg: &Cfg, errors: &mut DiagnosticManager) {
    ///       for node in cfg {
    ///         errors.push(LintError::InvalidStackPointer(node.node()));
    ///      }
    ///   }
    /// }
    ///
    /// let nodes = &[iarith!(Addi X1 X0 0)];
    /// let errors = MyPass::new().run_single_pass_along_nodes(nodes);
    /// assert_eq!(errors.len(), 1);
    /// assert_eq!(errors[0].get_error_code(), "invalid-stack-pointer");
    /// ```
    fn run(&self, cfg: &Cfg, errors: &mut DiagnosticManager);

    fn get_default_options(&self) -> &LintPassDefaultOptions;
    fn get_default_options_mut(&mut self) -> &mut LintPassDefaultOptions;

    fn get_enabled(&self) -> bool {
        self.get_default_options().enabled
    }

    fn set_enabled(&mut self) {
        self.get_default_options_mut().enabled = true;
    }

    #[must_use]
    fn run_single_pass_along_nodes(&self, nodes: &[ParserNode]) -> DiagnosticManager {
        let cfg = Cfg::new(nodes.into()).unwrap();
        self.run_single_pass_along_cfg(&cfg)
    }

    #[must_use]
    fn run_single_pass_along_cfg(&self, cfg: &Cfg) -> DiagnosticManager {
        let mut errors = DiagnosticManager::new();
        self.run(cfg, &mut errors);
        errors
    }
}
