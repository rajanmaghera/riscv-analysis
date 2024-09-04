use crate::parser;
use crate::parser::DirectiveType;
use crate::parser::LabelString;
use crate::parser::ParserNode;
use crate::parser::With;
use crate::passes::CfgError;
use crate::passes::DiagnosticLocation;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use super::CfgNode;
use super::Function;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Cfg {
    pub nodes: Vec<Rc<CfgNode>>,
    pub label_node_map: HashMap<String, Rc<CfgNode>>,
    pub label_function_map: HashMap<With<LabelString>, Rc<Function>>,
}

impl IntoIterator for &Cfg {
    type Item = Rc<CfgNode>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    // nested iterator for blocks
    fn into_iter(self) -> Self::IntoIter {
        self.nodes.clone().into_iter()
    }
}

impl Cfg {
    /// Get an iterator over the `Cfg` nodes.
    #[must_use]
    pub fn iter(&self) -> std::vec::IntoIter<Rc<CfgNode>> {
        self.into_iter()
    }
}

trait BaseCfgGen {
    fn call_names(&self) -> HashSet<With<LabelString>>;
    fn jump_names(&self) -> HashSet<With<LabelString>>;
    fn label_names(&self) -> HashSet<With<LabelString>>;
    fn load_names(&self) -> HashSet<With<LabelString>>;
}

impl BaseCfgGen for Vec<ParserNode> {
    fn call_names(&self) -> HashSet<With<LabelString>> {
        self.iter()
            .filter_map(parser::ParserNode::calls_to)
            .collect()
    }

    fn jump_names(&self) -> HashSet<With<LabelString>> {
        self.iter()
            .filter_map(parser::ParserNode::jumps_to)
            .collect()
    }

    fn load_names(&self) -> HashSet<With<LabelString>> {
        self.iter()
            .filter_map(parser::ParserNode::reads_address_of)
            .collect()
    }

    fn label_names(&self) -> HashSet<With<LabelString>> {
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
        let mut labels = HashMap::new();
        let mut nodes = Vec::new();
        let mut current_labels = HashSet::new();
        let mut all_labels = HashSet::new();

        let label_names = old_nodes.label_names();
        let call_names = old_nodes.call_names();
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
            .collect::<HashSet<With<LabelString>>>();

        if !undefined_labels.is_empty() {
            return Err(Box::new(CfgError::LabelsNotDefined(undefined_labels)));
        }

        let mut data_section = false;

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
                    data_section = true;
                }
                ParserNode::Directive(x) if x.dir == DirectiveType::TextSection => {
                    data_section = false;
                }
                _ => {
                    // If any of the labels are a function call, add a function entry node
                    if current_labels
                        .clone()
                        .intersection(&call_names)
                        .next()
                        .is_some()
                    {
                        let rc_node = Rc::new(CfgNode::new(
                            ParserNode::new_func_entry(node.file(), node.token()),
                            current_labels.clone(),
                            data_section,
                        ));

                        // Add the node to the graph
                        nodes.push(Rc::clone(&rc_node));

                        // Add the node to the labels map
                        for label in current_labels.clone() {
                            labels.insert(label.data.0.clone(), Rc::clone(&rc_node));
                        }

                        // Clear the current labels
                        current_labels.clear();

                        // Add the node to the graph
                        nodes.push(Rc::new(CfgNode::new(node, HashSet::new(), data_section)));
                    } else {
                        let rc_node = Rc::new(CfgNode::new(
                            node.clone(),
                            current_labels.clone(),
                            data_section,
                        ));

                        // Add the node to the graph
                        nodes.push(Rc::clone(&rc_node));

                        // Add the node to the labels map
                        for label in current_labels.clone() {
                            labels.insert(label.data.0.clone(), Rc::clone(&rc_node));
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
