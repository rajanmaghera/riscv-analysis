use crate::cfg::{AnnotatedCFG, CFG};

use super::{DeadValueCheck, PassErrors, SaveToZeroCheck};

pub trait Pass {
    fn run(&self, cfg: &AnnotatedCFG) -> Result<(), PassErrors>;
}

pub struct PassManager {
    passes: Vec<Box<dyn Pass>>,
}

impl PassManager {
    pub fn new() -> PassManager {
        PassManager {
            passes: vec![Box::new(SaveToZeroCheck), Box::new(DeadValueCheck)],
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
