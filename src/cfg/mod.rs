use crate::parser::ast::ASTNode;
use crate::parser::ast::EqNodeDataVec;
use crate::parser::parser::Parser;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;
use std::str::FromStr;
use uuid::Uuid;

// This module handles grouping of basic blocks along with conversion into Rc types,
// and the beginning of the CFG.
//
// At this point, we don't know how to handle jumps to labels, so we check on building
// the CFG that all labels are defined.
//
// TODO handle jumps to labels

// -- BASIC BLOCK ---
#[derive(Debug)]
pub struct BasicBlock(pub Vec<Rc<ASTNode>>, pub Uuid);
impl PartialEq for BasicBlock {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}
impl Eq for BasicBlock {}
impl std::hash::Hash for BasicBlock {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.1.hash(state);
    }
}

// -- DATA WRAPPER FOR BASIC BLOCK --

pub struct BlockDataWrapper<'a>(pub &'a BasicBlock);
impl PartialEq for BlockDataWrapper<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0 .0.data() == other.0 .0.data()
    }
}
pub trait BlockWrapper {
    fn data(&self) -> BlockDataWrapper;
}
impl BlockWrapper for BasicBlock {
    fn data(&self) -> BlockDataWrapper {
        BlockDataWrapper(self)
    }
}

// -- DATA WRAPPER FOR VEC OF BASIC BLOCKS --

#[derive(Debug)]
pub struct VecBlockDataWrapper<'a>(pub &'a Vec<Rc<BasicBlock>>);
impl PartialEq for VecBlockDataWrapper<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0
            .iter()
            .map(|x| x.data())
            .collect::<Vec<BlockDataWrapper>>()
            == other
                .0
                .iter()
                .map(|x| x.data())
                .collect::<Vec<BlockDataWrapper>>()
    }
}
pub trait VecBlockWrapper {
    fn data(&self) -> VecBlockDataWrapper;
}
impl VecBlockWrapper for Vec<Rc<BasicBlock>> {
    fn data(&self) -> VecBlockDataWrapper {
        VecBlockDataWrapper(self)
    }
}

// -- BASIC BLOCK IMPLEMENTATION --

impl BasicBlock {
    pub fn new(nodes: Vec<Rc<ASTNode>>) -> BasicBlock {
        BasicBlock(nodes, Uuid::new_v4())
    }

    pub fn push(&mut self, node: Rc<ASTNode>) {
        self.0.push(node);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl IntoIterator for BasicBlock {
    type Item = Rc<ASTNode>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Default for BasicBlock {
    fn default() -> Self {
        Self::new(vec![])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CFG {
    pub blocks: Vec<Rc<BasicBlock>>,
    pub labels: HashMap<String, Rc<BasicBlock>>,
    pub labels_for_branch: Vec<Vec<String>>,
}

impl FromStr for CFG {
    type Err = CFGError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parser = Parser::new(s);
        let ast = parser.collect::<Vec<ASTNode>>();
        CFG::new(ast)
    }
}

#[derive(Debug)]
pub enum CFGError {
    LabelNotDefined,
}

impl Display for CFG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        let mut labels = self.labels_for_branch.iter();

        for block in self.blocks.iter() {
            s.push_str("/---------\n");
            s.push_str(&format!("| LABELS: {:?}\n", labels.next().unwrap()));
            for node in block.0.iter() {
                s.push_str(&format!("| {}\n", node));
            }
            s.push_str("\\--------\n");
        }
        write!(f, "{}", s)
    }
}

impl CFG {
    pub fn new(nodes: Vec<ASTNode>) -> Result<CFG, CFGError> {
        let mut labels = HashMap::new();
        let mut blocks = Vec::new();
        let mut current_block = BasicBlock::default();
        let mut last_labels: Vec<String> = Vec::new();
        let mut labels_for_branch: Vec<Vec<String>> = Vec::new();

        for node in nodes {
            match node {
                ASTNode::Label(s) => {
                    if current_block.len() > 0 {
                        let rc = Rc::new(current_block);
                        for label in last_labels.iter() {
                            if labels.insert(label.to_owned(), rc.clone()) != None {
                                return Err(CFGError::LabelNotDefined);
                            }
                        }
                        labels_for_branch.push(last_labels.clone());
                        last_labels.clear();
                        blocks.push(rc);
                        let new_block = BasicBlock::default();
                        current_block = new_block;
                    }
                    last_labels.push(s.name.data.0);
                }
                ASTNode::Branch(_)
                | ASTNode::JumpLink(_)
                | ASTNode::JumpLinkR(_)
                | ASTNode::Basic(_) => {
                    current_block.push(Rc::new(node));
                    // end block
                    let rc = Rc::new(current_block);
                    for label in last_labels.iter() {
                        if labels.insert(label.to_owned(), rc.clone()) != None {
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
                    current_block.push(Rc::new(node));
                }
            }
        }

        if current_block.len() > 0 {
            let rc = Rc::new(current_block);
            for label in last_labels.iter() {
                if labels.insert(label.to_owned(), rc.clone()) != None {
                    return Err(CFGError::LabelNotDefined);
                }
            }
            labels_for_branch.push(last_labels.clone());
            blocks.push(rc);
        }

        Ok(CFG {
            blocks,
            labels,
            labels_for_branch,
        })
    }
}
