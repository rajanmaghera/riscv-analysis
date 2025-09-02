use super::CfgSourceIterator;
use super::Function;
use super::{CfgBreadthFirstIterator, ExternalFunction};
use super::{CfgNode, RegisterSet};
use crate::analysis::HasGenKillInfo;
use crate::parser;
use crate::parser::{HasIdentity, InstructionProperties, JumpTarget};
use crate::parser::{LabelString, Position, RVInstructionNode, RVToken, Range, TokenType};
use crate::parser::{LabelStringToken, ProgramEntryType, RVParserOutput};
use crate::parser::{RVRegister, RegisterToken};
use crate::passes::CfgError;
use itertools::Itertools;
use std::collections::HashSet;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Cfg {
    nodes: Vec<Rc<CfgNode>>,
    nexts: HashMap<Uuid, HashSet<Rc<CfgNode>>>,
    prevs: HashMap<Uuid, HashSet<Rc<CfgNode>>>,
    provisional_nexts: HashMap<Uuid, HashSet<Rc<CfgNode>>>,
    pub label_node_map: HashMap<String, Rc<CfgNode>>,
    functions: HashSet<Rc<Function>>,
    label_function_map: HashMap<LabelStringToken, Rc<Function>>,
    label_external_function_map: HashMap<LabelStringToken, ExternalFunction>,
    function_to_call_sites_map: HashMap<Rc<Function>, HashSet<Rc<CfgNode>>>,
}

impl Cfg {
    /// Get an iterator over the `Cfg` nodes in source order.
    #[must_use]
    pub fn iter_source(&self) -> CfgSourceIterator<'_> {
        CfgSourceIterator::new(self)
    }

    /// Get an iterator over the `Cfg` nodes that are reachable using the
    /// nexts of `node`.
    #[must_use]
    pub fn iter_breadth_first<'a>(&'a self, node: &'a Rc<CfgNode>) -> CfgBreadthFirstIterator<'a> {
        CfgBreadthFirstIterator::new(self, node)
    }

    /// Get a function by its name
    #[must_use]
    pub fn get_function(&self, name: &LabelStringToken) -> Option<&Rc<Function>> {
        self.label_function_map.get(name)
    }

    /// Get every function in the program
    pub fn get_all_functions(&self) -> impl Iterator<Item = &Rc<Function>> {
        self.functions.iter()
    }

    /// Insert a new function
    pub fn insert_function(&mut self, label: LabelStringToken, func: Rc<Function>) {
        self.functions.insert(Rc::clone(&func));
        self.function_to_call_sites_map
            .insert(Rc::clone(&func), HashSet::new());
        self.label_function_map.insert(label, func);
    }

    /// Get the external functions of the CFG.
    #[must_use]
    pub fn get_external_function(&self, name: &LabelStringToken) -> Option<&ExternalFunction> {
        self.label_external_function_map.get(name)
    }

    /// Get the nodes of the CFG
    #[must_use]
    pub fn nodes(&self) -> &Vec<Rc<CfgNode>> {
        &self.nodes
    }
}

trait BaseCfgGen {
    fn call_names(&self) -> HashSet<LabelStringToken>;
    fn jump_names(&self) -> HashSet<LabelStringToken>;
    fn load_names(&self) -> HashSet<LabelStringToken>;
}

fn eliminate_register_jump_target(x: JumpTarget) -> Option<LabelStringToken> {
    match x {
        JumpTarget::Label(label) => Some(label),
        JumpTarget::Register(_) => None,
    }
}

impl BaseCfgGen for Vec<RVInstructionNode> {
    fn call_names(&self) -> HashSet<LabelStringToken> {
        self.iter()
            .filter_map(parser::RVInstructionNode::calls_to)
            .filter_map(eliminate_register_jump_target)
            .collect()
    }

    fn jump_names(&self) -> HashSet<LabelStringToken> {
        self.iter()
            .filter_map(parser::RVInstructionNode::jumps_to)
            .filter_map(eliminate_register_jump_target)
            .collect()
    }

