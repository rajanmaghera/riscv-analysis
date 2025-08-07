use super::{CfgError, DiagnosticManager};
use crate::parser::{ProgramEntryType, RVParserOutput};
use crate::{cfg::Cfg, parser::ParserNode};

pub trait GenerationPass {
    fn run(cfg: &mut Cfg) -> Result<(), Box<CfgError>>;
}

pub trait AssertionPass {
    fn run(cfg: &Cfg) -> Result<(), Box<CfgError>>;
}

pub trait LintPass {
    /// Run a single pass along a set of `ParserNode`s and return the errors.
    ///
    /// # Example
    ///
    /// ```
    /// use riscv_analysis::passes::{LintPass, LintError, DiagnosticManager};
    /// use riscv_analysis::parser::{ParserNode, RVStringParser};
    /// use riscv_analysis::cfg::Cfg;
    ///
    /// struct MyPass;
    /// impl MyPass {
    ///     fn new() -> Self {
    ///         Self
    ///     }
    /// }
    /// impl LintPass for MyPass {
    ///    fn get_pass_name(&self) -> &'static str {
    ///       "my-pass"
    ///    }
    ///    fn run(&self, cfg: &Cfg, errors: &mut DiagnosticManager) {
    ///       for node in cfg.iter_source() {
    ///         errors.push(LintError::InvalidStackPointer(node.node()));
    ///      }
    ///   }
    /// }
    ///
    /// let parsed = RVStringParser::parse_from_text("addi x1 x0 0");
    /// let errors = MyPass::new().run_single_pass_along_nodes(&parsed.nodes);
    /// assert_eq!(errors.len(), 1);
    /// assert_eq!(errors[0].get_error_code(), "invalid-stack-pointer");
    /// ```
    fn run(&self, cfg: &Cfg, errors: &mut DiagnosticManager);

    fn get_pass_name(&self) -> &'static str;

    #[must_use]
    fn run_single_pass_along_nodes(&self, nodes: &[ParserNode]) -> DiagnosticManager {
        let cfg = Cfg::new(
            RVParserOutput {
                nodes: nodes.into(),
                ..Default::default()
            },
            None,
            None,
            &ProgramEntryType::FirstInstruction,
        )
        .unwrap();
        self.run_single_pass_along_cfg(&cfg)
    }

    #[must_use]
    fn run_single_pass_along_cfg(&self, cfg: &Cfg) -> DiagnosticManager {
        let mut errors = DiagnosticManager::new();
        self.run(cfg, &mut errors);
        errors
    }
}
