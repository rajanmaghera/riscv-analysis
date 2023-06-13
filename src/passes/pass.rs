use crate::cfg::AnnotatedCFG;

use super::{ControlFlowCheck, DeadValueCheck, EcallCheck, PassErrors, SaveToZeroCheck};

pub trait Pass {
    fn run(&self, cfg: &AnnotatedCFG) -> Result<(), PassErrors>;
}

pub struct PassManager {
    passes: Vec<Box<dyn Pass>>,
}

impl PassManager {
    pub fn new() -> PassManager {
        PassManager {
            passes: vec![
                Box::new(SaveToZeroCheck),
                Box::new(DeadValueCheck),
                Box::new(EcallCheck),
                Box::new(ControlFlowCheck),
            ],
        }
    }

    pub fn run(&self, cfg: AnnotatedCFG) -> Result<(), PassErrors> {
        let mut errors = Vec::new();
        for pass in self.passes.iter() {
            match pass.run(&cfg) {
                Ok(_) => (),
                Err(mut pass_errors) => {
                    errors.append(&mut pass_errors.errors);
                }
            }
        }
        if errors.len() > 0 {
            Err(PassErrors { errors })
        } else {
            Ok(())
        }
    }
}
