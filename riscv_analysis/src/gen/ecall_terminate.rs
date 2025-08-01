use crate::passes::CfgError;
use crate::passes::GenerationPass;
use std::rc::Rc;

pub struct EcallTerminationPass;
impl GenerationPass for EcallTerminationPass {
    fn run(cfg: &mut crate::cfg::Cfg) -> Result<(), Box<CfgError>> {
        let mut edges_to_remove = Vec::new();
        for node in cfg.iter() {
            if node.is_program_exit() {
                for temp_node in cfg.get_nexts(node.as_ref()) {
                    edges_to_remove.push((Rc::clone(&node), Rc::clone(temp_node)));
                }
            }
        }
        for (from, to) in edges_to_remove {
            cfg.remove_edge(from.as_ref(), to.as_ref());
        }
        Ok(())
    }
}
