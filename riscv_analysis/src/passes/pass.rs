use crate::{cfg::Cfg, parser::ParserNode};
use super::{CfgError, DiagnosticManager};

pub trait GenerationPass {
    fn run(cfg: &mut Cfg) -> Result<(), Box<CfgError>>;
}

pub trait AssertionPass {
    fn run(cfg: &Cfg) -> Result<(), Box<CfgError>>;
}

/// Configuration for a pass.
/// 
/// Every `PassConfiguration` must implement `Default`.
pub trait PassConfiguration: Default {
    /// Check if the pass is enabled.
    fn get_enabled(&self) -> bool;
    /// Enable or disable the pass.
    fn set_enabled(&mut self, enabled: bool);
}
pub trait LintPass<Config:PassConfiguration> {
    fn run(cfg: &Cfg, errors: &mut DiagnosticManager, config: &Config);

    /// Run a single pass along a set of `ParserNode`s and return the errors.
    ///
    /// # Example
    ///
    /// ```
    /// use riscv_analysis::passes::{LintPass, LintError, DiagnosticManager, PassConfiguration};
    /// use riscv_analysis::parser::ParserNode;
    /// use riscv_analysis::{arith, iarith};
    /// use riscv_analysis::cfg::Cfg;
    ///
    /// struct MyPass;
    /// impl LintPass<MyPassConfiguration> for MyPass {
    ///    fn run(cfg: &Cfg, errors: &mut DiagnosticManager, config: &MyPassConfiguration) {
    ///        if !config.get_enabled() {
    ///            return;
    ///        }
    ///        for node in cfg {
    ///            errors.push(LintError::InvalidStackPointer(node.node()));
    ///        }
    ///    }
    /// }
    /// #[derive(Default)] // pass is disabled by default
    /// pub struct MyPassConfiguration {
    ///     /// Is the pass enabled?
    ///     enabled: bool,
    /// }
    /// impl PassConfiguration for MyPassConfiguration {
    ///     fn get_enabled(&self) -> bool {
    ///         self.enabled
    ///     }
    ///
    ///     fn set_enabled(&mut self, enabled: bool) {
    ///         self.enabled = enabled
    ///     }
    /// }
    ///
    /// let nodes = &[iarith!(Addi X1 X0 0)];
    /// let config = MyPassConfiguration { enabled: true };
    /// let errors = MyPass::run_single_pass_along_nodes(nodes, &config);
    /// assert_eq!(errors.len(), 1);
    /// assert_eq!(errors[0].get_error_code(), "invalid-stack-pointer");
    /// ```
    #[must_use]
    fn run_single_pass_along_nodes(nodes: &[ParserNode], config: &Config) -> DiagnosticManager {
        let cfg = Cfg::new(nodes.into()).unwrap();
        Self::run_single_pass_along_cfg(&cfg, &config)
    }

    #[must_use]
    fn run_single_pass_along_cfg(cfg: &Cfg, config: &Config) -> DiagnosticManager {
        let mut errors = DiagnosticManager::new();
        Self::run(cfg, &mut errors, &config);
        errors
    }
}
