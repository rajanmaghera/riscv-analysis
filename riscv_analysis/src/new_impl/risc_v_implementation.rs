use crate::cfg::Cfg;
use crate::new_impl::digraph::{Digraph, DigraphNodes};
use crate::parser::HasIdentity;
use std::collections::HashSet;
use std::iter::{Enumerate, Peekable};
use std::ops::Deref;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealInst {}

impl RealInst {
    // Remove this, this is not the place it should be
    fn get_idx(&self) -> usize {
        todo!()
    }
}

impl HasIdentity for RealInst {
    fn id(&self) -> Uuid {
        Uuid::default()
    }
}

pub struct RealInstList {
    list: Vec<RealInst>,
}

impl RealInstList {
    fn get_label_names(&self, idx: usize) -> impl Iterator<Item = &str> {
        [].into_iter()
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
    labels: Vec<String>,
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
            labels: vec![],
        }
    }

    /// Add a new instruction to the end of the basic block.
    ///
    /// This function should NOT be published.
    fn push_inst(&mut self, inst: &'a RealInst) {
        self.insts.push(inst);
    }

    /// Add a label to the basic block.
    ///
    /// This function should NOT be published.
    fn add_label(&mut self, label: &impl ToString) {
        self.labels.push(label.to_string());
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

impl<'a> Iterator for RealBasicBlockIterator<'a> {
    type Item = &'a RealInst;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iterator.next()?.deref())
    }
}

/// FUNCTION

pub struct RealFunction<'a> {
    first_basic_block: RealBasicBlock<'a>,
    rest_basic_blocks: HashSet<RealBasicBlock<'a>>,
}

struct RealFunctionIterator<'a> {
    first: Option<&'a RealBasicBlock<'a>>,
    iterator: std::collections::hash_set::Iter<'a, RealBasicBlock<'a>>,
}

impl<'a> RealFunction<'a> {
    fn iter(&'a self) -> impl Iterator<Item = &'a RealBasicBlock<'a>> {
        RealFunctionIterator {
            first: Some(&self.first_basic_block),
            iterator: self.rest_basic_blocks.iter(),
        }
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
    rest_functions: HashSet<RealFunction<'a>>,
    first_function: RealFunction<'a>,
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
