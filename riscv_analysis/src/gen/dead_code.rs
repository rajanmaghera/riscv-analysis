use crate::{
    cfg::Cfg,
    parser::InstructionProperties,
    passes::{CfgError, GenerationPass},
};
use std::rc::Rc;

pub struct EliminateDeadCodeDirectionsPass;
impl GenerationPass for EliminateDeadCodeDirectionsPass {
    fn run(cfg: &mut Cfg) -> Result<(), Box<CfgError>> {
        // PASS 3:
        // --------------------
        // Eliminate nexts and prevs for dead code

        let mut changed = true;
        while changed {
            changed = false;
            let mut edges_to_remove = Vec::new();
            for node in cfg.nodes() {
                if node.is_return() || node.is_any_entry() || node.might_terminate() {
                    continue;
                }
                // If the node has no nexts, remove it from the prevs of all its prevs
                if cfg.get_nexts(node.as_ref()).len() == 0 {
                    for prev in cfg.get_prevs(node.as_ref()) {
                        edges_to_remove.push((Rc::clone(prev), Rc::clone(node)));
                    }
                }

                // If the node has no prevs, remove it from the nexts of all its nexts
                if cfg.get_prevs(node.as_ref()).len() == 0 {
                    for next in cfg.get_nexts(node.as_ref()) {
                        edges_to_remove.push((Rc::clone(node), Rc::clone(next)));
                    }
                }
            }
            for (from, to) in edges_to_remove {
                changed |= cfg.remove_edge(from.as_ref(), to.as_ref());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use std::rc::Rc;

    use super::*;
    use crate::parser::ProgramEntryType;
    use crate::{
        cfg::CfgNode,
        gen::NodeDirectionPass,
        parser::RVStringParser,
        passes::{CfgError, GenerationPass},
    };

    fn run_pass(text: &str) -> Result<Vec<Rc<CfgNode>>, Box<CfgError>> {
        let parser_output = RVStringParser::parse_from_text(text);
        assert_eq!(parser_output.errors.len(), 0);
        let mut cfg = Cfg::new_with_predefined_call_names(
            parser_output,
            None,
            &ProgramEntryType::FirstInstruction,
        )
        .unwrap();
        NodeDirectionPass::run(&mut cfg)?;
        EliminateDeadCodeDirectionsPass::run(&mut cfg)?;
        Ok(cfg.iter().collect())
    }
}
