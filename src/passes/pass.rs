use crate::cfg::AnnotatedCFG;

use super::{
    CalleeSavedGarbageReadCheck, CalleeSavedRegisterCheck, ControlFlowCheck, DeadValueCheck,
    EcallCheck, GarbageInputValueCheck, PassErrors, SaveToZeroCheck, StackCheckPass,
};

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
                Box::new(GarbageInputValueCheck),
                Box::new(StackCheckPass),
                Box::new(CalleeSavedRegisterCheck),
                Box::new(CalleeSavedGarbageReadCheck),
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
