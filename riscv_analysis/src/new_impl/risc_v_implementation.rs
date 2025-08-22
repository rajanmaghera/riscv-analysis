use crate::cfg::Cfg;
use crate::new_impl::contains_basic_blocks::ContainsBasicBlocks;
use crate::new_impl::contains_functions::ContainsFunctions;
use crate::new_impl::contains_instructions::ContainsInstructions;
use crate::new_impl::digraph::{Digraph, DigraphNodes};
use crate::new_impl::has_labels::HasLabels;
use crate::new_impl::instruction_like::{InstructionLike, InstructionLikeInProg};
use crate::new_impl::interprocedural_instruction_iterator::InterproceduralInstructionIterator;
use crate::new_impl::intrablock_instruction_iterator::IntrablockInstructionIterator;
use crate::new_impl::intraprocedural_block_iterator::IntraproceduralBlockIterator;
use crate::new_impl::intraprocedural_instruction_iterator::IntraproceduralInstructionIterator;
use crate::new_impl::limited_element_set::LimitedElementSet;
use crate::new_impl::liveness::{CanGenerateLiveness, LivenessInfo};
use crate::parser::HasIdentity;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::iter::{self, Enumerate, Peekable};
use std::ops::Deref;
use uuid::Uuid;

pub struct RealInst {
    labels: Vec<String>,
    id: Uuid,
}


impl RealInst {
    // Remove this, this is not the place it should be
    fn get_idx(&self) -> usize {
        todo!()
    }

    fn is_ret(&self) -> bool {
        todo!()
    }

    fn is_call(&self) -> bool {
        todo!()
    }

    fn is_call_target(&self) -> bool {
        todo!()
    }

    fn is_return_target(&self) -> bool {
        todo!()
    }

    fn get_target_label(&self) -> Option<String> {
        todo!()
    }
}

impl InstructionLikeInProg for RealInst {}

impl HasLabels for RealInst {
    fn get_labels(&self) -> impl Iterator<Item = &String> {
        self.labels.iter()
    }

    fn has_label(&self, label: &impl ToString) -> bool {
        self.labels.contains(&label.to_string())
    }
}

impl HasIdentity for RealInst {
    fn id(&self) -> Uuid {
        self.id
    }
}

impl PartialEq for RealInst {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Hash for RealInst {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl Eq for RealInst {}

pub struct RealInstList {
    list: Vec<RealInst>,
    inst_id_to_index_map: HashMap<Uuid, usize>,
}

impl RealInstList {
    /// Construct a new `RealInstList`.
    fn new(list: Vec<RealInst>) -> Self {
        let inst_id_to_index_map: HashMap<Uuid, usize> = list.iter().enumerate().map(|(idx, inst)| (inst.id(), idx)).collect();
        RealInstList { list, inst_id_to_index_map }
    }

    /// Get the labels of the instruction at `idx`.
    fn get_label_names(&self, idx: usize) -> impl Iterator<Item = &String> {
        self.list[idx].get_labels()
    }

    /// Return if this instruction is a branch target.
    ///
    /// This includes function calls.
    fn is_any_jump_target(&self, idx: usize) -> bool {
        todo!()
    }

    /// Return if this instruction is a branch target of only function calls.
    fn is_function_call_target(&self, idx: usize) -> bool {
        todo!()
    }

    /// Return the next item in the list.
    fn next_in_source(&self, idx: usize) -> Option<&RealInst> {
        self.list.get(idx + 1)
    }

    fn is_uncond_function_call(&self, idx: usize) -> bool {
        todo!()
    }

    /// If this is a function call, get the target
    fn if_uncond_function_call_get_target(&self, idx: usize) -> Option<&RealInst> {
        todo!()
    }

    /// Return the instruction that a function should return to after a function call,
    /// if this is a function call.
    ///
    /// You should not use the `next_in_source` function to determine this, as the
    /// instruction may set its return address in a different way.
    fn if_uncond_function_call_get_return_target(&self, idx: usize) -> Option<&RealInst> {
        todo!()
    }

    fn is_uncond_non_function_call_jump(&self, idx: usize) -> bool {
        todo!()
    }

    fn if_uncond_non_function_call_jump_get_target(&self, idx: usize) -> Option<&RealInst> {
        todo!()
    }

    fn is_any_uncond_jump(&self, idx: usize) -> bool {
        todo!()
    }

    fn if_any_uncond_jump_get_target(&self, idx: usize) -> Option<&RealInst> {
        todo!()
    }

    fn is_any_cond_jump(&self, idx: usize) -> bool {
        todo!()
    }
    fn if_any_cond_jump_get_target(&self, idx: usize) -> Option<&RealInst> {
        todo!()
    }

    fn if_any_cond_jump_get_fallthrough(&self, idx: usize) -> Option<&RealInst> {
        todo!()
    }

