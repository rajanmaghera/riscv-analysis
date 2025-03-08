use crate::{
    cfg::Cfg,
    parser::InstructionProperties,
    passes::{CfgError, GenerationPass},
};

pub struct EliminateDeadCodeDirectionsPass;
impl GenerationPass for EliminateDeadCodeDirectionsPass {
    fn run(cfg: &mut Cfg) -> Result<(), Box<CfgError>> {
        // PASS 3:
        // --------------------
        // Eliminate nexts and prevs for dead code

        let nodes = cfg.nodes();
        let mut changed = true;
        while changed {
            changed = false;
            let old = nodes.clone();
            for node in nodes {
                if node.is_return() || node.is_any_entry() || node.might_terminate() {
                    continue;
                }
                // If the node has no nexts, remove it from the prevs of all its prevs
                if node.nexts().is_empty() {
                    for prev in node.prevs().clone() {
                        prev.remove_next(node);
                    }
                    node.clear_prevs();
                }

                // If the node has no prevs, remove it from the nexts of all its nexts
                if node.prevs().is_empty() {
                    for next in node.nexts().clone() {
                        next.remove_prev(node);
                    }
                    node.clear_nexts();
                }
            }
            if &old != nodes {
                changed = true;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use std::rc::Rc;

    use super::*;
    use crate::{
        cfg::CfgNode,
        gen::NodeDirectionPass,
        parser::RVStringParser,
        passes::{CfgError, GenerationPass},
    };

    fn run_pass(text: &str) -> Result<Vec<Rc<CfgNode>>, Box<CfgError>> {
        let (nodes, error) = RVStringParser::parse_from_text(text);
        assert_eq!(error.len(), 0);
        let mut cfg = Cfg::new(nodes).unwrap();
        NodeDirectionPass::run(&mut cfg)?;
        EliminateDeadCodeDirectionsPass::run(&mut cfg)?;
        Ok(cfg.iter().collect())
    }

    #[test]
    fn test_immediate_exit() {
        let input = "\
            main:
            li a7, 10
            ecall
            ";
        let cfg = run_pass(input).unwrap();
        assert_eq!(cfg.len(), 3);
        assert!(cfg[0].prevs().is_empty());
        assert!(cfg[0].nexts().len() == 1);
        assert!(cfg[1].prevs().len() == 1);
        assert!(cfg[1].nexts().len() == 1);
        assert!(cfg[2].prevs().len() == 1);
        assert!(cfg[2].nexts().is_empty());
    }
}
