use crate::passes::LintPassDefaultOptions;
use crate::{
    cfg::Cfg,
    parser::{Label, ParserNode},
    passes::{DiagnosticManager, LintError, LintPass},
};
use uuid::Uuid;

/// A lint to ensure warn about instructions that exist in more than one
/// function.
///
/// Though it is technically correct to have overlapping functions, this pattern
/// doesn't generally occur in canonical code. Instead, the existence of
/// overlapping functions usually indicates a mistaken jump to the middle of a
/// function.
#[non_exhaustive]
pub struct OverlappingFunctionPass {
    default_options: LintPassDefaultOptions,
}
impl OverlappingFunctionPass {
    pub fn new() -> Self {
        Self {
            default_options: LintPassDefaultOptions::default(),
        }
    }
}

impl Default for OverlappingFunctionPass {
    fn default() -> Self {
        Self::new()
    }
}

impl LintPass for OverlappingFunctionPass {
    fn get_default_options(&self) -> &LintPassDefaultOptions {
        &self.default_options
    }

    fn get_default_options_mut(&mut self) -> &mut LintPassDefaultOptions {
        &mut self.default_options
    }
    fn run(&self, cfg: &Cfg, errors: &mut DiagnosticManager) {
        for node in cfg {
            // Capture entry points that are part of more than one function
            // NOTE: We only give an error for the first line of a function,
            //       even though there may be many overlapping instructions.
            //       This is done to not overwhelm the user with errors.
            if node.functions().len() > 1 && node.is_function_entry_with_func().is_some() {
                // HACK: Create a dummy label with the same name
                let labels = node.labels();
                let labels = labels
                    .iter()
                    .map(|l| Label {
                        name: l.clone(),
                        key: Uuid::new_v4(),
                        token: l.raw_token().clone(),
                    })
                    .collect::<Vec<_>>();
                let label = labels.first();

                if let Some(l) = label {
                    errors.push(LintError::NodeInManyFunctions(
                        ParserNode::Label(l.clone()),
                        node.functions().clone().into_iter().collect::<Vec<_>>(),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lints::OverlappingFunctionPass;
    use crate::parser::RVStringParser;
    use crate::passes::{DiagnosticManager, LintPass, Manager};

    /// Compute the lints for a given input
    fn run_pass(input: &str) -> DiagnosticManager {
        let (nodes, error) = RVStringParser::parse_from_text(input);
        assert_eq!(error.len(), 0);

        let cfg = Manager::gen_full_cfg(nodes).unwrap(); // Need fn annotations
        OverlappingFunctionPass::new().run_single_pass_along_cfg(&cfg)
    }

    #[test]
    fn two_overlapping_functions() {
        // Functions `fn_a` and `fn_b` share the same `ret` instruction
        let input = "\
            main:                      \n\
                li     a0, 0           \n\
                jal    fn_a            \n\
                jal    fn_b            \n\
                addi   a7, zero, 10    \n\
                ecall                  \n\
            fn_a:                      \n\
                addi   a0, a0, 1       \n\
            fn_b:                      \n\
                addi   a0, a0, 2       \n\
                ret                    \n";

        let lints = run_pass(input);
        assert_eq!(lints.len(), 1);

        assert_eq!(lints[0].get_error_code(), "node-in-many-functions");
        assert_eq!(lints[0].raw_text(), "fn_b:",);
    }

    #[test]
    fn three_overlapping_functions() {
        // The functions `fn_a`, `fn_b`, and `fn_c` overlap
        let input = "\
            main:                      \n\
                li     a0, 0           \n\
                jal    fn_a            \n\
                jal    fn_b            \n\
                jal    fn_c            \n\
                addi   a7, zero, 10    \n\
                ecall                  \n\
            fn_a:                      \n\
                addi   a0, a0, 1       \n\
            fn_b:                      \n\
                addi   a0, a0, 2       \n\
            fn_c:                      \n\
                addi   a0, a0, 3       \n\
                ret                    \n";

        let lints = run_pass(input);

        assert_eq!(lints.len(), 2);

        assert_eq!(lints[0].get_error_code(), "node-in-many-functions");
        assert_eq!(lints[0].raw_text(), "fn_b:");

        assert_eq!(lints[1].get_error_code(), "node-in-many-functions");
        assert_eq!(lints[1].raw_text(), "fn_c:");
    }

    #[test]
    fn no_overlap() {
        // The function `fn_b` has its source inside of `fn_a`, but there is no
        // real overlap
        let input = "\
            main:                      \n\
                li     a0, 0           \n\
                jal    fn_a            \n\
                jal    fn_b            \n\
                addi   a7, zero, 10    \n\
                ecall                  \n\
            fn_a:                      \n\
                addi   a0, a0, 1       \n\
                j      fn_a_rest       \n\
            fn_b:                      \n\
                addi   a0, a0, 2       \n\
                ret                    \n\
            fn_a_rest:                 \n\
                sub    a0, a0, a0      \n\
                ret                    \n";

        let lints = run_pass(input);

        assert_eq!(lints.len(), 0);
    }
}