    fn load_names(&self) -> HashSet<LabelStringToken> {
        self.iter()
            .filter_map(parser::RVInstructionNode::reads_address_of)
            .collect()
    }
}
impl Cfg {
    pub fn new(
        parser_output: RVParserOutput,
        predefined_call_names: Option<&HashSet<LabelStringToken>>,
        external_functions: &Option<HashSet<(LabelStringToken, RegisterSet, RegisterSet)>>,
        program_entry: &ProgramEntryType,
    ) -> Result<Cfg, Box<CfgError>> {
        let mut labels = HashMap::new();
        let mut nodes = Vec::new();

        // Get list of all defined labels (labels that can safely be jump targets)
        let mut defined_labels = parser_output.all_defined_labels;
        if let Some(new_set) = &external_functions {
            defined_labels.extend(new_set.iter().map(|(name, _, _)| name.clone()));
        }

        // Get list of all labels that are called to as a functions
        let call_names = {
            let mut set = parser_output.nodes.call_names();
            // If they exist, treat these labels as the start of a function
            if let Some(new_set) = predefined_call_names {
                set.extend(new_set.clone());
            }
            if let Some(new_set) = external_functions {
                set.extend(new_set.iter().map(|(name, _, _)| name.clone()));
            }
            set
        };
        // External functions can contain functions that are also defined in the code
        // When that happens, don't treat it as an external function
        let jump_names = parser_output.nodes.jump_names();
        let load_names = parser_output.nodes.load_names();

        // Check if any call or jump names are not defined
        let all_label_targets: HashSet<_> = call_names
            .iter()
            .chain(jump_names.iter())
            .chain(load_names.iter())
            .cloned()
            .collect();
        let undefined_labels: HashSet<_> = all_label_targets
            .difference(&defined_labels)
            .cloned()
            .collect();

        if !undefined_labels.is_empty() {
            return Err(Box::new(CfgError::LabelsNotDefined(undefined_labels)));
        }

        // If the program entry is a label, return an error if the label does not exist
        if let ProgramEntryType::LookForLabel(name) = &program_entry {
            if !defined_labels.contains(name) {
                return Err(Box::new(CfgError::LabelsNotDefined(HashSet::from([
                    name.clone()
                ]))));
            }
        }

        for (idx, node) in parser_output.nodes.into_iter().enumerate() {
            let is_program_entry = match &program_entry {
                ProgramEntryType::LookForLabel(l) => node.label_names().contains(&l),
                ProgramEntryType::FirstInstruction => idx == 0,
                ProgramEntryType::None => false,
            };

            // Check if this node's label has already been defined
            for label in node.label_names() {
                if labels.keys().any(|x| x == label.as_str()) {
                    return Err(Box::new(CfgError::DuplicateLabel(label.clone())));
                }
            }

            // If any of the labels are a function call, add a function entry node
            let is_function_entry = node.label_names().any(|x| call_names.contains(&x));

            // Create the new node
            let new_node = Rc::new(CfgNode::new(node, is_function_entry, is_program_entry));

            // Add the node to the labels map
            for label in new_node.node().label_names() {
                labels.insert(label.to_string(), Rc::clone(&new_node));
            }

            nodes.push(new_node);
        }

        let nexts = nodes.iter().map(|x| (x.id(), HashSet::new())).collect();
        let prevs = nodes.iter().map(|x| (x.id(), HashSet::new())).collect();

        let label_external_function_map: HashMap<_, _> = external_functions
            .iter()
            .flat_map(|x| x.iter())
            // Remove external functions where the label is defined in the input code
            .filter(|x| !labels.keys().any(|y| x.0.get().as_str() == y.as_str()))
            .map(|(name, args, rets)| {
                (
                    name.clone(),
                    ExternalFunction::new([name.clone()].into_iter().collect(), *args, *rets),
                )
            })
            .collect();

        Ok(Cfg {
            nodes,
            nexts,
            prevs,
            provisional_nexts: HashMap::new(),
            label_function_map: HashMap::new(),
            functions: HashSet::new(),
            label_node_map: labels,
            label_external_function_map,
            function_to_call_sites_map: HashMap::new(),
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
    pub fn error_ranges_for_first_store(
        &self,
        node: &Rc<CfgNode>,
        item: RVRegister,
    ) -> Vec<RegisterToken> {
        let mut queue = VecDeque::new();
        let mut ranges = Vec::new();
        // push the previous nodes onto the queue
        queue.extend(self.get_prevs(node.as_ref()).cloned());

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
            for reg in prev.writes_to() {
                if *reg.get() == item {
                    ranges.push(reg);
                }
            }
            queue.extend(self.get_prevs(prev.as_ref()).cloned());
        }
        ranges
    }

    // TODO move to a more appropriate place
    // TODO make better, what even is this?
    pub fn error_ranges_for_first_usage(
        &self,
        node: &Rc<CfgNode>,
        item: RVRegister,
    ) -> Vec<RegisterToken> {
        let mut queue = VecDeque::new();
        let mut ranges = Vec::new();
        // push the next nodes onto the queue

        queue.extend(self.get_nexts(node.as_ref()).cloned());

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

            queue.extend(self.get_nexts(next.as_ref()).cloned());
        }
        ranges
    }

    /// Get the successors of a given node.
    ///
    /// # Panics
    ///
    /// Panics if the node does not exist on this CFG.
    pub fn get_nexts<'a>(
        &'a self,
        node: &'a CfgNode,
    ) -> impl ExactSizeIterator<Item = &'a Rc<CfgNode>> + 'a {
        self.nexts.get(&node.id()).unwrap().iter()
    }

