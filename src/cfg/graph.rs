use super::BasicBlock;
use crate::parser::Node;
use crate::parser::Parser;
use crate::parser::Register;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct BaseCFG {
    pub blocks: Vec<Rc<BasicBlock>>,
    pub nodes: Vec<Rc<Node>>,
    pub labels: HashMap<String, Rc<BasicBlock>>,
    pub labels_for_branch: Vec<Vec<String>>,
}

impl FromStr for BaseCFG {
    type Err = CFGError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parser = Parser::new(s);
        let ast = parser.collect::<Vec<Node>>();
        BaseCFG::new(ast)
    }
}
// todo move to cfg.into_nodes_iter() with separate struct wrapper
impl IntoIterator for BaseCFG {
    type Item = Rc<Node>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    // nested iterator for blocks
    fn into_iter(self) -> Self::IntoIter {
        self.blocks
            .into_iter()
            .flat_map(|x| x.0.clone())
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[derive(Debug)]
pub enum CFGError {
    LabelNotDefined,
}

impl Display for BaseCFG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        let mut labels = self.labels_for_branch.iter();

        for block in &self.blocks {
            s.push_str("/---------\n");
            s.push_str(&format!(
                "| LABELS: {:?}, ID: {}\n",
                labels.next().unwrap(),
                // get last 8 chars of uuid
                &block.1.as_simple().to_string().get(..8).unwrap_or("")
            ));
            for node in &block.0 {
                s.push_str(&format!("| {node}\n"));
            }
            s.push_str("\\--------\n");
        }
        write!(f, "{s}")
    }
}

impl BaseCFG {
    pub fn new(nodes: Vec<Node>) -> Result<BaseCFG, CFGError> {
        // TODO transition nodes/blocks to iterator of single type
        let mut labels = HashMap::new();
        let mut blocks = Vec::new();
        let mut new_nodes = Vec::new();
        let mut current_block = BasicBlock::default();
        let mut last_labels: Vec<String> = Vec::new();
        let mut labels_for_branch: Vec<Vec<String>> = Vec::new();
        let mut func_labels = HashSet::new();
        let mut non_func_labels = HashSet::new();

        // FIND ALL POTENTIAL FUNCTION LABELS
        for node in &nodes {
            match node {
                Node::JumpLink(x) => {
                    // TODO determine if ra is set to some value
                    // if the inst sets the ra, then it is a function
                    if x.rd == Register::X1 {
                        func_labels.insert(x.name.clone());
                    } else {
                        non_func_labels.insert(x.name.clone());
                    }
                }
                Node::Branch(x) => {
                    non_func_labels.insert(x.name.clone());
                }
                _ => (),
            }
        }

        // ADD PROGRAM START NODE
        let start_node = Rc::new(Node::new_program_entry());
        current_block.push(Rc::clone(&start_node));
        new_nodes.push(start_node);

        for node in nodes {
            match node {
                Node::Label(s) => {
                    if current_block.len() > 0 {
                        let rc = Rc::new(current_block);
                        for label in &last_labels {
                            if labels.insert(label.clone(), Rc::clone(&rc)).is_some() {
                                return Err(CFGError::LabelNotDefined);
                            }
                        }
                        labels_for_branch.push(last_labels.clone());
                        last_labels.clear();
                        blocks.push(rc);
                        let new_block = BasicBlock::default();
                        current_block = new_block;
                    }
                    // if label is a function label, add it to the block
                    if func_labels.contains(&s.name) {
                        let current_node = Rc::new(Node::new_func_entry(s.name.clone()));
                        current_block.push(Rc::clone(&current_node));
                        new_nodes.push(current_node);
                    }
                    last_labels.push(s.name.data.0);
                }
                _ if matches!(node.potential_jumps_to(), Some(_)) => {
                    let new_node = Rc::new(node);
                    current_block.push(Rc::clone(&new_node));
                    new_nodes.push(new_node);
                    // end block
                    let rc = Rc::new(current_block);
                    for label in &last_labels {
                        if labels.insert(label.clone(), Rc::clone(&rc)).is_some() {
                            return Err(CFGError::LabelNotDefined);
                        }
                    }
                    labels_for_branch.push(last_labels.clone());
                    last_labels.clear();
                    blocks.push(rc);
                    let new_block = BasicBlock::default();
                    current_block = new_block;
                }
                _ => {
                    let new_node = Rc::new(node);
                    new_nodes.push(Rc::clone(&new_node));
                    current_block.push(new_node);
                }
            }
        }

        if current_block.len() > 0 {
            let rc = Rc::new(current_block);
            for label in &last_labels {
                if labels.insert(label.clone(), Rc::clone(&rc)).is_some() {
                    return Err(CFGError::LabelNotDefined);
                }
            }
            labels_for_branch.push(last_labels.clone());
            blocks.push(rc);
        }

        Ok(BaseCFG {
            blocks,
            nodes: new_nodes,
            labels,
            labels_for_branch,
        })
    }
}
