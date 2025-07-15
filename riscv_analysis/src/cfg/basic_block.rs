use std::collections::HashSet;
use std::rc::Rc;

use uuid::Uuid;

use crate::cfg::CfgNode;
use crate::parser::{HasIdentity, InstructionProperties, LabelString, With};
use crate::passes::DiagnosticLocation;

pub struct BasicBlock {
    /// The nodes in the basic block.
    nodes: Vec<Rc<CfgNode>>,
    /// The ids of the nodes in the basic block,
    /// used for fast lookups of whether a given node is in the block.
    /// 
    /// The BasicBlock API keeps the node_ids synchronized with the nodes vector.
    node_ids: HashSet<Uuid>,
}

impl BasicBlock {
    /// Create a new basic block, consuming a list of nodes.
    pub fn new(nodes: Vec<Rc<CfgNode>>) -> Self {
        let node_ids = HashSet::from_iter(nodes.iter().map(|n| n.id()));
        BasicBlock {
            nodes: nodes,
            node_ids,
        }
    }

    /// Create a new empty basic block.
    pub fn new_empty() -> Self {
        BasicBlock {
            nodes: Vec::new(),
            node_ids: HashSet::new(),
        }
    }

    /// Check if the basic block contains no nodes.
    pub fn is_empty(&self) -> bool {
        self.node_ids.is_empty()
    }

    /// Push a node into the basic block,
    /// unless the basic block already contains the node.
    /// 
    /// Returns true if the node was pushed,
    /// or false if the node is already in the basic block.
    pub fn push(&mut self, node: Rc<CfgNode>) -> bool {
        let node_id = node.id();
        if self.contains(&node_id) {
            false
        } else {
            self.nodes.push(node);
            self.node_ids.insert(node_id);
            true
        }
    }

    /// Get the first node in the basic block.
    /// 
    /// Returns None if the basic block is empty.
    pub fn leader(&self) -> Option<Rc<CfgNode>> {
        let Some(leader) = self.nodes.first() else { return None };
        Some(Rc::clone(leader))
    }

    /// Get the last node in the basic block.
    /// 
    /// Returns None if the basic block is empty.
    pub fn terminator(&self) -> Option<Rc<CfgNode>> {
        let Some(leader) = self.nodes.last() else { return None };
        Some(Rc::clone(leader))
    }

    /// Get the number of nodes in the basic block.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the given node is the leader of the basic block.
    pub fn is_leader_of(&self, node: &Rc<CfgNode>) -> bool {
        let Some(leader) = self.leader() else { return false };
        *leader == **node
    }

    /// Check if the given node is the terminator of the basic block.
    pub fn is_terminator_of(&self, node: &Rc<CfgNode>) -> bool {
        let Some(terminator) = self.terminator() else { return false };
        *terminator == **node
    }

    /// Check if the basic block contains a node with the given id.
    pub fn contains(&self, node_id: &Uuid) -> bool {
        self.node_ids.contains(&node_id)
    }

    /// Get the id of the basic block.
    /// 
    /// This is the same as the id of its leader (unique for each block),
    /// or Uuid::nil() if the basic block is empty.
    pub fn id(&self) -> Uuid {
        let Some(leader) = self.leader() else { return Uuid::nil() };
        leader.id()
    }

    /// Get an iterator over the nodes in this block.
    pub fn iter(&self) -> impl Iterator<Item = &Rc<CfgNode>> {
        self.nodes.iter()
    }

    /// Get the labels of this block.
    /// 
    /// This is the same as the labels of the block leader,
    /// or None if the basic block is empty (there is no leader).
    pub fn labels(&self) -> Option<HashSet<With<LabelString>>> {
        let Some(leader) = self.leader() else { return None };
        Some(leader.labels())
    }

    /// Get the canonical label of this block.
    /// 
    /// The canonical label is the block leader's unique label.
    /// Returns None if the basic block is empty (there is no leader)
    /// or if the leader has more than one label (no unique canonical label).
    pub fn canonical_label(&self) -> Option<With<LabelString>> {
        let Some(labels) = self.labels() else { return None };
        if labels.len() != 1 {
            return None;
        }
        let Some(label) = labels.iter().next() else { return None };
        Some(label.clone())
    }

    /// Get a string to act as the heading for the basic block's DOT representation.
    /// 
    /// The heading is the canonical label of the basic block if it exists,
    /// and the id of the block otherwise.
    pub fn dot_str_heading(&self) -> String {
        match self.canonical_label() {
            Some(l) => l.to_string(),
            None => self.id().to_string()
        }
    }

    /// Get a string that represents this basic block as a record-based node in DOT format.
    /// 
    /// See the DOT language reference at https://graphviz.org/doc/info/lang.html.
    /// Record-based nodes are described at https://graphviz.org/doc/info/shapes.html#record.
    pub fn dot_str(&self) -> String {
        let instruction_string = self.iter()
        .filter(|n| n.is_instruction())
        .map(|n| n.raw_text())
        .collect::<Vec<String>>()
        .join("\\l")
        // square brackets, vertical bars, and angle brackets must be escaped
        // see https://graphviz.org/doc/info/shapes.html#record
        .replace("[", "\\[")
        .replace("]", "\\]")
        .replace("|", "\\|")
        .replace("<", "\\<")
        .replace(">", "\\>");

        format!("\"{}\" [label=\"{{{}:\\l|{}\\l}}\"]", self.id(), self.dot_str_heading(), instruction_string)
    }
}
