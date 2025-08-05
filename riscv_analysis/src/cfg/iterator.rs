use crate::cfg::{Cfg, CfgNode};
use std::{collections::HashSet, rc::Rc};

/// Iterate over all nodes in the order that they appear in the source file.
///
/// If there are multiple files, nodes in the same file will be grouped by
/// the order they are included.
pub struct CfgSourceIterator<'a> {
    nodes: std::slice::Iter<'a, Rc<CfgNode>>,
}

impl<'a> CfgSourceIterator<'a> {
    /// Create a new source order iterator
    #[must_use]
    pub fn new(cfg: &'a Cfg) -> Self {
        Self {
            nodes: cfg.nodes().iter(),
        }
    }
}

impl<'a> Iterator for CfgSourceIterator<'a> {
    type Item = &'a Rc<CfgNode>;

    fn next(&mut self) -> Option<Self::Item> {
        self.nodes.next()
    }
}

impl DoubleEndedIterator for CfgSourceIterator<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.nodes.next_back()
    }
}

/// Iterate over all CFG nodes reachable from some start node, using the nexts.
///
/// This iterator uses the `nexts()` functions of each CFG node. Thus if the
/// `NodeDirectionPass` has not run yet, only the start node will be iterated
/// over.
///
/// You must not modify the key of any CFG node during the traversal.
pub struct CfgBreadthFirstIterator<'a> {
    cfg: &'a Cfg,
    queue: Vec<&'a Rc<CfgNode>>, // Nodes we have seen but not visited yet
    visited: HashSet<&'a Rc<CfgNode>>, // Nodes we have visited
}

impl<'a> CfgBreadthFirstIterator<'a> {
    /// Create a new iterator over all nodes reachable from `start`.
    #[must_use]
    pub fn new(cfg: &'a Cfg, start: &'a Rc<CfgNode>) -> Self {
        Self {
            cfg,
            queue: vec![start],
            visited: HashSet::new(),
        }
    }
}

impl<'a> Iterator for CfgBreadthFirstIterator<'a> {
    type Item = &'a Rc<CfgNode>;

    fn next(&mut self) -> Option<Self::Item> {
        // If the queue runs out, there are no more nodes that are reachable
        while let Some(node) = self.queue.pop() {
            // Skip over nodes we have already visited
            if self.visited.contains(node) {
                continue;
            }

            // Mark this node as visited
            self.visited.insert(node);

            // Add all successor nodes to the queue
            for suc in self.cfg.get_nexts(node) {
                self.queue.push(suc);
            }

            return Some(node);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::cfg::Cfg;
    use crate::parser::{ProgramEntryType, RVStringParser};

    /// Generate the complete CFG from an input string.
    fn gen_cfg(input: &str) -> Cfg {
        let parser_output = RVStringParser::parse_from_text(input);
        assert_eq!(parser_output.errors.len(), 0);
        Cfg::new(parser_output, None, &ProgramEntryType::FirstInstruction).unwrap()
    }

    #[test]
    fn empty() {
        let input = "";
        let cfg = gen_cfg(input);
        let mut iterator = cfg.iter_source();

        iterator.next(); // There is a program entry node by default
        assert_eq!(iterator.next(), None);
    }
}
