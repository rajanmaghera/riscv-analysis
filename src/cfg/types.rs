use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::parser::{LabelString, Node, With};

pub type LabelToNode = HashMap<LabelString, Rc<Node>>;
pub type LabelToNodes = HashMap<LabelString, HashSet<Rc<Node>>>;
pub type NodeToNodes = HashMap<Rc<Node>, HashSet<Rc<Node>>>;
pub type NodeToPotentialLabel = HashMap<Rc<Node>, With<LabelString>>;