    fn is_any_non_function_call_jump(&self, idx: usize) -> bool {
        self.is_any_cond_jump(idx) || self.is_uncond_non_function_call_jump(idx)
    }

    /// Return the next instruction in the graph, including unconditional jump
    /// targets. If this is a conditional branch, return the fallthrough.
    fn get_next_or_any_cond_jump_fallthrough(&self, idx: usize) -> Option<&RealInst> {
        todo!()
    }
}

/// BASIC BLOCK

/// A basic block.
///
/// A basic block contains 1 or more instructions.
///
/// A basic block must be sequential in memory.
pub struct RealBasicBlock<'a> {
    insts: Vec<&'a RealInst>,
    inst_ids_to_indices: HashMap<Uuid, usize>,
}

struct RealBasicBlockIterator<'a> {
    iterator: core::slice::Iter<'a, &'a RealInst>,
}

impl<'a> RealBasicBlock<'a> {
    pub fn iter(&'a self) -> impl Iterator<Item = &'a RealInst> {
        RealBasicBlockIterator {
            iterator: self.insts.iter(),
        }
    }

    /// Construct a new basic block.
    ///
    /// The input instructions should not be empty. This function
    /// should NOT be published.
    fn new(first_inst: &'a RealInst) -> Self {
        let mut inst_ids_to_indices = HashMap::new();
        inst_ids_to_indices.insert(first_inst.id(), 0);
        Self {
            insts: vec![first_inst],
            inst_ids_to_indices,
        }
    }

    /// Add a new instruction to the end of the basic block.
    ///
    /// This function should NOT be published.
    fn push_inst(&mut self, inst: &'a RealInst) {
        self.inst_ids_to_indices.insert(inst.id(), self.insts.len());
        self.insts.push(inst);
    }

    /// Get the first instruction.
    fn get_first_instruction(&self) -> &RealInst {
        self.insts.first().expect("First instructions should exist")
    }

    /// Get the last instruction.
    ///
    /// This instruction will always exist, but it might not always be a
    /// control-flow instruction (ie. final instruction in the program that
    /// falls off the end).
    fn get_last_instruction(&self) -> &RealInst {
        self.insts.last().expect("Last instructions should exist")
    }

    /// Check if an instruction is the first instruction of this basic block.
    fn first_instruction_is(&self, inst: &RealInst) -> bool {
        inst == self.get_first_instruction()
    }

    /// Check if an instruction is the last instruction of this basic block.
    fn last_instruction_is(&self, inst: &RealInst) -> bool {
        inst == self.get_last_instruction()
    }

    /// Get the index of an instruction in this block.
    ///
    /// Panics if there is no instruction with the given id in this block.
    fn get_index_of_inst(&self, inst: &RealInst) -> usize {
        self.get_index_of_inst_by_id(&inst.id())
    }

    /// Get the index of an instruction in this block, given the instruction's id.
    ///
    /// Panics if there is no instruction with the given id in this block.
    fn get_index_of_inst_by_id(&self, inst_id: &Uuid) -> usize {
        *self.inst_ids_to_indices.get(inst_id)
        .expect("Instruction id should be in inst_ids_to_indices map")
    }

    /// Get the next instruction after the instruction with id `inst_id` in this block.
    ///
    /// Returns `None` if the instruction with id `inst_id` is the last instruction in the basic block
    /// and thus has no next instruction in the block.
    fn get_next_by_id(&self, inst_id: &Uuid) -> Option<&RealInst> {
        self.insts.get(self.get_index_of_inst_by_id(inst_id) + 1).map(|v| *v)
    }

    /// Get the previous instruction before the instruction with id `inst_id` in this block.
    ///
    /// Returns `None` if the instruction with id `inst_id` is the first instruction in the basic block
    /// and thus has no previous instruction in the block.
    fn get_prev_by_id(&self, inst_id: &Uuid) -> Option<&RealInst> {
        self.insts.get(self.get_index_of_inst_by_id(inst_id) - 1).map(|v| *v)
    }
}

impl<'a> HasLabels for RealBasicBlock<'a> {
    /// TODO naming may be confusing - only considers first instruction
    fn get_labels(&self) -> impl Iterator<Item = &String> {
        self.get_first_instruction().get_labels()
    }

    /// TODO naming may be confusing - only considers first instruction
    fn has_label(&self, label: &impl ToString) -> bool {
        self.get_first_instruction().has_label(label)
    }
}

impl<'a> PartialEq for RealBasicBlock<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<'a> Hash for RealBasicBlock<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl<'a> Eq for RealBasicBlock<'a> {}

impl<'a> Iterator for RealBasicBlockIterator<'a> {
    type Item = &'a RealInst;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iterator.next()?.deref())
    }
}

