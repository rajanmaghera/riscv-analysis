use crate::cfg::AnnotatedCFG;

use super::{
    CalleeSavedGarbageReadCheck, CalleeSavedRegisterCheck, ControlFlowCheck, DeadValueCheck,
    EcallCheck, GarbageInputValueCheck, PassError, SaveToZeroCheck, StackCheckPass,
};

pub trait Pass {
    fn run(&self, cfg: &AnnotatedCFG, errors: &mut Vec<PassError>);
}

pub struct Manager {
    passes: Vec<Box<dyn Pass>>,
}

impl Manager {
    pub fn new() -> Manager {
        Manager {
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

    pub fn run(&self, cfg: &AnnotatedCFG) -> Vec<PassError> {
        let mut errors = Vec::new();
        for pass in &self.passes {
            pass.run(cfg, &mut errors);
        }
        errors
    }
}
