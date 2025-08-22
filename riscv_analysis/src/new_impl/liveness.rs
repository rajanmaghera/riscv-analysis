use crate::new_impl::calling_convention::CallingConventionISA;
use crate::new_impl::gen_kill::HasGenKill;
use crate::new_impl::instruction_like::{InstructionLike, InstructionLikeInProg};
use crate::new_impl::isa::ISA;
use crate::new_impl::limited_element_set::LimitedElementSet;
use crate::new_impl::register_like::RegisterLike;
use crate::new_impl::set_combination::SetCombination;
use crate::new_impl::set_combination_fold::SetCombinationFold;
use crate::new_impl::set_traits::SetTraits;
use crate::new_impl::worklist::WorklistIter;

pub trait CanGenerateLiveness<T: InstructionLike> {
    /// Set the live in and out sets for every item to be empty.
    fn initialize_liveness(&self);

    /// Get live in for `item`.
    fn get_live_in(&self, item: &T) -> <<T as InstructionLike>::Register as LimitedElementSet>::ArrayType;

    /// Get live out for `item`.
    fn get_live_out(&self, item: &T) -> <<T as InstructionLike>::Register as LimitedElementSet>::ArrayType;

    /// Set live in and out for `item`, returning if either were mutated or not
    fn set_live_in_out(
        &self,
        item: &T,
        live_in: <<T as InstructionLike>::Register as LimitedElementSet>::ArrayType,
        live_out: <<T as InstructionLike>::Register as LimitedElementSet>::ArrayType
    ) -> bool;

    fn get_prevs<'a>(&'a self, item: &T) -> impl Iterator<Item = &'a T> where T: 'a;

    fn get_nexts<'a>(&'a self, item: &T) -> impl Iterator<Item = &'a T> where T: 'a;

    /// Return the nodes that a backwards analysis should begin its worklist with.
    fn get_leaf_and_loop_nodes<'a>(&'a self) -> impl Iterator<Item = &'a T> where T: 'a;

    fn get_return_instructions_for_function_that_this_inst_targets<'a>(
        &'a self,
        node: &T,
    ) -> impl Iterator<Item = &'a T> where T: 'a;

    fn get_target_instruction_of_function_call_instruction(&self, item: &T) -> &T;

    fn get_function_call_instructions_that_target_this<'a>(
        &'a self,
        item: &T,
    ) -> impl Iterator<Item = &'a T> where T: 'a;

    fn is_first_instruction_in_function(&self, item: &T) -> bool;
}

#[derive(Clone)]
pub struct LivenessInfo<T: RegisterLike> {
    live_in: T::ArrayType,
    live_out: T::ArrayType,
}

impl<T: RegisterLike> LivenessInfo<T> {
    pub fn new() -> Self {
        Self {
            live_out: T::ArrayType::new(),
            live_in: T::ArrayType::new(),
        }
    }

    // Set live in and out, returning if either were mutated or not
    pub fn set_live_in_out(&mut self, live_in: T::ArrayType, live_out: T::ArrayType) -> bool {
        self.set_live_in(live_in) || self.set_live_out(live_out)
    }

    // Set live in, returning if it was mutated or not
    pub fn set_live_in(&mut self, new: T::ArrayType) -> bool {
        let live_in_changed = (self.live_in == new);
        self.live_in = new;
        live_in_changed
    }

    // Set live out, returning if it was mutated or not
    pub fn set_live_out(&mut self, new: T::ArrayType) -> bool {
        let live_out_changed = (self.live_out == new);
        self.live_out = new;
        live_out_changed
    }

    pub fn get_live_in(&self) -> T::ArrayType {
        self.live_in.clone()
    }

    pub fn get_live_out(&self) -> T::ArrayType {
        self.live_out.clone()
    }
}

fn generate_liveness_information<CC: CallingConventionISA, CFG: CanGenerateLiveness<<CC as ISA>::Instruction>> (
    cfg: &CFG,
) where
    <CC as ISA>::Instruction: InstructionLikeInProg + HasGenKill,
{
    cfg.initialize_liveness();
    let worklist = WorklistIter::new(cfg.get_leaf_and_loop_nodes());
    for node in &worklist {
        let live_out = if CC::is_function_return_instruction(node) {
            cfg.get_function_call_instructions_that_target_this(node)
                .map(|node| cfg.get_live_out(node))
                .intersection_all()
                .intersection(&CC::return_registers())
        } else {
            cfg.get_nexts(node)
                .map(|node| cfg.get_live_in(node))
                .union_all()
        };

        let live_in = if CC::is_function_call_instruction(node) {
            if live_out != cfg.get_live_out(node) {
                worklist
                    .add_many(cfg.get_return_instructions_for_function_that_this_inst_targets(node))
            }
            cfg.get_live_in(cfg.get_target_instruction_of_function_call_instruction(node))
                .intersection(&CC::argument_registers())
                .union(&live_out.clone().difference(&CC::caller_saved_registers()))
        } else {
            live_out.difference(&node.get_kill()).union(&node.get_gen())
        };
        if cfg.set_live_in_out(node, live_in, live_out) {
            worklist.add_many(cfg.get_prevs(node));
        }
        if cfg.is_first_instruction_in_function(node) {
            worklist.add_many(cfg.get_function_call_instructions_that_target_this(node));
        }
    }
}
