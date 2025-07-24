use super::CfgIterator;
use super::CfgNextsIterator;
use super::CfgNode;
use super::CfgPrevsIterator;
use super::CfgSourceIterator;
use super::Function;
use super::Segment;
use crate::analysis::HasGenKillInfo;
use crate::parser;
use crate::parser::InstructionProperties;
use crate::parser::LabelStringToken;
use crate::parser::ParserNode;
use crate::parser::{DirectiveType, Register, RegisterToken};
use crate::passes::CfgError;
use crate::passes::DiagnosticLocation;
use std::collections::HashSet;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

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
