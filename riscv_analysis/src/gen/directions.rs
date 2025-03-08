use std::rc::Rc;

use crate::{
    cfg::Cfg,
    parser::InstructionProperties,
    passes::{CfgError, GenerationPass},
};

/// Calculate the next and previous nodes for each node in the CFG.
///
/// This allows for easier (and required) traversal of the CFG.
/// This must be run before most passes.
pub struct NodeDirectionPass;
impl GenerationPass for NodeDirectionPass {
    fn run(cfg: &mut Cfg) -> Result<(), Box<CfgError>> {
        let mut prev = None;
        for node in cfg.iter() {
            // If node jumps to another node, add it to the nexts of the current node and the prevs of the node it jumps to.
            if let Some(label) = node.jumps_to() {
                let jump_to_node = cfg
                    .iter()
                    .find(|n| n.labels.contains(&label))
                    .ok_or_else(|| CfgError::UnexpectedError)?;

                node.insert_next(Rc::clone(&jump_to_node));
                jump_to_node.insert_prev(Rc::clone(&node));
            }

            // Linearly scan for nexts and prevs
            if let Some(prev) = prev {
                node.insert_prev(Rc::clone(&prev));
                prev.insert_next(Rc::clone(&node));
            }

            // Set previous node to current node, if it is not a return
            prev = if node.is_return() || node.is_unconditional_jump() {
                None
            } else {
                Some(Rc::clone(&node))
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
        parser::RVStringParser,
        passes::{CfgError, GenerationPass},
    };

    fn run_pass(text: &str) -> Result<Vec<Rc<CfgNode>>, Box<CfgError>> {
        let (nodes, error) = RVStringParser::parse_from_text(text);
        assert_eq!(error.len(), 0);
        let mut cfg = Cfg::new(nodes.into()).unwrap();
        NodeDirectionPass::run(&mut cfg)?;
        Ok(cfg.iter().collect())
    }

    #[test]
    fn test_immediate_exit() {
        let input = "\
            main:       \n\
            li a7, 10   \n\
            ecall       \n";
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
