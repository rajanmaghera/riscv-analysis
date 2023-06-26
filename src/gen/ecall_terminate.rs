use crate::passes::GenerationPass;

pub struct EcallTerminationPass;
impl GenerationPass for EcallTerminationPass {
    fn run(cfg: &mut crate::cfg::CFG) -> Result<(), crate::passes::CFGError> {
        for node in cfg.into_iter() {
            if node.is_program_exit() {
                for temp_node in node.nexts().clone() {
                    temp_node.remove_prev(&node);
                }
                node.clear_nexts();
            }
        }
        Ok(())
    }
}