impl<'a> ContainsInstructions for RealBasicBlock<'a> {
    fn get_inst_by_id(&self, inst_id: &Uuid) -> &RealInst {
        *self.insts.get(
            self.get_index_of_inst_by_id(inst_id)
        )
        .expect("Instruction index should be within the bounds of the insts vector")
    }
}

impl<'a> IntrablockInstructionIterator for RealBasicBlock<'a> {
    /// Get the next instruction after `inst` in this block.
    ///
    /// Returns `None` if `inst` is the last instruction in the basic block
    /// and thus has no next instruction in the block.
    fn get_next_inst_intrablock(&self, inst: &RealInst) -> Option<&RealInst> {
        self.get_next_by_id(&inst.id())
    }

    /// Get the previous instruction before `inst` in this block.
    ///
    /// Returns `None` if `inst` is the first instruction in the basic block
    /// and thus has no previous instruction in the block.
    fn get_prev_inst_intrablock(&self, inst: &RealInst) -> Option<&RealInst> {
        self.get_prev_by_id(&inst.id())
    }
}

/// FUNCTION

pub struct RealFunction<'a> {
    digraph: Digraph<RealBasicBlock<'a>>,
    inst_ids_to_block_ids: HashMap<Uuid, Uuid>,
    entry_block_id: Uuid,
    exit_block_ids: HashSet<Uuid>, // may be empty if function never exits
    id: Uuid,
}

struct RealFunctionIterator<'a> {
    first: Option<&'a RealBasicBlock<'a>>,
    iterator: std::collections::hash_set::Iter<'a, RealBasicBlock<'a>>,
}

impl<'a> RealFunction<'a> {
    // Create a new function given its entry block.
    pub fn new(entry_block: RealBasicBlock<'a>) -> Self {
        let entry_block_id = entry_block.id();
        let inst_ids_to_block_ids: HashMap<Uuid, Uuid> = entry_block.iter().map(|inst| (inst.id(), entry_block.id())).collect();
        let mut function = RealFunction {
            digraph: Digraph::new(),
            inst_ids_to_block_ids: inst_ids_to_block_ids,
            entry_block_id,
            exit_block_ids: HashSet::new(),
            id: Uuid::new_v4(),
        };
        function.add_block(entry_block);
        function
    }

    // Add a block to this function without creating any edges.
    pub fn add_block(&mut self, block: RealBasicBlock<'a>) {
        self.digraph.add_node(block);
    }

    // Add a block to this function, creating edges from prevs to the provided block and from block to the provided nexts.
    pub fn add_block_with_edges(&mut self, block: RealBasicBlock<'a>, prevs: impl Iterator<Item = &'a RealBasicBlock<'a>>, nexts: impl Iterator<Item = &'a RealBasicBlock<'a>>) {
        prevs.for_each(|prev| self.digraph.edges.add_edge(prev, &block));
        nexts.for_each(|next| self.digraph.edges.add_edge(&block, next));
        self.digraph.add_node(block);
    }

    /// Remove a non-entry block from the function.
    ///
    /// Returns true if the block was successfully removed,
    /// meaning that it was not the entry block for the function,
    /// and false otherwise.
    pub fn remove_non_entry_block(&mut self, block: &RealBasicBlock<'a>) -> bool {
        // Cannot remove the entry block
        if self.block_is_entry(block) {
            return false;
        }

        if self.block_is_an_exit(block) {
            self.exit_block_ids.remove(&block.id());
        }
        self.digraph.remove_node(block);
        true
    }

    /// Get the entry block of the function.
    pub fn get_entry_block(&self) -> &RealBasicBlock {
        self.digraph.get_by_id(&self.entry_block_id)
    }

    /// Get the entry instruction of the function.
    pub fn get_entry_inst(&self) -> &RealInst {
        self.get_entry_block().get_first_instruction()
    }

    /// Get the exit blocks of the function.
    ///
    /// The returned iterator will contain no elements if this function never exits (eg. it loops infinitely).
    pub fn get_exit_blocks(&self) -> impl Iterator<Item = &RealBasicBlock> {
        self.exit_block_ids.iter().map(|b| self.digraph.get_by_id(b))
    }

    /// Get the exit instructions of the function.
    ///
    /// The returned iterator will contain no elements if this function never exits (eg. it loops infinitely).
    pub fn get_exit_insts(&self) -> impl Iterator<Item = &RealInst> {
        self.get_exit_blocks().map(|block| block.get_last_instruction())
    }

    /// Determine if this function has at least one exit.
    ///
    /// This will be false if the function never exits (eg. it loops infinitely).
    pub fn has_an_exit(&self) -> bool {
        !self.exit_block_ids.is_empty()
    }

