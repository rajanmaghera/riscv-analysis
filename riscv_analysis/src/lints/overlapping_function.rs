use crate::{
    cfg::Cfg,
    parser::{Label, ParserNode, RawToken},
    passes::{DiagnosticLocation, LintError, LintPass},
};
use uuid::Uuid;

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
                        token: RawToken {
                            text: l.get().0.clone(),
                            pos: l.range(),
                            file: l.file(),
                        },
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
    use crate::lints::OverlappingFunctionCheck;
    use crate::parser::{ParserNode, RVStringParser};
    use crate::passes::{LintError, LintPass, Manager};

    /// Compute the lints for a given input
    fn run_pass(input: &str) -> Vec<LintError> {
        let (nodes, error) = RVStringParser::parse_from_text(input);
        assert_eq!(error.len(), 0);

        let cfg = Manager::gen_full_cfg(nodes).unwrap(); // Need fn annotations
        OverlappingFunctionCheck::run_single_pass_along_cfg(&cfg)
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
        assert!(matches!(
        &lints[0], LintError::NodeInManyFunctions(node, _)
            if matches!(
                node, ParserNode::Label(label)
                    if label.token.text == "fn_b"
            )
        ));
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
        assert!(matches!(
        &lints[0], LintError::NodeInManyFunctions(node, _)
            if matches!(
                node, ParserNode::Label(label)
                    if label.token.text == "fn_b"
            )
        ));
        assert!(matches!(
        &lints[1], LintError::NodeInManyFunctions(node, _)
            if matches!(
                node, ParserNode::Label(label)
                    if label.token.text == "fn_c"
            )
        ));
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
