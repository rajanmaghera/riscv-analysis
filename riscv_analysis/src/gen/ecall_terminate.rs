use crate::passes::CFGError;
use crate::passes::GenerationPass;

pub struct EcallTerminationPass;
impl GenerationPass for EcallTerminationPass {
    fn run(cfg: &mut crate::cfg::Cfg) -> Result<(), Box<CFGError>> {
        for node in &*cfg {
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