    /// Get the blocks of this function that end with a return instruction.
    ///
    /// The returned iterator will contain no elements if this function has no blocks that end with a return instruction.
    pub fn get_return_blocks(&self) -> impl Iterator<Item = &RealBasicBlock> {
        self.exit_block_ids.iter().map(|b| self.digraph.get_by_id(b)).filter(
            |block| block.get_last_instruction().is_ret()
        )
    }

    /// Get the return instructions of the function.
    ///
    /// The returned iterator will contain no elements if this function has no return instructions.
    pub fn get_return_insts(&self) -> impl Iterator<Item = &RealInst> {
        self.exit_block_ids.iter().map(|b| self.digraph.get_by_id(b).get_last_instruction())
        .filter(|last_inst| last_inst.is_ret())
    }

    /// Determine if this function has at least one return.
    pub fn has_a_return(&self) -> bool {
        self.get_return_insts().next().is_some()
    }

    /// Check if the instruction is the entry point of this function.
    pub fn inst_is_entry(&self, inst: &RealInst) -> bool {
        self.get_entry_inst() == inst
    }

    /// Check if the block is the entry block of this function.
    pub fn block_is_entry(&self, block: &RealBasicBlock) -> bool {
        self.entry_block_id == block.id()
    }

    /// Check if the instruction is an exit point of this function.
    pub fn inst_is_an_exit(&self, inst: &RealInst) -> bool {
        self.get_exit_insts().collect::<HashSet<&RealInst>>().contains(inst)
    }

    /// Check if the block is an exit block of this function.
    pub fn block_is_an_exit(&self, block: &RealBasicBlock) -> bool {
        self.exit_block_ids.contains(&block.id())
    }

    /// Check if this function contains the given block.
    pub fn contains(&self, block: &RealBasicBlock) -> bool {
        self.digraph.contains(block)
    }

    /// Visits all instructions in the function, in arbitrary order.
    pub fn intraprocedural_iter(&self) -> impl Iterator<Item = &RealInst> {
        self.digraph.nodes.iter().flat_map(|block| block.iter())
    }
}

impl<'a> HasIdentity for RealFunction<'a> {
    /// Get the id of the function.
    fn id(&self) -> Uuid {
        self.id
    }
}

impl<'a> PartialEq for RealFunction<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<'a> Hash for RealFunction<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl<'a> Eq for RealFunction<'a> {}

impl<'a> ContainsInstructions for RealFunction<'a> {
    /// Given the id of an instruction in this function, get the instruction.
    ///
    /// Panics if there is no instruction with the given id in this function.
    fn get_inst_by_id(&self, inst_id: &Uuid) -> &RealInst {
        self.get_basic_block_of_inst_by_id(&inst_id).get_inst_by_id(&inst_id)
    }
}

impl<'a> ContainsBasicBlocks for RealFunction<'a> {
    /// Given the id of a basic block in this function, get the basic block.
    ///
    /// Panics if there is no block with the given id in this function.
    fn get_basic_block_by_id(&self, block_id: &Uuid) -> &RealBasicBlock {
        self.digraph.get_by_id(block_id)
    }

    /// Given an instruction in this function, get the basic block that contains it.
    ///
    /// Panics if the instruction is not in this function.
    fn get_basic_block_of_inst(&self, inst: &RealInst) -> &RealBasicBlock {
        self.get_basic_block_of_inst_by_id(&inst.id())
    }

    /// Given the id of an instruction in this function, get the basic block that contains it.
    ///
    /// Panics if there is no instruction with the given id in this function.
    fn get_basic_block_of_inst_by_id(&self, inst_id: &Uuid) -> &RealBasicBlock {
        self.get_basic_block_by_id(
            self.inst_ids_to_block_ids.get(&inst_id)
            .expect("Instruction id should be in inst_ids_to_block_ids map")
        )
    }
}

impl<'a> IntraproceduralInstructionIterator for RealFunction<'a> {
    /// Given an instruction `inst` in this function, get the next instructions of `inst`
    /// that are in this function.
    ///
    /// Returns an empty iterator if `inst` is a leaf node in the function (ie. if `inst` returns
    /// unconditionally or if `inst` exits the program unconditionally).
    fn get_next_insts_intraprocedural(&self, inst: &RealInst) -> HashSet<&RealInst> {
        let block = self.get_basic_block_of_inst(inst);
        if block.last_instruction_is(inst) {
            self.get_next_blocks_intraprocedural(block)
            .iter()
            .map(|b| b.get_first_instruction())
            .collect()
        } else {
            iter::once(block.get_next_inst_intrablock(inst).unwrap()).collect()
        }
    }

