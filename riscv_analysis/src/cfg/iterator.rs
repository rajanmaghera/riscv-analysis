use crate::cfg::{Cfg, CfgNode};
use std::{collections::HashSet, rc::Rc};

/// Iterate over all nodes in a CFG.
pub struct CfgIterator<'a> {
    nodes: &'a Vec<Rc<CfgNode>>,
    start: usize,       // Location in the iteration
    end: usize,
    end_final: bool,    // True if `end` has reached the start
}

impl<'a> CfgIterator<'a> {
    /// Create a new iterator over all nodes in the CFG.
    ///
    /// Useful if you don't care about the order of the nodes.
    #[must_use]
    pub fn new(cfg: &'a Cfg) -> Self {
        let nodes = &cfg.nodes();
        Self {
            nodes,
            start: 0,
            end: nodes.len() - 1,
            end_final: false,
        }
    }

    fn in_bounds(&self) -> bool {
        self.start <= self.end && !self.end_final
    }
}

impl<'a> Iterator for CfgIterator<'a> {
    type Item = Rc<CfgNode>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.in_bounds() {
            return None;
        }

        // Return the next node, if there is one
        if let Some(node) = self.nodes.get(self.start) {
            self.start += 1;
            return Some(Rc::clone(node));
        }

        None
    }
}

impl<'a> DoubleEndedIterator for CfgIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if !self.in_bounds() {
            return None;
        }

        let result = self.nodes
                         .get(self.end)
                         .map(Rc::clone);

        if self.end == 0 {
            self.end_final = true;
        } else {
            self.end -= 1;
        }

        result
    }
}


/// Iterate over all nodes in the order that they appear in the source file.
///
/// If there are multiple files, nodes in the same file will be grouped, but
/// the files will not be given in any particular order.
pub struct CfgSourceIterator {
    nodes: Vec<Rc<CfgNode>>,
    start: usize,
}

impl CfgSourceIterator {
    /// Create a new source order iterator
    #[must_use]
    pub fn new(cfg: &Cfg) -> Self {
        let mut nodes = cfg.nodes().clone();

        // Sort by location
        nodes.sort_by(|a, b| {
            let a_pos = a.node().token().pos.end.raw_index;
            let b_pos = b.node().token().pos.end.raw_index;
            a_pos.cmp(&b_pos)
        });

        // Sort by file. We know that `sort_by` is stable, so this has the
        // effect of grouping by file
        nodes.sort_by(|a, b| {
            let a_file = a.node().token().file;
            let b_file = b.node().token().file;
            a_file.cmp(&b_file)
        });

        Self {
            nodes,
            start: 0,
        }
    }
}

impl Iterator for CfgSourceIterator {
    type Item = Rc<CfgNode>;

    fn next(&mut self) -> Option<Self::Item> {
        // Return the next node, if there is one
        if let Some(node) = self.nodes.get(self.start) {
            self.start += 1;
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
/// This iterator uses the `prevs()` functions of each CFG node. Thus if the
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