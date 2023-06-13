use crate::parser::ast::ASTNode;
use crate::parser::ast::EqNodeDataVec;


use std::rc::Rc;
use uuid::Uuid;

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