    /// Get the predecessors of a given node.
    ///
    /// # Panics
    ///
    /// Panics if the node does not exist on this CFG.
    pub fn get_prevs<'a>(
        &'a self,
        node: &'a CfgNode,
    ) -> impl ExactSizeIterator<Item = &'a Rc<CfgNode>> + 'a {
        self.prevs.get(&node.id()).unwrap().iter()
    }

    /// Insert an edge from one node to another.
    ///
    /// # Panics
    ///
    /// Panics if the nodes `from` and `to` do not exist on this CFG.
    pub fn insert_edge(&mut self, from: &Rc<CfgNode>, to: &Rc<CfgNode>) {
        self.nexts
            .get_mut(&from.id())
            .unwrap()
            .insert(Rc::clone(to));
        self.prevs
            .get_mut(&to.id())
            .unwrap()
            .insert(Rc::clone(from));
    }

    /// Insert a provisional edge from one node to another.
    ///
    /// These edges are not inserted initially, but later can be promoted to
    /// a real edge
    pub fn insert_provisional_edge(&mut self, from: &Rc<CfgNode>, to: &Rc<CfgNode>) {
        let entry = self.provisional_nexts.entry(from.id()).or_default();
        entry.insert(Rc::clone(to));
    }

    /// Promote a provisional edge to a real edge, if they exist.
    ///
    /// This function returns if a modification was made
    #[must_use]
    pub fn promote_provisional_nexts_to_real(&mut self, from: &Rc<CfgNode>) -> bool {
        if let Some(tos) = self.provisional_nexts.remove(&from.id()) {
            for to in tos {
                self.insert_edge(from, &to);
            }
            true
        } else {
            false
        }
    }

    /// Remove an edge from one node to another.
    ///
    /// # Panics
    ///
    /// Panics if the nodes `from` and `to` do not exist on this CFG.
    pub fn remove_edge(&mut self, from: &CfgNode, to: &CfgNode) -> bool {
        let res_1 = self.nexts.get_mut(&from.id()).unwrap().remove(to);
        let res_2 = self.prevs.get_mut(&to.id()).unwrap().remove(from);
        res_1 | res_2
    }

    /// Get the nodes that call this function
    ///
    /// # Panics
    ///
    /// Panics if the function does not exist on this CFG.
    pub fn get_call_sites(&self, function: &Rc<Function>) -> impl Iterator<Item = &Rc<CfgNode>> {
        self.function_to_call_sites_map
            .get(function)
            .unwrap()
            .iter()
    }

    /// Insert a call site for a function.
    ///
    /// # Panics
    ///
    /// Panics if the function does not exist on this cfg
    pub fn insert_call_site(&mut self, function: &Rc<Function>, node: Rc<CfgNode>) {
        self.function_to_call_sites_map
            .get_mut(function)
            .unwrap()
            .insert(node);
    }
}
