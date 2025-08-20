use uuid::Uuid;

use crate::new_impl::{contains_basic_blocks::ContainsBasicBlocks, risc_v_implementation::{RealBasicBlock, RealFunction, RealInst}};

pub trait ContainsFunctions: ContainsBasicBlocks {
    fn get_function_by_id(&self, function_id: &Uuid) -> Option<&RealFunction>;
    fn get_function_of_inst(&self, inst: &RealInst) -> Option<&RealFunction>;
    fn get_function_of_inst_by_id(&self, inst_id: &Uuid) -> Option<&RealFunction>;
    fn get_function_of_basic_block(&self, basic_block: &RealBasicBlock) -> Option<&RealFunction>;
    fn get_function_of_basic_block_by_id(&self, basic_block_id: &Uuid) -> Option<&RealFunction>;
}