    /// Given an instruction `inst` in this function, get the previous instructions of `inst`
    /// that are in this function.
    ///
    /// Returns an empty iterator if `inst` is the entry point of the function.
    fn get_prev_insts_intraprocedural(&self, inst: &RealInst) -> HashSet<&RealInst> {
        let block = self.get_basic_block_of_inst(inst);
        if block.first_instruction_is(inst) {
            self.get_prev_blocks_intraprocedural(block)
            .iter()
            .map(|b| b.get_first_instruction())
            .collect()
        } else {
            iter::once(block.get_prev_inst_intrablock(inst).unwrap()).collect()
        }
    }
}

impl<'a> IntraproceduralBlockIterator<'a> for RealFunction<'a> {
    /// Get an iterator over the successor blocks of the provided block that are in this function.
    fn get_next_blocks_intraprocedural(&'a self, block: &'a RealBasicBlock<'a>) -> HashSet<&'a RealBasicBlock<'a>> {
        self.digraph.get_nexts(block).collect()
    }

    /// Get an iterator over the predecessor blocks of the provided block that are in this function.
    fn get_prev_blocks_intraprocedural(&'a self, block: &'a RealBasicBlock<'a>) -> HashSet<&'a RealBasicBlock<'a>> {
        self.digraph.get_prevs(block).collect()
    }
}

impl<'a> HasLabels for RealFunction<'a> {
    /// TODO naming may be confusing - only considers first basic block
    fn get_labels(&self) -> impl Iterator<Item = &String> {
        self.get_entry_block().get_labels()
    }

    /// TODO naming may be confusing - only considers first basic block
    fn has_label(&self, label: &impl ToString) -> bool {
        self.get_entry_block().has_label(label)
    }
}

impl<'a> Iterator for RealFunctionIterator<'a> {
    type Item = &'a RealBasicBlock<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.first.take().or_else(|| self.iterator.next())
    }
}

/// An iterator that returns basic blocks from a list
/// of instructions
struct RealInstListToBasicBlockConverter<'a> {
    inst_list: &'a RealInstList,
    current_basic_block: Option<RealBasicBlock<'a>>,
    list_iterator: Peekable<Enumerate<core::slice::Iter<'a, RealInst>>>,
}

impl<'a> Iterator for RealInstListToBasicBlockConverter<'a> {
    type Item = RealBasicBlock<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        // If a basic block exists and the next instruction is a branch target, then
        // return this current basic block.
        if let Some((idx, _)) = self.list_iterator.peek() {
            if self.inst_list.is_any_jump_target(*idx) {
                return self.current_basic_block.take();
            }
        }

        // Check if there's another instruction in the list
        while let Some((idx, inst)) = self.list_iterator.next() {
            // If there isn't a basic block yet, construct one with this instruction
            // Otherwise, add the instruction to the end.

            if let Some(bb) = self.current_basic_block.as_mut() {
                bb.push_inst(inst);
            } else {
                self.current_basic_block.insert(RealBasicBlock::new(inst));
            }

            // If the instruction is a non-function-call jump, stop the basic block
            if self.inst_list.is_any_non_function_call_jump(idx) {
                return self.current_basic_block.take();
            }
            continue;
        }
        // If there are no more instructions, return current basic block
        self.current_basic_block.take()
    }
}

impl RealInstList {
    pub(crate) fn iter_to_basic_blocks(&self) -> RealInstListToBasicBlockConverter {
        RealInstListToBasicBlockConverter {
            inst_list: &self,
            current_basic_block: None,
            list_iterator: self.list.iter().enumerate().peekable(),
        }
    }
}

impl<'a> HasIdentity for RealBasicBlock<'a> {
    fn id(&self) -> Uuid {
        self.get_first_instruction().id()
    }
}

impl<'a> DigraphNodes<RealBasicBlock<'a>> {
    fn find_basic_block_with_first_instruction(
        &self,
        target: &RealInst,
    ) -> Option<&RealBasicBlock> {
        self.iter().find(|x| x.get_first_instruction() == target)
    }
    fn find_basic_block_with_last_instruction(&self, target: &RealInst) -> Option<&RealBasicBlock> {
        self.iter().find(|x| x.get_last_instruction() == target)
    }
    fn get_instruction_edges(
        &'a self,
        inst_list: &'a RealInstList,
    ) -> impl Iterator<Item = (&RealInst, &RealInst)> {
        self.iter()
            // Get the last instruction
            .map(|x| x.get_last_instruction())
            // Get the three kinds of successors possible for an instruction
            .map(|x| {
                [
                    (x, inst_list.if_any_cond_jump_get_fallthrough(x.get_idx())),
                    (x, inst_list.if_any_cond_jump_get_target(x.get_idx())),
                    (
                        x,
                        inst_list.if_uncond_function_call_get_return_target(x.get_idx()),
                    ),
                ]
            })
            // Filter out all non-existing edges
            .flat_map(|x| x.into_iter())
            .filter_map(|(x, my)| Some((x, my?)))
    }
}

