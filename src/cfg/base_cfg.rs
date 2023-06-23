use crate::parser::LabelString;
use crate::parser::Node;
use crate::parser::Parser;
use crate::parser::With;
use crate::passes::CFGError;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;
use std::str::FromStr;

use super::CFGNode;
use super::Function;

#[derive(Debug, PartialEq, Eq)]
pub struct BaseCFG {
    pub nodes: Vec<Rc<CFGNode>>,
    pub label_node_map: HashMap<String, Rc<CFGNode>>,
    pub label_function_map: HashMap<With<LabelString>, Rc<Function>>,
}

impl FromStr for BaseCFG {
    type Err = CFGError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parser = Parser::new(s);
        let ast = parser.collect::<Vec<Node>>();
        BaseCFG::new(ast)
    }
}

impl IntoIterator for &BaseCFG {
    type Item = Rc<CFGNode>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    // nested iterator for blocks
    fn into_iter(self) -> Self::IntoIter {
        self.nodes.clone().into_iter()
    }
}

trait BaseCFGGen {
    fn call_names(&self) -> HashSet<With<LabelString>>;
    fn jump_names(&self) -> HashSet<With<LabelString>>;
    fn label_names(&self) -> HashSet<With<LabelString>>;
}

impl BaseCFGGen for Vec<Node> {
    fn call_names(&self) -> HashSet<With<LabelString>> {
        self.iter().filter_map(|x| x.calls_to()).collect()
    }

    fn jump_names(&self) -> HashSet<With<LabelString>> {
        self.iter().filter_map(|x| x.jumps_to()).collect()
    }

    fn label_names(&self) -> HashSet<With<LabelString>> {
        self.iter()
            .filter_map(|x| match x {
                Node::Label(s) => Some(s.name.clone()),
                _ => None,
            })
            .collect()
    }
}
impl BaseCFG {
    pub fn new(old_nodes: Vec<Node>) -> Result<BaseCFG, CFGError> {
        let mut labels = HashMap::new();
        let mut nodes = Vec::new();
        let mut current_labels = HashSet::new();
        let mut all_labels = HashSet::new();

        let label_names = old_nodes.label_names();
        let call_names = old_nodes.call_names();
        let jump_names = old_nodes.jump_names();

        // Check if any call or jump names are not defined
        let undefined_labels = call_names
            .union(&jump_names)
            .filter(|x| !label_names.contains(x))
            .cloned()
            .collect::<HashSet<With<LabelString>>>();
        if !undefined_labels.is_empty() {
            return Err(CFGError::LabelsNotDefined(undefined_labels));
        }

        // Add program entry node
        nodes.push(Rc::new(CFGNode::new(
            Node::new_program_entry(),
            HashSet::new(),
        )));

        // PASS 1:
        // --------------------
        // Add nodes to graph

        for node in old_nodes {
            match node {
                Node::Label(s) => {
                    current_labels.insert(s.name.clone());

                    // Check for duplicate labels
                    if all_labels.insert(s.name.clone()) == false {
                        return Err(CFGError::DuplicateLabel(s.name.clone()));
                    }
                }
                _ => {
                    // If any of the labels are a function call, add a function entry node
                    if current_labels
                        .clone()
                        .intersection(&call_names)
                        .next()
                        .is_some()
                    {
                        let rc_node =
                            Rc::new(CFGNode::new(Node::new_func_entry(), current_labels.clone()));

                        // Add the node to the graph
                        nodes.push(rc_node.clone());

                        // Add the node to the labels map
                        for label in current_labels.clone() {
                            labels.insert(label.data.0.clone(), rc_node.clone());
                        }

                        // Clear the current labels
                        current_labels.clear();

                        // Add the node to the graph
                        nodes.push(Rc::new(CFGNode::new(node, HashSet::new())));
                    } else {
                        let rc_node = Rc::new(CFGNode::new(node.clone(), current_labels.clone()));

                        // Add the node to the graph
                        nodes.push(rc_node.clone());

                        // Add the node to the labels map
                        for label in current_labels.clone() {
                            labels.insert(label.data.0.clone(), rc_node.clone());
                        }

                        // Clear the current labels
                        current_labels.clear();
                    }
                }
            }
        }

        Ok(BaseCFG {
            nodes,
            label_function_map: HashMap::new(),
            label_node_map: labels,
        })
    }
}
