use crate::parser;
use crate::parser::DirectiveType;
use crate::parser::InstructionProperties;
use crate::parser::LabelStringToken;
use crate::parser::ParserNode;
use crate::passes::CfgError;
use crate::passes::DiagnosticLocation;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use super::CfgIterator;
use super::CfgNextsIterator;
use super::CfgNode;
use super::CfgPrevsIterator;
use super::CfgSourceIterator;
use super::Function;
use super::Segment;

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
        Cfg::new_with_predefined_call_names(old_nodes, None)
    }

    pub fn new_with_predefined_call_names(
        old_nodes: Vec<ParserNode>,
        predefined_call_names: Option<HashSet<LabelStringToken>>,
    ) -> Result<Cfg, Box<CfgError>> {
        let mut labels = HashMap::new();
        let mut nodes = Vec::new();
        let mut current_labels = HashSet::new();
        let mut all_labels = HashSet::new();

        let label_names = old_nodes.label_names();
        let call_names = {
            let mut set = old_nodes.call_names();
            if let Some(new_set) = &predefined_call_names {
                set.extend(new_set.clone());
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
                        let is_interrupt = if let Some(p_call_names) = &predefined_call_names {
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
}
