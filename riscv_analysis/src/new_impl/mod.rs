use crate::cfg::Cfg;
use crate::new_impl::digraph::AnnotatedDigraph;
use crate::new_impl::set_combination::SetCombination;
use crate::new_impl::worklist::WorklistIter;
use crate::parser::HasIdentity;
use calling_convention::CallingConventionISA;
use digraph::Digraph;
use gen_kill::HasGenKill;
use instruction_like::{InstructionLike, InstructionLikeInProg};
use isa::ISA;
use limited_element_set::LimitedElementSet;
use liveness::LivenessInfo;
use register_like::RegisterLike;
use risc_v_implementation::{RealBasicBlock, RealFunction, RealInst, RealInstList};
use set_combination_fold::SetCombinationFold;
use set_traits::SetTraits;
use std::collections::HashMap;
use uuid::Uuid;

mod calling_convention;
mod digraph;
mod gen_kill;
mod has_labels;
mod instruction_like;
mod isa;
mod limited_element_set;
mod liveness;
mod register_like;
mod risc_v_implementation;
mod set_combination;
mod set_combination_fold;
mod set_traits;
mod worklist;

impl HasIdentity for Uuid {
    fn id(&self) -> Uuid {
        *self
    }
}

// This trait allows an iterator of a node type
// to map to a related type
trait HasRelatedIterator<'a, T: HasIdentity + 'a>: Iterator<Item = &'a T> {
    fn get_related<U>(self, other: &'a AnnotatedDigraph<'a, T, U>) -> impl Iterator<Item = &'a U>
    where
        Self: Sized,
        U: Clone,
    {
        self.map(|x| other.get(x))
    }
}

impl<'a, T, U: HasIdentity + 'a> HasRelatedIterator<'a, U> for T where T: Iterator<Item = &'a U> {}

#[derive(Clone)]
struct ValueInfo<T: CanPerformSymbolicAnalysis> {
    live_in: T::LatticeRepresentation,
    live_out: T::LatticeRepresentation,
}

impl<T: CanPerformSymbolicAnalysis> ValueInfo<T> {
    fn new() -> Self {
        Self {
            live_out: T::LatticeRepresentation::undefined(),
            live_in: T::LatticeRepresentation::undefined(),
        }
    }

    // Set live in and out, returning if either were mutated or not
    fn set_live_in_out(
        &mut self,
        live_in: T::LatticeRepresentation,
        live_out: T::LatticeRepresentation,
    ) -> bool {
        todo!()
    }

    // Set live in, returning if it was mutated or not
    fn set_live_in(&mut self, new: T::LatticeRepresentation) -> bool {
        todo!()
    }

    // Set live out, returning if it was mutated or not
    fn set_live_out(&mut self, new: T::LatticeRepresentation) -> bool {
        todo!()
    }

    fn get_live_in(&self) -> T::LatticeRepresentation {
        self.live_in.clone()
    }

    fn get_live_out(&self) -> T::LatticeRepresentation {
        self.live_out.clone()
    }
}

impl<T: InstructionLikeInProg> Digraph<T> {
    /// Return the nodes that a backwards analysis should begin
    /// its worklist with.
    fn get_all_first_instructions(&self) -> impl Iterator<Item = &T> {
        // TODO fix
        self.nodes.iter()
    }
}

fn generate_value_information<CC: CallingConventionISA>(
    cfg: &Digraph<<CC as ISA>::Instruction>,
) -> AnnotatedDigraph<<CC as ISA>::Instruction, ValueInfo<<CC as ISA>::Instruction>>
where
    <CC as ISA>::Instruction:
        InstructionLikeInProg + HasGenKill + CanPerformSymbolicAnalysis + Clone,
{
    let mut values = AnnotatedDigraph::new(cfg, ValueInfo::<<CC as ISA>::Instruction>::new());
    let worklist = WorklistIter::new(cfg.get_all_first_instructions());
    for node in &worklist {
        let mut live_in = values
            .get_prevs(node)
            .map(ValueInfo::get_live_out)
            .join_all();
        if cfg.is_first_instruction_in_function(node) {
            todo!()
        }
    }
    values
}

/// A trait representing an element type.
///
/// It is the responsibility of the implementer that the join/meet operations
/// are monotonic.
trait IsLatticeElementType: Clone {
    /// Perform the Least Upper Bound (LUB) operation, becoming less precise
    fn join(&mut self, other: &Self);
    /// Perform the Greatest Lower Bound (GLB) operation, becoming less precise
    fn meet(&mut self, other: &Self);
    /// Get bottom-most undefined (most precise) element
    fn undefined() -> Self;
}

trait LatticeElementFold<U: IsLatticeElementType>: IntoIterator<Item = U> {
    fn join_all(self) -> Self::Item
    where
        Self: Sized,
    {
        self.into_iter()
            .reduce(|mut a, b| {
                a.join(&b);
                a
            })
            .unwrap_or_else(|| U::undefined())
    }

    fn meet_all(self) -> Self::Item
    where
        Self: Sized,
    {
        self.into_iter()
            .reduce(|mut a, b| {
                a.meet(&b);
                a
            })
            .unwrap_or_else(|| U::undefined())
    }
}
impl<U: IsLatticeElementType, V: Iterator<Item = U>> LatticeElementFold<U> for V {}

///
trait CanPerformSymbolicAnalysis: InstructionLike {
    type LatticeRepresentation: IsLatticeElementType;
    fn perform_operation(&self, element: &mut Self::LatticeRepresentation);
}

// impl<'a> RealCfg<'a> {
//     fn new(inst_list: &RealInstList) -> RealCfg<'a> {
//         todo!()
//     }
// }

trait RealCfgT {
    fn iter_over_source(&self) -> impl Iterator<Item = &RealInst>;
    fn iter_functions(&self) -> impl Iterator<Item = &RealFunction>;
    fn iter_basic_blocks(&self) -> impl Iterator<Item = &RealBasicBlock>;
}
