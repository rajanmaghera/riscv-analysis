use super::CfgIterator;
use super::CfgNextsIterator;
use super::CfgNode;
use super::CfgPrevsIterator;
use super::CfgSourceIterator;
use super::Function;
use crate::analysis::HasGenKillInfo;
use crate::parser;
use crate::parser::LabelStringToken;
use crate::parser::ParserNode;
use crate::parser::{Register, RegisterToken};
use crate::parser::{InstructionProperties, RVParserOutput};
use crate::passes::CfgError;
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
    pub fn iter_nexts(&self, node: Rc<CfgNode>) -> CfgNextsIterator {
        CfgNextsIterator::new(node)
    }

    /// Get an iterator over the `Cfg` nodes that are reachable using the
    /// prevs of `node`.
    #[must_use]
    pub fn iter_prevs(&self, node: Rc<CfgNode>) -> CfgPrevsIterator {
        CfgPrevsIterator::new(node)
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
    type Item = Rc<CfgNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

trait BaseCfgGen {
    fn call_names(&self) -> HashSet<LabelStringToken>;
    fn jump_names(&self) -> HashSet<LabelStringToken>;
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
}
impl Cfg {
    pub fn new(parser_output: RVParserOutput) -> Result<Cfg, Box<CfgError>> {
        Cfg::new_with_predefined_call_names(parser_output, None)
    }

    pub fn new_with_predefined_call_names(
        parser_output: RVParserOutput,
        predefined_call_names: Option<&HashSet<LabelStringToken>>,
    ) -> Result<Cfg, Box<CfgError>> {
        let mut labels = HashMap::new();
        let mut nodes = Vec::new();

        let defined_labels = parser_output.all_defined_labels;
        let call_names = {
            let mut set = parser_output.nodes.call_names();
            if let Some(new_set) = predefined_call_names {
                set.extend(new_set.clone());
            }
            set
        };
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

        // For now, the first instruction is the program entry
        let mut is_program_entry = true;

        for node in parser_output.nodes {
            // Check if this node's label has already been defined
            for label in node.label_names() {
                if labels.keys().any(|x| x == label.as_str()) {
                    return Err(Box::new(CfgError::DuplicateLabel(label.clone())));
                }
            }

            // If any of the labels are a function call, add a function entry node
            let is_function_call = node
                .label_names()
                .cloned()
                .collect::<HashSet<_>>()
                .intersection(&call_names)
                .next()
                .is_some();

            // Get a copy of the node's labels
            let current_labels = node.label_names().cloned().collect::<HashSet<_>>();

            // Create the new node
            let new_node = Rc::new(CfgNode::new(node, is_function_call, is_program_entry));

            // Add the node to the labels map
            for label in current_labels {
                labels.insert(label.to_string(), Rc::clone(&new_node));
            }

            nodes.push(new_node);
            is_program_entry = false;
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