impl<'a> Digraph<RealBasicBlock<'a>> {
    fn update_edges(&mut self, inst_list: &'a RealInstList) {
        // Get the instruction -> instruction edges
        let inst_edges = self.nodes.get_instruction_edges(inst_list);
        // Map them to bb -> bb
        let bb_edges = inst_edges.filter_map(|(x, y)| {
            Some((
                self.nodes.find_basic_block_with_last_instruction(x)?,
                self.nodes.find_basic_block_with_first_instruction(y)?,
            ))
        });
        // Insert the edges
        self.edges.add_edges(bb_edges);
    }
}

/// CFG

struct RealCfg<'a> {
    functions: HashMap<Uuid, RealFunction<'a>>,
    inst_ids_to_func_ids: HashMap<Uuid, Uuid>,
    block_ids_to_func_ids: HashMap<Uuid, Uuid>,
    labels_to_inst_ids: HashMap<String, Uuid>,
    callee_func_id_to_caller_inst_ids: HashMap<Uuid, HashSet<Uuid>>,
    call_id_to_return_target_id: HashMap<Uuid, Uuid>,
    liveness: RefCell<HashMap<Uuid, LivenessInfo<<RealInst as InstructionLike>::Register>>>,
}

impl<'a> RealCfg<'a> {
    /// Get the return instruction that returns to `ret_target_inst`.
    fn get_return_inst_for_return_target_inst(&self, ret_target_inst: &RealInst) -> &RealInst {
        todo!()
    }

    /// Get the id of the return target instruction that the
    /// call with id `call_inst_id` will return to.
    fn get_return_target_id_of_call_by_id(&self, call_inst_id: &Uuid) -> &Uuid {
        self.call_id_to_return_target_id.get(call_inst_id)
            .expect("Call instruction id should be in call_id_to_return_target_id map")
    }

    /// For a return instruction `ret_inst`, get the ids of all
    /// possible instructions that it could return to.
    ///
    /// Equivalently, for every instruction that calls the function containing `ret_inst`,
    /// this function gets the ids of the instruction at the return address of those calls.
    fn get_return_target_ids_for_ret_inst(&self, ret_inst: &RealInst) -> HashSet<&Uuid> {
        self.get_calling_inst_ids_for_func(self.get_function_of_inst(ret_inst))
        .iter()
        .map(|call_inst_id| self.get_return_target_id_of_call_by_id(call_inst_id))
        .collect()
    }

    /// For a return instruction `ret_inst`, get all possible instructions that it could return to.
    ///
    /// Equivalently, for every instruction that calls the function containing `ret_inst`,
    /// this function gets the instruction at the return address of those calls.
    fn get_return_target_insts_for_ret_inst(&self, ret_inst: &RealInst) -> HashSet<&RealInst> {
        self.get_return_target_ids_for_ret_inst(ret_inst)
        .iter()
        .map(|inst_id| self.get_inst_by_id(&inst_id))
        .collect()
    }

    /// Get the id of the instruction that `call_inst` targets.
    fn get_target_id_for_call_inst(&self, call_inst: &RealInst) -> &Uuid {
        self.labels_to_inst_ids.get(
            &call_inst.get_target_label()
            .expect("Call instruction should have target label")
        ).expect("Target label should be in labels_to_inst_ids map")
    }

    /// Get the target of the instruction that `call_inst` targets.
    fn get_target_inst_for_call_inst(&self, call_inst: &RealInst) -> &RealInst {
        self.get_inst_by_id(self.get_target_id_for_call_inst(call_inst))
    }

    /// Get the ids of all instructions that call to the provided function entry instruction.
    fn get_calling_inst_ids_for_func_entry_inst(&self, func_entry_inst: &RealInst) -> &HashSet<Uuid> {
        self.get_calling_inst_ids_for_func(self.get_function_of_inst(func_entry_inst))
    }

    /// Get all instructions that call to the provided function entry instruction.
    fn get_calling_insts_for_func_entry_inst(&self, func_entry_inst: &RealInst) -> HashSet<&RealInst> {
        self.get_calling_insts_for_func(self.get_function_of_inst(func_entry_inst))
    }

    /// Get the ids of all instructions that call `func`.
    fn get_calling_inst_ids_for_func(&self, func: &RealFunction) -> &HashSet<Uuid> {
        self.get_calling_inst_ids_for_func_by_id(&func.id())
    }

    /// Get the ids of all instructions that call the function with id `func_id`.
    fn get_calling_inst_ids_for_func_by_id(&self, func_id: &Uuid) -> &HashSet<Uuid> {
        self.callee_func_id_to_caller_inst_ids.get(func_id)
            .expect("Function id should be in callee_func_id_to_caller_inst_ids map")
    }

