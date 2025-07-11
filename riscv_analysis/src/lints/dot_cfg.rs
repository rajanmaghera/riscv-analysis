use crate::{
    cfg::{Cfg, CfgNode},
    passes::{DiagnosticLocation, DiagnosticManager, LintError, LintPass},
};
use std::collections::HashMap;
use std::rc::Rc;

// Generates a CFG in dot format
pub struct DotCFGGenerationPass;
impl LintPass for DotCFGGenerationPass {
    fn run(cfg: &Cfg, errors: &mut DiagnosticManager) {
        // Begin DOT graph and set node style
        println!("digraph cfg {{");
        println!("\tnode [shape=record, fontname=\"Courier\"];");

        // Create map of nodes to indices in order to have a unique ID for each node
        let nodes_to_indices: HashMap<Rc<CfgNode>, usize> =
            cfg.iter().enumerate().map(|(i, node)| (node, i)).collect();

        // Iterate over the CFG
        for (node_index, node) in cfg.iter().enumerate() {
            // Print node in DOT format
            let node_text = node.raw_text();
            if node_text.is_empty() {
                println!("\t{} [label=\"{}\"];", node_index, node_index);
            } else {
                println!(
                    "\t{} [label=\"{{{}:\\l|\t {}\\l}}\"];",
                    node_index,
                    node_index,
                    // square brackets, vertical bars, and angle brackets must be escaped
                    // see https://graphviz.org/doc/info/shapes.html#record
                    node_text
                        .replace("[", "\\[")
                        .replace("]", "\\]")
                        .replace("|", "\\|")
                        .replace("<", "\\<")
                        .replace(">", "\\>")
                );
            }

            // Specify edges by using indices of successors
            let mut successor_error = false;
            let successors = node.nexts();
            let successor_string = successors
                .iter()
                .map(|succ| match nodes_to_indices.get(succ) {
                    Some(s) => s.to_string(),
                    None => {
                        errors.push(LintError::DotCFGNodeHasNoIndex(succ.node()));
                        successor_error = true;
                        String::new()
                    }
                })
                .collect::<Vec<String>>()
                .join(" ");
            if successor_error {
                return;
            }

            // Print outgoing edges in DOT format
            if !successors.is_empty() {
                println!(
                    "\t{} -> {{ {} }};",
                    node_index.to_string(),
                    successor_string
                );
            }
        }
        // End DOT graph
        println!("}}");
    }
}
