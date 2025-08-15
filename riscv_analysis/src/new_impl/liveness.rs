use crate::new_impl::calling_convention::CallingConventionISA;
use crate::new_impl::digraph::{AnnotatedDigraph, Digraph};
use crate::new_impl::gen_kill::HasGenKill;
use crate::new_impl::instruction_like::{InstructionLike, InstructionLikeInProg};
use crate::new_impl::isa::ISA;
use crate::new_impl::register_like::RegisterLike;
use crate::new_impl::set_combination::SetCombination;
use crate::new_impl::set_combination_fold::SetCombinationFold;
use crate::new_impl::set_traits::SetTraits;
use crate::new_impl::worklist::WorklistIter;
use crate::new_impl::HasRelatedIterator;

impl<T: InstructionLikeInProg> Digraph<T> {
    /// Return the nodes that a backwards analysis should begin
    /// its worklist with.
    fn get_leaf_and_loop_nodes(&self) -> impl Iterator<Item = &T> {
        // TODO fix
        self.nodes.iter()
    }

    fn get_return_instructions_for_function_that_this_inst_targets(
        &self,
        node: &T,
    ) -> impl Iterator<Item = &T> {
        // TODO fix
        self.nodes.iter()
    }

    fn get_target_instruction_of_function_call_instruction(&self, item: &T) -> &T {
        todo!()
    }
    fn get_function_call_instructions_that_target_this(
        &self,
        item: &T,
    ) -> impl Iterator<Item = &T> {
        // TODO fix
        self.nodes.iter()
    }

    pub fn is_first_instruction_in_function(&self, item: &T) -> bool {
        todo!()
    }
}

#[derive(Clone)]
pub struct LivenessInfo<T: RegisterLike> {
    live_in: T::ArrayType,
    live_out: T::ArrayType,
}

impl<T: RegisterLike> LivenessInfo<T> {
    fn new() -> Self {
        Self {
            live_out: T::ArrayType::new(),
            live_in: T::ArrayType::new(),
        }
    }

    // Set live in and out, returning if either were mutated or not
    fn set_live_in_out(&mut self, live_in: T::ArrayType, live_out: T::ArrayType) -> bool {
        self.set_live_in(live_in) || self.set_live_out(live_out)
    }

    // Set live in, returning if it was mutated or not
    fn set_live_in(&mut self, new: T::ArrayType) -> bool {
        let live_in_changed = (self.live_in == new);
        self.live_in = new;
        live_in_changed
    }

    // Set live out, returning if it was mutated or not
    fn set_live_out(&mut self, new: T::ArrayType) -> bool {
        let live_out_changed = (self.live_out == new);
        self.live_out = new;
        live_out_changed
    }

    fn get_live_in(&self) -> T::ArrayType {
        self.live_in.clone()
    }

    fn get_live_out(&self) -> T::ArrayType {
        self.live_out.clone()
    }
}

fn generate_liveness_information<CC: CallingConventionISA>(
    cfg: &Digraph<<CC as ISA>::Instruction>,
) -> AnnotatedDigraph<
    <CC as ISA>::Instruction,
    LivenessInfo<<<CC as ISA>::Instruction as InstructionLike>::Register>,
>
where
    <CC as ISA>::Instruction: InstructionLikeInProg + HasGenKill,
{
    let mut liveness = AnnotatedDigraph::new(
        cfg,
        LivenessInfo::<<<CC as ISA>::Instruction as InstructionLike>::Register>::new(),
    );
    let worklist = WorklistIter::new(cfg.get_leaf_and_loop_nodes());
    for node in &worklist {
        let live_out = if CC::is_function_return_instruction(node) {
            cfg.get_function_call_instructions_that_target_this(node)
                .get_related(&liveness)
                .map(LivenessInfo::get_live_out)
                .intersection_all()
                .intersection(&CC::return_registers())
        } else {
            liveness
                .get_nexts(node)
                .map(LivenessInfo::get_live_in)
                .union_all()
        };

        let live_in = if CC::is_function_call_instruction(node) {
            if live_out != liveness.get(node).live_out {
                worklist
                    .add_many(cfg.get_return_instructions_for_function_that_this_inst_targets(node))
            }
            liveness
                .get(cfg.get_target_instruction_of_function_call_instruction(node))
                .live_in
                .intersection(&CC::argument_registers())
                .union(&live_out.clone().difference(&CC::caller_saved_registers()))
        } else {
            live_out.difference(&node.get_kill()).union(&node.get_gen())
        };
        if liveness.get_mut(node).set_live_in_out(live_in, live_out) {
            worklist.add_many(cfg.get_prevs(node));
        }
        if cfg.is_first_instruction_in_function(node) {
            worklist.add_many(cfg.get_function_call_instructions_that_target_this(node));
        }
    }
    liveness
}
