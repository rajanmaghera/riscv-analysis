use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::parser::{ASTNode, LabelString, WithToken};

use super::{BasicBlock, Direction};

pub type DirectionMap = HashMap<Rc<BasicBlock>, Direction>;
pub type LabelToNode = HashMap<LabelString, Rc<ASTNode>>;
pub type LabelToNodes = HashMap<LabelString, HashSet<Rc<ASTNode>>>;
pub type NodeToNodes = HashMap<Rc<ASTNode>, HashSet<Rc<ASTNode>>>;
pub type BlockSet = HashSet<Rc<BasicBlock>>;
pub type NodeToPotentialLabel = HashMap<Rc<ASTNode>, WithToken<LabelString>>;
