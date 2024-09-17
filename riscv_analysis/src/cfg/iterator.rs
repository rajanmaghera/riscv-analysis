use crate::cfg::{Cfg, CfgNode};
use std::{collections::HashSet, rc::Rc};

/// Iterate over a CFG using the underlying node order.
///
/// This iterator is useful when you don't care about the order you iterate over
/// the nodes.
pub struct CfgIterator {
    // We have to store a copy of the nodes, since the user may insert nodes
    // during the iteration.
    nodes: Vec<Rc<CfgNode>>,
    // Where we are in the iteration.
    index: usize,
}

impl CfgIterator {
    /// Create a new iterator over all nodes in the CFG.
    #[must_use]
    pub fn new(cfg: &Cfg) -> Self {
        Self {
            nodes: cfg.nodes.clone(),
            index: 0,
        }
    }
}

impl Iterator for CfgIterator {
    type Item = Rc<CfgNode>;

    fn next(&mut self) -> Option<Self::Item> {
        // Return the next node, if there is one
        if let Some(node) = self.nodes.get(self.index) {
            self.index += 1;
            return Some(Rc::clone(node));
        }

        None
    }
}

/// Iterate over all CFG nodes reachable from some start node, using the nexts.
///
/// This iterator uses the `nexts()` functions of each CFG node. Thus if the
/// `NodeDirectionPass` has not run yet, only the start node will be iterated
/// over.
///
/// You must not modify the key of any CFG node during the traversal.
pub struct CfgNextsIterator {
    queue: Vec<Rc<CfgNode>>,        // Nodes we have seen but not visited yet
    visted: HashSet<Rc<CfgNode>>,   // Nodes we have visited
}

impl CfgNextsIterator {
    /// Create a new iterator over all nodes reachable from `start`.
    #[must_use]
    pub fn new(start: Rc<CfgNode>) -> Self {
        Self {
            queue: vec![start],
            visted: HashSet::new(),
        }
    }
}

impl Iterator for CfgNextsIterator {
    type Item = Rc<CfgNode>;

    fn next(&mut self) -> Option<Self::Item> {
        // If the queue runs out, there are no more nodes that are reachable
        while let Some(node) = self.queue.pop() {
            // Skip over nodes we have already visited
            if self.visted.contains(&node) {
                continue;
            }

            // Mark this node as visited
            self.visted.insert(Rc::clone(&node));

            // Add all successor nodes to the queue
            for suc in node.nexts().iter() {
                self.queue.push(Rc::clone(suc));
            }

            return Some(node);
        }

        None
    }
}

/// Iterate over all CFG nodes reachable from some start node, using the prevs.
///
/// This iterator uses the `nexts()` functions of each CFG node. Thus if the
/// `NodeDirectionPass` has not run yet, only the start node will be iterated
/// over.
///
/// You must not modify the key of any CFG node during the traversal.
pub struct CfgPrevsIterator {
    queue: Vec<Rc<CfgNode>>,        // Nodes we have seen but not visited yet
    visted: HashSet<Rc<CfgNode>>,   // Nodes we have visited
}

impl CfgPrevsIterator {
    /// Create a new iterator over all nodes reachable from `start`.
    #[must_use]
    pub fn new(start: Rc<CfgNode>) -> Self {
        Self {
            queue: vec![start],
            visted: HashSet::new(),
        }
    }
}

impl Iterator for CfgPrevsIterator {
    type Item = Rc<CfgNode>;

    fn next(&mut self) -> Option<Self::Item> {
        // If the queue runs out, there are no more nodes that are reachable
        while let Some(node) = self.queue.pop() {
            // Skip over nodes we have already visited
            if self.visted.contains(&node) {
                continue;
            }

            // Mark this node as visited
            self.visted.insert(Rc::clone(&node));

            // Add all successor nodes to the queue
            for suc in node.prevs().iter() {
                self.queue.push(Rc::clone(suc));
            }

            return Some(node);
        }

        None
    }
}
