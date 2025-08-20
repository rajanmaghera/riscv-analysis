use uuid::Uuid;

use crate::new_impl::{contains_instructions::ContainsInstructions, risc_v_implementation::{RealBasicBlock, RealInst}};

pub trait ContainsBasicBlocks: ContainsInstructions {
    fn get_basic_block_by_id(&self, block_id: &Uuid) -> Option<&RealBasicBlock>;
    fn get_basic_block_of_inst(&self, inst: &RealInst) -> Option<&RealBasicBlock>;
    fn get_basic_block_of_inst_by_id(&self, inst_id: &Uuid) -> Option<&RealBasicBlock>;
}
