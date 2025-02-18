use crate::{
    analysis::MemoryLocation,
    cfg::Cfg,
    parser::ParserNode,
    passes::{LintError, LintPass},
};

impl MemoryLocation {
    fn get_location_with_offset(&self, offset: i32) -> MemoryLocation {
        match self {
            MemoryLocation::StackOffset(so) => MemoryLocation::StackOffset(so + offset),
        }
    }
}

/// A lint to ensure that the proper optimization is done for
/// double stores that can be replaced with a single store.
pub struct DoubleStoreInstCheck;
impl LintPass for DoubleStoreInstCheck {
    fn run(cfg: &Cfg, errors: &mut Vec<LintError>) {
        for node in cfg {
            match node.node() {
                // If this node is a store instruction
                ParserNode::Store(_) => {
                    // TODO: This currrently only works for stack, as our analysis implementation
                    // only supports stack.

                    // Find out where this value stores to, if it's known
                    if let Some((location, _)) = node.gen_memory_value() {
                        // Check both ahead and before if there are some existing values
                        for offset in [4, -4] {
                            let check_location = location.get_location_with_offset(offset);
                            if let Some(_) = node.memory_values_out().get(&check_location) {
                                errors.push(LintError::DoubleStoreInst {
                                    current_node: node.node(),
                                    offset_to_use: offset,
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parser::RVStringParser, passes::Manager};

    /// Compute the lints for a given input
    fn run_pass(input: &str) -> Vec<LintError> {
        let (nodes, error) = RVStringParser::parse_from_text(input);
        assert_eq!(error.len(), 0);

        let cfg = Manager::gen_full_cfg(nodes).unwrap(); // Need fn annotations
        DoubleStoreInstCheck::run_single_pass_along_cfg(&cfg)
    }

    #[test]
    fn basic_double_stores() {
        let input = "\
            main:                       \n\
                addi   sp, sp, -8       \n\
                li     a0, 10           \n\
                li     a1, 20           \n\
                sw     a0, 0(sp)        \n\
                sw     a1, 4(sp)        \n\
                addi   sp, sp, 8        \n\
                addi   a7, zero, 10     \n\
                ecall";

        let lints = run_pass(input);

        assert_eq!(lints.len(), 1);
    }

    #[test]
    fn stores_seperated_by_one() {
        let input = "\
            main:                       \n\
                addi   sp, sp, -8       \n\
                li     a0, 10           \n\
                sw     a0, 0(sp)        \n\
                li     a1, 20           \n\
                sw     a1, 4(sp)        \n\
                addi   sp, sp, 8        \n\
                addi   a7, zero, 10     \n\
                ecall";

        let lints = run_pass(input);

        assert_eq!(lints.len(), 1);
    }
}
