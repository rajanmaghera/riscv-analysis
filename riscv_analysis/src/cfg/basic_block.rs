use std::collections::HashSet;
use std::rc::Rc;

use uuid::Uuid;

use crate::cfg::CfgNode;
use crate::parser::{HasIdentity, InstructionProperties, LabelString, With};
use crate::passes::DiagnosticLocation;

/// A group of linearly-executed instructions
///
/// A basic block is a linear sequence of code with a single-entry and single-exit. Each instruction
/// on its own also a basic block.
#[derive(Clone, Debug)]
pub struct BasicBlock {
    /// The nodes in the basic block.
    nodes: Vec<Rc<CfgNode>>,
    /// The ids of the nodes in the basic block,
    /// used for fast lookups of whether a given node is in the block.
    ///
    /// The `BasicBlock` API keeps the `node_ids` synchronized with the nodes vector.
    node_ids: HashSet<Uuid>,
}

impl BasicBlock {
    /// Create a new basic block, consuming a list of nodes.
    #[must_use]
    pub fn new(nodes: Vec<Rc<CfgNode>>) -> Self {
        let node_ids = nodes.iter().map(|n| n.id()).collect::<HashSet<_>>();
        BasicBlock { nodes, node_ids }
    }

    /// Create a new empty basic block.
    #[must_use]
    pub fn new_empty() -> Self {
        BasicBlock {
            nodes: Vec::new(),
            node_ids: HashSet::new(),
        }
    }

    /// Check if the basic block contains no nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.node_ids.is_empty()
    }

    /// Push a node into the basic block,
    /// unless the basic block already contains the node.
    ///
    /// Returns true if the node was pushed,
    /// or false if the node is already in the basic block.
    #[allow(clippy::must_use_candidate)]
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
    #[must_use]
    pub fn leader(&self) -> Option<Rc<CfgNode>> {
        let leader = self.nodes.first()?;
        Some(Rc::clone(leader))
    }

    /// Get the last node in the basic block.
    ///
    /// Returns None if the basic block is empty.
    #[must_use]
    pub fn terminator(&self) -> Option<Rc<CfgNode>> {
        let terminator = self.nodes.last()?;
        Some(Rc::clone(terminator))
    }

    /// Get the number of nodes in the basic block.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the given node is the leader of the basic block.
    #[must_use]
    pub fn is_leader_of(&self, node: &Rc<CfgNode>) -> bool {
        let Some(leader) = self.leader() else {
            return false;
        };
        *leader == **node
    }

    /// Check if the given node is the terminator of the basic block.
    #[must_use]
    pub fn is_terminator_of(&self, node: &Rc<CfgNode>) -> bool {
        let Some(terminator) = self.terminator() else {
            return false;
        };
        *terminator == **node
    }

    /// Check if the basic block contains a node with the given id.
    #[must_use]
    pub fn contains(&self, node_id: &Uuid) -> bool {
        self.node_ids.contains(node_id)
    }

    /// Get an iterator over the nodes in this block.
    pub fn iter(&self) -> impl Iterator<Item = &Rc<CfgNode>> {
        self.nodes.iter()
    }

    /// Get the labels of this block.
    ///
    /// This is the same as the labels of the block leader,
    /// or None if the basic block is empty (there is no leader).
    #[must_use]
    pub fn labels(&self) -> Option<HashSet<With<LabelString>>> {
        let leader = self.leader()?;
        Some(leader.labels())
    }

    /// Get the canonical label of this block.
    ///
    /// The canonical label is the block leader's unique label.
    /// Returns None if the basic block is empty (there is no leader)
    /// or if the leader has more than one label (no unique canonical label).
    #[must_use]
    pub fn canonical_label(&self) -> Option<With<LabelString>> {
        let labels = self.labels()?;
        if labels.len() != 1 {
            return None;
        }
        let canonical_label = labels.iter().next()?;
        Some(canonical_label.clone())
    }

    /// Get a string to act as the heading for the basic block.
    ///
    /// The heading is the canonical label of the basic block if it exists,
    /// and the id of the block otherwise.
    #[must_use]
    pub fn heading(&self) -> String {
        match self.canonical_label() {
            Some(l) => l.to_string(),
            None => self.id().to_string(),
        }
    }
}

impl std::fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{}:", self.heading())?;
        self.iter()
            .filter(|n| n.is_instruction())
            .try_for_each(|n| writeln!(f, "{}", n.raw_text()))?;
        Ok(())
    }
}
impl HasIdentity for BasicBlock {
    /// Get the id of the basic block.
    ///
    /// This is the same as the id of its leader (unique for each block),
    /// or `Uuid::nil()` if the basic block is empty.
    fn id(&self) -> Uuid {
        let Some(leader) = self.leader() else {
            return Uuid::nil();
        };
        leader.id()
    }
}
