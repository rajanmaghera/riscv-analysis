use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::parser::{LabelString, ParserNode, With};

pub type LabelToNode = HashMap<LabelString, Rc<ParserNode>>;
pub type LabelToNodes = HashMap<LabelString, HashSet<Rc<ParserNode>>>;
pub type NodeToNodes = HashMap<Rc<ParserNode>, HashSet<Rc<ParserNode>>>;
pub type NodeToPotentialLabel = HashMap<Rc<ParserNode>, With<LabelString>>;