    /// Get all instructions that call `func`.
    fn get_calling_insts_for_func(&self, func: &RealFunction) -> HashSet<&RealInst> {
        self.get_calling_inst_ids_for_func(func)
        .iter()
        .map(|inst_id| self.get_inst_by_id(inst_id))
        .collect()
    }

    /// Get all instructions that call the function with id `func_id`.
    fn get_calling_insts_for_func_by_id(&self, func_id: &Uuid) -> HashSet<&RealInst> {
        self.get_calling_inst_ids_for_func_by_id(func_id)
        .iter()
        .map(|inst_id| self.get_inst_by_id(inst_id))
        .collect()
    }
}

impl<'a> ContainsInstructions for RealCfg<'a> {
    /// Get the next instructions of `inst` that are in the same function as `inst`.
    fn get_inst_by_id(&self, inst_id: &Uuid) -> &RealInst {
        self.get_function_of_inst_by_id(inst_id).get_inst_by_id(inst_id)
    }
}

impl<'a> ContainsBasicBlocks for RealCfg<'a> {
    /// Given the id of a basic block in this CFG, get the basic block.
    fn get_basic_block_by_id(&self, block_id: &Uuid) -> &RealBasicBlock {
        self.get_function_of_basic_block_by_id(block_id).get_basic_block_by_id(block_id)
    }

    /// Given an instruction in this CFG, get the basic block that contains it.
    fn get_basic_block_of_inst(&self, inst: &RealInst) -> &RealBasicBlock {
        self.get_basic_block_of_inst_by_id(&inst.id())
    }

    /// Given a the id of an instruction in this CFG, get the basic block that contains it.
    fn get_basic_block_of_inst_by_id(&self, inst_id: &Uuid) -> &RealBasicBlock {
        self.get_function_of_inst_by_id(inst_id).get_basic_block_of_inst_by_id(inst_id)
    }
}

impl<'a> ContainsFunctions for RealCfg<'a> {
    /// Given the id of a function in this CFG, get the function.
    fn get_function_by_id(&self, function_id: &Uuid) -> &RealFunction {
        self.functions.get(&function_id).expect("Function id should be in functions map")
    }

    /// Given an instruction in this CFG, get the function that contains it.
    fn get_function_of_inst(&self, inst: &RealInst) -> &RealFunction {
        self.get_function_of_inst_by_id(&inst.id())
    }

    /// Given the id of an instruction in this CFG, get the function that contains it.
    fn get_function_of_inst_by_id(&self, inst_id: &Uuid) -> &RealFunction {
        self.get_function_by_id(
            self.inst_ids_to_func_ids.get(&inst_id)
            .expect("Instruction id should be in inst_ids_to_func_ids map")
        )
    }

    /// Given a basic block in this CFG, get the function that contains it.
    fn get_function_of_basic_block(&self, basic_block: &RealBasicBlock) -> &RealFunction {
        self.get_function_of_basic_block_by_id(&basic_block.id())
    }

    /// Given the id of a basic block in this CFG, get the function that contains it.
    fn get_function_of_basic_block_by_id(&self, basic_block_id: &Uuid) -> &RealFunction {
        self.get_function_by_id(
            self.block_ids_to_func_ids.get(&basic_block_id)
            .expect("Basic block id should be in block_ids_to_func_ids map")
        )
    }
}

impl<'a> IntraproceduralInstructionIterator for RealCfg<'a> {
    /// Get the next instructions of `inst` that are in the same function as `inst`.
    fn get_next_insts_intraprocedural(&self, inst: &RealInst) -> HashSet<&RealInst> {
        self.get_function_of_inst(inst).get_next_insts_intraprocedural(inst)
    }

    /// Get the previous instructions of `inst` that are in the same function as `inst`.
    fn get_prev_insts_intraprocedural(&self, inst: &RealInst) -> HashSet<&RealInst> {
        self.get_function_of_inst(inst).get_prev_insts_intraprocedural(inst)
    }
}

impl<'a> InterproceduralInstructionIterator for RealCfg<'a> {
    /// Given an instruction in this CFG, get all instructions that follow it
    /// including instructions outside of the function that `inst` is in.
    fn get_next_insts_interprocedural(&self, inst: &RealInst) -> HashSet<&RealInst> {
        // get_next_insts_intraprocedural will return None if, and only if, inst is not in function.
        // So, we can exit get_next_insts_interprocedural early in that case.
        let intraprocedural_nexts: HashSet<&RealInst> = self.get_next_insts_intraprocedural(inst);
        let mut all_nexts = intraprocedural_nexts;
        if inst.is_call() {
            let call_target = self.get_target_inst_for_call_inst(inst);
            all_nexts.insert(call_target);
        }
        if inst.is_ret() {
            let return_target_insts = self.get_return_target_insts_for_ret_inst(inst);
            all_nexts.extend(return_target_insts);
        }
        all_nexts
    }

