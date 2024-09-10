use crate::{
    cfg::Cfg,
    passes::{LintError, LintPass},
};

/// A lint to ensure warn about instructions that exist in more than one
/// function.
///
/// Though it is technically correct to have overlapping functions, this pattern
/// doesn't generally occur in canonical code. Instead, the existence of
/// overlapping functions usually indicates a mistaken jump to the middle of a
/// function.
pub struct OverlappingFunctionCheck;
impl LintPass for OverlappingFunctionCheck {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
        for node in &cfg.clone().nodes {
            // Capture entry points that are part of more than one function
            if node.functions().len() > 1 && node.is_function_entry().is_some() {
                errors.push(LintError::OverlappingFunctions(
                    node.node().clone(),
                    node.functions()
                        .clone()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ));
            }
        }
    }
}
