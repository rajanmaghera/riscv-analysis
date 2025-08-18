use crate::cfg::Cfg;
use crate::new_impl::contains_basic_blocks::ContainsBasicBlocks;
use crate::new_impl::contains_functions::ContainsFunctions;
use crate::new_impl::digraph::{Digraph, DigraphNodes};
use crate::new_impl::has_labels::HasLabels;
use crate::parser::HasIdentity;
use std::collections::{HashMap, HashSet};
use std::iter::{Enumerate, Peekable};
use std::ops::Deref;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealInst {
    labels: HashSet<String>,
    id: Uuid,
}


impl RealInst {
    // Remove this, this is not the place it should be
    fn get_idx(&self) -> usize {
        todo!()
    }
}

impl HasLabels for RealInst {
    fn get_labels(&self) -> &HashSet<String> {
        &self.labels
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
    fn get_label_names(&self, idx: usize) -> &HashSet<String> {
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
        Self {
            insts: vec![first_inst],
        }
    }

    /// Add a new instruction to the end of the basic block.
    ///
    /// This function should NOT be published.
    fn push_inst(&mut self, inst: &'a RealInst) {
        self.insts.push(inst);
    }

    /// Get the last instruction.
    ///
    /// This instruction will always exist, but it might not always be a
    /// control-flow instruction (ie. final instruction in the program that
    /// falls off the end).
    fn get_last_instruction(&self) -> &RealInst {
        self.insts.last().unwrap()
    }

    fn get_first_instruction(&self) -> &RealInst {
        self.insts.first().unwrap()
    }
}

impl<'a> HasLabels for RealBasicBlock<'a> {
    /// TODO naming may be confusing - only considers first instruction
    fn get_labels(&self) -> &HashSet<String> {
        &self.get_first_instruction().get_labels()
    }

    /// TODO naming may be confusing - only considers first instruction
    fn has_label(&self, label: &impl ToString) -> bool {
        self.get_first_instruction().has_label(label)
    }
}

impl<'a> Iterator for RealBasicBlockIterator<'a> {
    type Item = &'a RealInst;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iterator.next()?.deref())
    }
}

/// FUNCTION

pub struct RealFunction<'a> {
    digraph: Digraph<RealBasicBlock<'a>>,
    inst_ids_to_block_ids: HashMap<Uuid, Uuid>,
    entry_block_id: Uuid,
    exit_block_ids: HashSet<Uuid>,
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

    // TODO add a block, connecting it to successors and predecessors,
    // and adding its id to exit_block_ids if it may exit the function
    pub fn add_block(&mut self, block: RealBasicBlock<'a>) {
        self.digraph.add_node(block);
        todo!(); // add edges as appropriate
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
        self.digraph.get_by_id(&self.entry_block_id).expect("Entry block should exist in function digraph")
    }

    /// Get the exit blocks of the function.
    pub fn get_exit_blocks(&self) -> impl Iterator<Item = &RealBasicBlock> {
        self.exit_block_ids.iter().map(|b| self.digraph.get_by_id(b).expect("Exit block should exist in function digraph"))
    }

    /// Check if the block is the entry block of this function.
    pub fn block_is_entry(&self, block: &RealBasicBlock) -> bool {
        self.entry_block_id == block.id()
    }

    /// Check if the block is an exit block of this function.
    pub fn block_is_an_exit(&self, block: &RealBasicBlock) -> bool {
        self.exit_block_ids.contains(&block.id())
    }

    /// Get an iterator over the successor blocks of the provided block that are in this function.
    pub fn get_block_nexts_in_function(&self, block: &RealBasicBlock<'a>) -> impl Iterator<Item = &RealBasicBlock<'a>> {
        self.digraph.get_nexts(block)
    }

    /// Get an iterator over the predecessor blocks of the provided block that are in this function.
    pub fn get_block_prevs_in_function(&self, block: &RealBasicBlock<'a>) -> impl Iterator<Item = &RealBasicBlock<'a>> {
        self.digraph.get_prevs(block)
    }

    /// Check if this function contains the given block.
    pub fn contains(&self, block: &RealBasicBlock) -> bool {
        self.digraph.contains(block)
    }
}

impl<'a> HasIdentity for RealFunction<'a> {
    /// Get the id of the function.
    fn id(&self) -> Uuid {
        self.id
    }
}

impl<'a> ContainsBasicBlocks for RealFunction<'a> {
    /// Given the id of a basic block in this function, get the basic block.
    ///
    /// Returns `None` if there is no block with the specified id in this function.
    fn get_basic_block_by_id(&self, block_id: &Uuid) -> Option<&RealBasicBlock> {
        self.digraph.get_by_id(block_id)
    }

    /// Given an instruction in this function, get the basic block that contains it.
    ///
    /// Returns `None` if the instruction is not in this function.
    fn get_basic_block_of_inst(&self, inst: &RealInst) -> Option<&RealBasicBlock> {
        self.get_basic_block_by_id(self.inst_ids_to_block_ids.get(&inst.id())?)
    }
}

impl<'a> HasLabels for RealFunction<'a> {
    /// TODO naming may be confusing - only considers first basic block
    fn get_labels(&self) -> &HashSet<String> {
        &self.get_entry_block().get_labels()
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
}

impl<'a> ContainsBasicBlocks for RealCfg<'a> {
    /// Given the id of a basic block in this CFG, get the basic block.
    ///
    /// Returns `None` if there is no block with the specified id in this CFG.
    fn get_basic_block_by_id(&self, block_id: &Uuid) -> Option<&RealBasicBlock> {
        self.get_function_by_id(self.block_ids_to_func_ids.get(block_id)?)?.get_basic_block_by_id(block_id)
    }

    /// Given an instruction in this CFG, get the basic block that contains it.
    ///
    /// Returns `None` if the instruction is not in this CFG.
    fn get_basic_block_of_inst(&self, inst: &RealInst) -> Option<&RealBasicBlock> {
        self.get_function_of_inst(inst)?.get_basic_block_of_inst(inst)
    }
}

impl<'a> ContainsFunctions for RealCfg<'a> {
    /// Given the id of a function in this CFG, get the function.
    ///
    /// Returns `None` if there is no function with the specified id in this CFG.
    fn get_function_by_id(&self, function_id: &Uuid) -> Option<&RealFunction> {
        self.functions.get(&function_id)
    }

    /// Given an instruction in this CFG, get the function that contains it.
    ///
    /// Returns `None` if the specified instruction is not in this CFG.
    fn get_function_of_inst(&self, inst: &RealInst) -> Option<&RealFunction> {
        self.get_function_by_id(self.inst_ids_to_func_ids.get(&inst.id())?)
    }

    /// Given a basic block in this CFG, get the function that contains it.
    ///
    /// Returns `None` if the specified basic block is not in this CFG.
    fn get_function_of_basic_block(&self, basic_block: &RealBasicBlock) -> Option<&RealFunction> {
        self.get_function_by_id(self.block_ids_to_func_ids.get(&basic_block.id())?)
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
