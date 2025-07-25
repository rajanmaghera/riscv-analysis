use std::async_iter::FromIter;
use super::CfgIterator;
use super::CfgNextsIterator;
use super::CfgNode;
use super::CfgPrevsIterator;
use super::CfgSourceIterator;
use super::Function;
use super::Segment;
use crate::analysis::HasGenKillInfo;
use crate::parser;
use crate::parser::{HasIdentity, InstructionProperties};
use crate::parser::LabelStringToken;
use crate::parser::ParserNode;
use crate::parser::{DirectiveType, Register, RegisterToken};
use crate::passes::CfgError;
use crate::passes::DiagnosticLocation;
use itertools::Itertools;
use std::collections::HashSet;
use std::collections::{HashMap, VecDeque};
use std::iter::{Enumerate, Peekable};
use std::ops::Deref;
use std::rc::Rc;
use std::vec;
use splitmut::SplitMut;
use uuid::Uuid;

struct RealInst;

struct RealInstList {
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
/// A basic block contains 1 or more instructions and always ends with a control-flow instruction
/// or the program falling off at the bottom (implicit halt/exception).
///
/// A basic block must be sequential in memory.
struct RealBasicBlock<'a> {
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
}
impl<'a> Iterator for RealBasicBlockIterator<'a> {
    type Item = &'a RealInst;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iterator.next()?.deref())
    }
}

/// FUNCTION

struct RealFunction<'a> {
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

/// CFG

struct RealCfg<'a> {
    rest_functions: HashSet<RealFunction<'a>>,
    first_function: RealFunction<'a>,
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
    fn iter_to_basic_blocks(&self) -> RealInstListToBasicBlockConverter {
        RealInstListToBasicBlockConverter {
            inst_list: &self,
            current_basic_block: None,
            list_iterator: self.list.iter().enumerate().peekable(),
        }
    }
}

struct DigraphElement<T: HasIdentity> {
    item: T,
}

impl<T: HasIdentity> HasIdentity for DigraphElement<T> {
    fn id(&self) -> Uuid {
        self.item.id()
    }}

struct Digraph<T: HasIdentity> {
    items: HashMap<Uuid, DigraphElement<T>>,
    nexts: HashMap<Uuid, HashSet<Uuid>>,
    prevs: HashMap<Uuid, HashSet<Uuid>>,
}

impl<T: HasIdentity> Digraph<T> {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            nexts: HashMap::new(),
            prevs: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, item: T) {
        self.items.insert(item.id(), DigraphElement { item });
    }

    pub fn add_edge(&mut self, from: &T, to: &T) {
        assert!(self.items.contains_key(&from.id()));
        assert!(self.items.contains_key(&to.id()));
        if let Some(nexts_set ) = self.nexts.get_mut(&from.id()) {
            nexts_set.insert(to.id());
        } else {
            self.nexts.insert(from.id(), HashSet::new());
        }

        if let Some(prevs_set ) = self.prevs.get_mut(&to.id()) {
            prevs_set.insert(from.id());
        } else {
            self.prevs.insert(to.id(), HashSet::new());
        }
    }

    pub fn remove_edge(&mut self, from: &T, to: &T) {
        assert!(self.items.contains_key(&from.id()));
        assert!(self.items.contains_key(&to.id()));
        if let Some(nexts_set) = self.nexts.get_mut(&from.id()) {
            nexts_set.remove(&to.id());
        }
        if let Some(prevs_set) = self.prevs.get_mut(&to.id()) {
            prevs_set.remove(&from.id());
        }
    }

    pub fn remove_node(&mut self, item: &T) {
        assert!(self.items.contains_key(&item.id()));
        if let Some(nexts_set) = self.nexts.get_mut(&item.id()) {
            // Find all previous nodes and remove this entry
            for next in nexts_set.iter() {
                if let Some(prevs_set) = self.prevs.get_mut(next) {
                    prevs_set.remove(&item.id());
                }
            }
        }
        if let Some(prevs_set) = self.prevs.get_mut(&item.id()) {
            for prev in prevs_set.iter() {
                if let Some(nexts_set) = self.nexts.get_mut(prev) {
                    nexts_set.remove(&item.id());
                }
            }
        }
    }

    pub fn get_nexts(&self, item: &T) -> impl Iterator<Item = &T> {
        self.nexts.get(&item.id()).unwrap().iter().map(|x| &self.items.get(x).unwrap().item)
    }

    pub fn get_prevs(&self, item: &T) -> impl Iterator<Item = &T> {
        self.prevs.get(&item.id()).unwrap().iter().map(|x| &self.items.get(x).unwrap().item)
    }

    pub fn get_nexts_mut(&mut self, item: &T) -> impl Iterator<Item = &mut T> {
        let ids = self.nexts.get(&item.id()).unwrap();
        self.items.iter_mut().filter(move |x| !ids.contains(x.0)).map(|x| &mut x.1.item)
    }

    pub fn get_prevs_mut(&mut self, item: &T) -> impl Iterator<Item = &mut T> {
        let ids = self.prevs.get(&item.id()).unwrap();
        self.items.iter_mut().filter(move |x| !ids.contains(x.0)).map(|x| &mut x.1.item)
    }
}

