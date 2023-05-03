use crate::parser::ast::ASTNode;
use crate::parser::parser::Parser;
use crate::parser::token::{Token, WithToken};
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

// This module handles grouping of basic blocks along with conversion into Rc types,
// and the beginning of the CFG.
//
// At this point, we don't know how to handle jumps to labels, so we check on building
// the CFG that all labels are defined.
//
// TODO handle jumps to labels

#[derive(Debug, PartialEq)]
pub struct BasicBlock(pub Vec<Rc<WithToken<ASTNode>>>);

impl BasicBlock {
    pub fn new(nodes: Vec<Rc<WithToken<ASTNode>>>) -> BasicBlock {
        BasicBlock(nodes)
    }

    pub fn from_nodes(nodes: Vec<WithToken<ASTNode>>) -> Rc<BasicBlock> {
        let mut rc_nodes = Vec::new();
        for node in nodes {
            rc_nodes.push(Rc::new(node));
        }
        Rc::new(BasicBlock(rc_nodes))
    }

    pub fn push(&mut self, node: Rc<WithToken<ASTNode>>) {
        self.0.push(node);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl IntoIterator for BasicBlock {
    type Item = Rc<WithToken<ASTNode>>;
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

#[derive(Debug, PartialEq)]
pub struct CFG {
    pub blocks: Vec<Rc<BasicBlock>>,
    pub labels: HashMap<String, Rc<BasicBlock>>,
}

impl FromStr for CFG {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parser = Parser::new(s);
        let ast = parser.collect::<Vec<WithToken<ASTNode>>>();
        CFG::new(ast)
    }
}

impl CFG {
    pub fn new(nodes: Vec<WithToken<ASTNode>>) -> Result<CFG, ()> {
        let mut labels = HashMap::new();
        let mut blocks = Vec::new();
        let mut current_block = BasicBlock::default();
        let mut last_labels: Vec<String> = Vec::new();

        for node in nodes {
            dbg!(&node);
            match node.data {
                ASTNode::Label(s) => {
                    if current_block.len() > 0 {
                        let rc = Rc::new(current_block);
                        for label in last_labels.iter() {
                            if labels.insert(label.to_owned(), rc.clone()) != None {
                                return Err(());
                            }
                        }
                        last_labels.clear();
                        blocks.push(rc);
                        let new_block = BasicBlock::default();
                        current_block = new_block;
                    }
                    last_labels.push(s);
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
                    return Err(());
                }
            }
            blocks.push(rc);
        }

        Ok(CFG { blocks, labels })
    }
}
