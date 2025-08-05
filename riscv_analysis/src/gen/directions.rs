use std::rc::Rc;

use crate::cfg::CfgNode;
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
        let mut prev: Option<Rc<CfgNode>> = None;
        let mut edges_to_insert = Vec::new();
        for node in cfg.iter() {
            // If node jumps to another node, add it to the nexts of the current node and the prevs of the node it jumps to.
            if let Some(label) = node.jumps_to() {
                let jump_to_node = cfg
                    .iter()
                    .find(|n| n.labels().contains(&label))
                    .ok_or_else(|| CfgError::UnexpectedError)?;
                edges_to_insert.push((Rc::clone(&node), Rc::clone(&jump_to_node)));
            }

            // Linearly scan for nexts and prevs
            if let Some(p) = prev {
                edges_to_insert.push((Rc::clone(&p), Rc::clone(&node)));
            }

            // Set previous node to current node, if it is not a return
            prev = if node.is_return() || node.is_unconditional_jump() {
                None
            } else {
                Some(Rc::clone(&node))
            }
        }
        for (from, to) in edges_to_insert {
            cfg.insert_edge(&from, &to);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::ProgramEntryType;
    use crate::{
        parser::RVStringParser,
        passes::{CfgError, GenerationPass},
    };
    use itertools::Itertools;

    fn run_pass(text: &str) -> Result<Cfg, Box<CfgError>> {
        let parser_output = RVStringParser::parse_from_text(text);
        assert_eq!(parser_output.errors.len(), 0);
        let mut cfg = Cfg::new(parser_output, None, &ProgramEntryType::FirstInstruction).unwrap();
        NodeDirectionPass::run(&mut cfg)?;
        Ok(cfg)
    }

    #[test]
    fn test_immediate_exit() {
        let input = "\
            main:       \n\
            li a7, 10   \n\
            ecall       \n";
        let cfg = run_pass(input).unwrap();
        let nodes = cfg.nodes();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].labels().len(), 1);
        assert_eq!(
            nodes[0].labels().iter().next().unwrap().get().as_str(),
            "main"
        );
        assert!(cfg.get_prevs(nodes[0].as_ref()).len() == 0);
        assert!(cfg.get_nexts(nodes[0].as_ref()).len() == 1);
        assert!(cfg.get_prevs(nodes[1].as_ref()).len() == 1);
        assert!(cfg.get_nexts(nodes[1].as_ref()).len() == 0);
    }
}