fn construct_new_cfg_from_inst_list(inst_list: &RealInstList) -> Cfg {
    // Step 1: Construct basic blocks
    let basic_block_list: Vec<_> = inst_list.iter_to_basic_blocks().collect();

    // Step 2: Assign directions to basic blocks
    let basic_block_next_map

    todo!()
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Cfg {
    nodes: Vec<Rc<CfgNode>>,
    pub label_node_map: HashMap<String, Rc<CfgNode>>,
    label_function_map: HashMap<LabelStringToken, Rc<Function>>,
}

impl Cfg {
    /// Get an iterator over the `Cfg` nodes.
    #[must_use]
    pub fn iter(&self) -> CfgIterator {
        CfgIterator::new(self)
    }

    /// Get an iterator over the `Cfg` nodes in source order.
    #[must_use]
    pub fn iter_source(&self) -> CfgSourceIterator {
        CfgSourceIterator::new(self)
    }

    /// Get an iterator over the `Cfg` nodes that are reachable using the
    /// nexts of `node`.
    #[must_use]
    pub fn iter_nexts<'a>(&'a self, node: &'a Rc<CfgNode>) -> CfgNextsIterator<'a> {
        CfgNextsIterator::new(self, node)
    }

    /// Get an iterator over the `Cfg` nodes that are reachable using the
    /// prevs of `node`.
    #[must_use]
    pub fn iter_prevs<'a>(&'a self, node: &'a Rc<CfgNode>) -> CfgPrevsIterator<'a> {
        CfgPrevsIterator::new(self, node)
    }

    /// Get the functions of the CFG.
    #[must_use]
    pub fn functions(&self) -> HashMap<LabelStringToken, Rc<Function>> {
        self.label_function_map.clone()
    }

    /// Insert a new function
    pub fn insert_function(&mut self, label: LabelStringToken, func: Rc<Function>) {
        self.label_function_map.insert(label, func);
    }

    /// Get the nodes of the CFG
    #[must_use]
    pub fn nodes(&self) -> &Vec<Rc<CfgNode>> {
        &self.nodes
    }
}

impl<'a> IntoIterator for &'a Cfg {
    type IntoIter = CfgIterator<'a>;
    type Item = &'a Rc<CfgNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

trait BaseCfgGen {
    fn call_names(&self) -> HashSet<LabelStringToken>;
    fn jump_names(&self) -> HashSet<LabelStringToken>;
    fn label_names(&self) -> HashSet<LabelStringToken>;
    fn load_names(&self) -> HashSet<LabelStringToken>;
}

impl BaseCfgGen for Vec<ParserNode> {
    fn call_names(&self) -> HashSet<LabelStringToken> {
        self.iter()
            .filter_map(parser::ParserNode::calls_to)
            .collect()
    }

    fn jump_names(&self) -> HashSet<LabelStringToken> {
        self.iter()
            .filter_map(parser::ParserNode::jumps_to)
            .collect()
    }

    fn load_names(&self) -> HashSet<LabelStringToken> {
        self.iter()
            .filter_map(parser::ParserNode::reads_address_of)
            .collect()
    }

    fn label_names(&self) -> HashSet<LabelStringToken> {
        self.iter()
            .filter_map(|x| match x {
                ParserNode::Label(s) => Some(s.name.clone()),
                _ => None,
            })
            .collect()
    }
}
impl Cfg {
    pub fn new(old_nodes: Vec<ParserNode>) -> Result<Cfg, Box<CfgError>> {
        Cfg::new_with_predefined_call_names(old_nodes, &None)
    }

    pub fn new_with_predefined_call_names(
        old_nodes: Vec<ParserNode>,
        predefined_call_names: &Option<HashSet<LabelStringToken>>,
    ) -> Result<Cfg, Box<CfgError>> {
        let mut labels = HashMap::new();
        let mut nodes = Vec::new();
        let mut current_labels = HashSet::new();
        let mut all_labels = HashSet::new();

        let label_names = old_nodes.label_names();
        let call_names = {
            let mut set = old_nodes.call_names();
            if let Some(new_set) = predefined_call_names.clone() {
                set.extend(new_set);
            }
            set
        };
        let jump_names = old_nodes.jump_names();
        let load_names = old_nodes.load_names();

        // Check if any call or jump names are not defined
        let undefined_labels = call_names
            .union(&jump_names)
            .cloned()
            .collect::<HashSet<_>>()
            .union(&load_names)
            .filter(|x| !label_names.contains(x))
            .cloned()
            .collect::<HashSet<LabelStringToken>>();

        if !undefined_labels.is_empty() {
            return Err(Box::new(CfgError::LabelsNotDefined(undefined_labels)));
        }

        // Code always begins in the text segment if it is not defined.
        let mut segment = Segment::Text;
        // PASS 1:
        // --------------------
        // Add nodes to graph

        for node in old_nodes {
            match node {
                ParserNode::Label(s) => {
                    current_labels.insert(s.name.clone());

                    // Check for duplicate labels
                    if !all_labels.insert(s.name.clone()) {
                        return Err(Box::new(CfgError::DuplicateLabel(s.name)));
                    }
                }
                ParserNode::Directive(x) if x.dir == DirectiveType::DataSection => {
                    segment = Segment::Data;
                }
                ParserNode::Directive(x) if x.dir == DirectiveType::TextSection => {
                    segment = Segment::Text;
                }
                // Ignore other types of directives
                ParserNode::Directive(_) => {}
                _ => {
                    // If any of the labels are a function call, add a function entry node
                    if current_labels
                        .clone()
                        .intersection(&call_names)
                        .next()
                        .is_some()
                    {
                        let is_interrupt = if let Some(ref p_call_names) = predefined_call_names {
                            // If any of the current_labels are in the predefined call names, then we need to add
                            // a boolean switch
                            current_labels
                                .clone()
                                .intersection(p_call_names)
                                .next()
                                .is_some()
                        } else {
                            false
                        };

                        let rc_node = Rc::new(CfgNode::new(
                            ParserNode::new_func_entry(
                                node.file(),
                                node.token().clone(),
                                is_interrupt,
                            ),
                            current_labels.clone(),
                            segment,
                        ));

                        // Add the node to the graph
                        nodes.push(Rc::clone(&rc_node));

                        // Add the node to the labels map
                        for label in current_labels.clone() {
                            labels.insert(label.to_string(), Rc::clone(&rc_node));
                        }

                        // Clear the current labels
                        current_labels.clear();

                        // Add the node to the graph
                        nodes.push(Rc::new(CfgNode::new(node, HashSet::new(), segment)));
                    } else {
                        let rc_node =
                            Rc::new(CfgNode::new(node.clone(), current_labels.clone(), segment));

                        // Add the node to the graph
                        nodes.push(Rc::clone(&rc_node));

                        // Add the node to the labels map
                        for label in current_labels.clone() {
                            labels.insert(label.to_string(), Rc::clone(&rc_node));
                        }

                        // Clear the current labels
                        current_labels.clear();
                    }
                }
            }
        }

        Ok(Cfg {
            nodes,
            label_function_map: HashMap::new(),
            label_node_map: labels,
        })
    }

    /// Perform a backwards search to find the first node that stores to the given register.
    ///
    /// From a given end point, like a return value, find the first node that stores to the given register.
    /// This function works by traversing the previous nodes until it finds a node that stores to the given register.
    /// This is used to correctly mark up the first store to a register that might
    /// have been incorrect.
    ///
    /// If we need to add an error to a register at its first use/store, we need to
    /// know their ranges. This function will take a register and return the ranges
    /// that need to be annotated. If it cannot find any, then it will return the original
    /// node's range.
    pub fn error_ranges_for_first_store(node: &Rc<CfgNode>, item: Register) -> Vec<RegisterToken> {
        let mut queue = VecDeque::new();
        let mut ranges = Vec::new();
        // push the previous nodes onto the queue
        queue.extend(node.prevs().clone());

        // keep track of visited nodes
        #[allow(clippy::mutable_key_type)]
        let mut visited = HashSet::new();
        visited.insert(Rc::clone(node));

        // visit each node in the queue
        // if the error is found, add error
        // if not, add the previous nodes to the queue
        while let Some(prev) = queue.pop_front() {
            if visited.contains(&prev) {
                continue;
            }
            visited.insert(Rc::clone(&prev));
            if let Some(reg) = prev.writes_to() {
                if *reg.get() == item {
                    ranges.push(reg);
                    continue;
                }
            }
            queue.extend(prev.prevs().clone().into_iter());
        }
        ranges
    }

    // TODO move to a more appropriate place
    // TODO make better, what even is this?
    pub fn error_ranges_for_first_usage(node: &Rc<CfgNode>, item: Register) -> Vec<RegisterToken> {
        let mut queue = VecDeque::new();
        let mut ranges = Vec::new();
        // push the next nodes onto the queue

        queue.extend(node.nexts().clone());

        // keep track of visited nodes
        #[allow(clippy::mutable_key_type)]
        let mut visited = HashSet::new();
        visited.insert(Rc::clone(node));

        // visit each node in the queue
        // if the error is found, add error
        // if not, add the next nodes to the queue
        while let Some(next) = queue.pop_front() {
            if visited.contains(&next) {
                continue;
            }
            visited.insert(Rc::clone(&next));
            if next.gen_reg().contains(&item) {
                // find the use
                let regs = next.reads_from();
                let mut it = None;
                for reg in regs {
                    if reg == item {
                        it = Some(reg);
                        break;
                    }
                }
                if let Some(reg) = it {
                    ranges.push(reg);
                    break;
                }
                break;
            }

            queue.extend(next.nexts().clone().into_iter());
        }
        ranges
    }
}