    /// Given an instruction in this CFG, get all instructions that precede it
    /// including instructions outside of the function that `inst` is in.
    fn get_prev_insts_interprocedural(&self, inst: &RealInst) -> HashSet<&RealInst> {
        // get_prev_insts_intraprocedural will return None if, and only if, inst is not in function.
        // So, we can exit get_prev_insts_interprocedural early in that case.
        let intraprocedural_prevs: HashSet<&RealInst> = self.get_prev_insts_intraprocedural(inst);
        let mut all_prevs = intraprocedural_prevs;
        if inst.is_call_target() {
            let calling_insts = self.get_calling_insts_for_func_entry_inst(inst);
            all_prevs.extend(calling_insts);
        }
        if inst.is_return_target() {
            let return_inst = self.get_return_inst_for_return_target_inst(inst);
            all_prevs.insert(return_inst);
        }
        all_prevs
    }
}

impl<'a> CanGenerateLiveness<RealInst> for RealCfg<'a> {
    fn initialize_liveness(&self) {
        let mut liveness = self.liveness.borrow_mut();
        for inst_id in self.inst_ids_to_func_ids.keys() { // TODO better iterator
            liveness.insert(*inst_id, LivenessInfo::new());
        }
    }

    fn get_live_in(&self, item: &RealInst) -> <<RealInst as InstructionLike>::Register as LimitedElementSet>::ArrayType {
        self.liveness.borrow()
        .get(&item.id())
        .expect("Instruction id should be in liveness map")
        .get_live_in()
    }

    fn get_live_out(&self, item: &RealInst) -> <<RealInst as InstructionLike>::Register as LimitedElementSet>::ArrayType {
        self.liveness.borrow()
        .get(&item.id())
        .expect("Instruction id should be in liveness map")
        .get_live_out()
    }

    fn set_live_in_out(
        &self,
        item: &RealInst,
        live_in: <<RealInst as InstructionLike>::Register as LimitedElementSet>::ArrayType,
        live_out: <<RealInst as InstructionLike>::Register as LimitedElementSet>::ArrayType
    ) -> bool {
        self.liveness.borrow_mut()
        .get_mut(&item.id())
        .expect("Instruction id should be in liveness map")
        .set_live_in_out(live_in, live_out)
    }

    fn get_leaf_and_loop_nodes<'b>(&'b self) -> impl Iterator<Item = &'b RealInst> where RealInst: 'b {
        self.functions.values()
        .flat_map(|func| {
            // If function has an exit, return an iterator over all exit (leaf) instructions.
            // Otherwise, return an iterator over all of the instructions in the function
            if func.has_an_exit() {
                itertools::Either::Left(func.get_exit_insts())
            } else {
                itertools::Either::Right(func.intraprocedural_iter())
            }
        })
    }

    fn get_return_instructions_for_function_that_this_inst_targets<'b>(
        &'b self,
        inst: &RealInst,
    ) -> impl Iterator<Item = &'b RealInst> where RealInst: 'b {
        self.get_function_of_inst(
            self.get_target_inst_for_call_inst(inst)
        ).get_return_insts()
    }

    fn get_target_instruction_of_function_call_instruction(&self, inst: &RealInst) -> &RealInst {
        self.get_target_inst_for_call_inst(inst)
    }

    fn get_function_call_instructions_that_target_this<'b>(
        &'b self,
        inst: &RealInst,
    ) -> impl Iterator<Item = &'b RealInst> where RealInst: 'b {
        self.get_calling_insts_for_func_entry_inst(inst).into_iter()
    }

    fn is_first_instruction_in_function(&self, inst: &RealInst) -> bool {
        self.get_function_of_inst(inst).inst_is_entry(inst)
    }

    fn get_prevs<'b>(&'b self, inst: &RealInst) -> impl Iterator<Item = &'b RealInst> where RealInst: 'b {
        self.get_prev_insts_interprocedural(inst).into_iter()
    }

     fn get_nexts<'b>(&'b self, inst: &RealInst) -> impl Iterator<Item = &'b RealInst> where RealInst: 'b {
        self.get_next_insts_interprocedural(inst).into_iter()
    }
}

fn construct_new_cfg_from_inst_list(inst_list: &RealInstList) -> Cfg {
    // Step 1: Construct basic blocks into empty Digraph
    let mut basic_block_list: Digraph<_> = inst_list.iter_to_basic_blocks().collect();

    // Step 2: Assign directions to basic blocks
    basic_block_list.update_edges(inst_list);

    // Step 3: Group basic blocks into functions
    // let basic_block_next_map =
    //
    todo!()
}
