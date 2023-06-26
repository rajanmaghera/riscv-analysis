use crate::{
    cfg::CFG,
    passes::{CFGError, GenerationPass},
};

pub struct EliminateDeadCodeDirectionsPass;
impl GenerationPass for EliminateDeadCodeDirectionsPass {
    fn run(cfg: &mut CFG) -> Result<(), Box<CFGError>> {
        // PASS 3:
        // --------------------
        // Eliminate nexts and prevs for dead code

        let nodes = &cfg.nodes;
        let mut changed = true;
        while changed {
            changed = false;
            for node in nodes {
                if node.node.is_return() || node.node.is_any_entry() {
                    continue;
                }
                // If the node has no nexts, remove it from the prevs of all its prevs
                if node.nexts().is_empty() {
                    for prev in node.prevs().clone() {
                        prev.remove_next(node);
                    }
                    node.clear_prevs();
                    changed = true;
                }

                // If the node has no prevs, remove it from the nexts of all its nexts
                if node.prevs().is_empty() {
                    for next in node.nexts().clone() {
                        next.remove_prev(node);
                    }
                    node.clear_nexts();
                    changed = true;
                }
            }
        }

        Ok(())
    }
}
