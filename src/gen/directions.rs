use std::rc::Rc;

use crate::{
    cfg::CFG,
    passes::{CFGError, GenerationPass},
};

pub struct NodeDirectionPass;
impl GenerationPass for NodeDirectionPass {
    fn run(cfg: &mut CFG) -> Result<(), CFGError> {
        // PASS 2:
        // --------------------
        // Calculate directions

        let nodes = &cfg.nodes;
        let mut prev = None;
        for node in nodes {
            // If node jumps to another node, add it to the nexts of the current node and the prevs of the node it jumps to.
            if let Some(label) = node.node.jumps_to() {
                let jump_to_node = nodes
                    .iter()
                    .find(|n| n.labels.contains(&label))
                    .ok_or_else(|| CFGError::UnexpectedError)?;

                node.insert_next(Rc::clone(jump_to_node));
                jump_to_node.insert_prev(Rc::clone(&node));
            }

            // Linearly scan for nexts and prevs
            if let Some(prev) = prev {
                node.insert_prev(Rc::clone(&prev));
                prev.insert_next(Rc::clone(&node));
            }

            // Set previous node to current node, if it is not a return
            prev = if node.node.is_return() {
                None
            } else {
                Some(Rc::clone(&node))
            }
        }

        Ok(())
    }
}
