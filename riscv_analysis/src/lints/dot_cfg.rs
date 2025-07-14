use uuid::Uuid;

use crate::{
    cfg::{Cfg, CfgNode},
    parser::{HasIdentity, InstructionProperties, LabelString, With},
    passes::{DiagnosticLocation, DiagnosticManager, LintError, LintPass}
};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Debug)]

struct BasicBlock {
    nodes: Vec<Rc<CfgNode>>,
}

impl BasicBlock {
    pub fn new(nodes: Vec<Rc<CfgNode>>) -> Self {
        BasicBlock {
            nodes: nodes,
        }
    }

    pub fn new_empty() -> Self {
        BasicBlock {
            nodes: Vec::new(),
        }
    }

    pub fn leader(&self) -> Option<Rc<CfgNode>> {
        let Some(leader) = self.nodes.first() else { return None };
        Some(Rc::clone(leader))
    }

    pub fn terminator(&self) -> Option<Rc<CfgNode>> {
        let Some(leader) = self.nodes.last() else { return None };
        Some(Rc::clone(leader))
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_leader_of(&self, node: &Rc<CfgNode>) -> bool {
        let Some(leader) = self.leader() else { return false };
        *leader == **node
    }

    pub fn is_terminator_of(&self, node: &Rc<CfgNode>) -> bool {
        let Some(terminator) = self.terminator() else { return false };
        *terminator == **node
    }

    pub fn is_in(&self, node: &Rc<CfgNode>) -> bool {
        for own_node in self.iter() {
            if **own_node == **node { return true }
        };
        false
    }

    pub fn id(&self) -> Uuid {
        let Some(leader) = self.leader() else { return Uuid::nil() };
        leader.id()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Rc<CfgNode>> {
        self.nodes.iter()
    }

    pub fn labels(&self) -> Option<HashSet<With<LabelString>>> {
        let Some(leader) = self.leader() else { return None };
        Some(leader.labels())
    }

    pub fn canonical_label(&self) -> Option<With<LabelString>> {
        let Some(labels) = self.labels() else { return None };
        if labels.len() != 1 {
            return None;
        }
        let Some(label) = labels.iter().next() else { return None };
        Some(label.clone())
    }

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

        let heading = match self.canonical_label() {
            Some(l) => l.to_string(),
            None => self.id().to_string()
        };

        format!("\"{}\" [label=\"{{{}:\\l|{}\\l}}\"]", self.id(), heading, instruction_string)
    }
}

// Generates a CFG in dot format
pub struct DotCFGGenerationPass;
impl LintPass for DotCFGGenerationPass {
    fn run(cfg: &Cfg, errors: &mut DiagnosticManager) {
        // Identify block leaders/terminators and callers/call targets
        let mut leaders: HashSet<Uuid> = HashSet::new();
        let mut terminators: HashSet<Uuid> = HashSet::new();
        let mut target_to_callers_map: HashMap<Uuid, Vec<Rc<CfgNode>>> = HashMap::new(); // each function can have multiple callers
        let mut caller_to_target_map: HashMap<Uuid, Rc<CfgNode>> = HashMap::new(); // each caller calls exactly one function

        for node in cfg.iter() {
            let prevs = node.prevs();
            let succs = node.nexts();

            let mut node_is_leader = false;
            let mut node_is_terminator = false;

            // If node has no predecessors or is program entry, node is leader
            if prevs.is_empty() || node.is_program_entry() {
                node_is_leader = true;
            }

            // If node is has no successors or is program exit, node is terminator
            if succs.is_empty() || node.is_program_exit() {
                node_is_terminator = true;
            }

            // If node has jump target:
            // - node is terminator
            // - target is leader
            // - predecessors of target are terminators (cannot jump into middle of basic block)
            // - other successors of node are leaders
            let jump_target = node.jumps_to();
            if let Some(label_string) = jump_target {
                node_is_terminator = true;

                // Get jump target instruction
                let jump_target_instruction = cfg.label_node_map
                .get(label_string.as_str())
                .expect("Jump target should be in the label node map"); // TODO push error and return

                // All predecessors of jump target are terminators
                for prev in jump_target_instruction.prevs().iter() {
                    terminators.insert(prev.id());
                }
            }

            // If node has call target:
            // - node is terminator
            // - target is leader
            // Note: call targets are not considered successors
            let call_target = node.calls_to();
            if let Some(label_string) = call_target {
                node_is_terminator = true;

                let call_target_instruction = cfg.label_node_map
                .get(label_string.as_str())
                .expect("Call target should be in the label node map"); // TODO push error and return
                leaders.insert(call_target_instruction.id());

                // Update target_to_callers_map
                match target_to_callers_map.get_mut(&call_target_instruction.id()) {
                    Some(callers) => {
                        callers.push(Rc::clone(&node));
                    },
                    None => {
                        let mut callers: Vec<Rc<CfgNode>> = Vec::new();
                        callers.push(Rc::clone(&node));
                        target_to_callers_map.insert(call_target_instruction.id(), callers);
                    }
                };

                // Update caller_to_target_map
                caller_to_target_map.insert(node.id(), Rc::clone(call_target_instruction));
            }

            
            if node_is_leader {
                leaders.insert(node.id());
            }

            if node_is_terminator {
                terminators.insert(node.id());

                // If node is terminator, all successors must be leaders
                for succ in succs.iter() {
                    leaders.insert(succ.id());
                }
            }
        }

        // Create basic blocks
        let mut ids_to_blocks: HashMap<Uuid, BasicBlock> = HashMap::new();
        let mut current_block = BasicBlock::new_empty();

        for node in cfg.iter() {
            current_block.nodes.push(Rc::clone(&node));
            if terminators.contains(&node.id()) {
                ids_to_blocks.insert(current_block.id(), current_block);
                current_block = BasicBlock::new_empty();
            }
        }

        // Begin DOT graph and set node style
        println!("digraph cfg {{");
        println!("\tnode [shape=record, fontname=\"Courier\"];");

        let mut current_block = &BasicBlock::new_empty();
        for node in cfg.iter() {
            let node_id = node.id();

            // If node is leader, update current block 
            if leaders.contains(&node_id) {
                current_block = ids_to_blocks.get(&node_id)
                .unwrap_or_else(|| panic!("Leader id {} not found in ids_to_blocks HashMap", node_id)); // TODO push error and return
            }

            // If node is terminator, print block in DOT format and indicate outgoing edges
            if terminators.contains(&node_id) {
                // Print block in DOT format
                println!("\t{}", current_block.dot_str());

                // Print successors in DOT format
                if let Some(call_target) = caller_to_target_map.get(&node.id()) {
                    // If node is call, then it only has one successor (the call target)
                    let succ_string = call_target.id();

                    println!(
                        "\t\"{}\" -> {{ \"{}\" }};",
                        current_block.id(),
                        succ_string
                    );
                } else {
                    // Otherwise, print all successors
                    let succs = node.nexts();
                    let mut succ_error = false;
                    let succ_string = succs
                    .iter()
                    .map(|succ| if ids_to_blocks.contains_key(&succ.id()) {
                        succ.id().to_string()
                    } else {
                        errors.push(LintError::DotCFGSuccessorOfTerminatorIsNotLeader(succ.node()));
                        succ_error = true;
                        String::new()
                    })
                    .collect::<Vec<String>>()
                    .join("\" \"");
                    
                    if succ_error {
                        return;
                    }

                    if !succs.is_empty() {
                        println!(
                            "\t\"{}\" -> {{ \"{}\" }};",
                            current_block.id(),
                            succ_string
                        );
                    }
                }
            }
        }
        // End DOT graph
        println!("}}");
    }
}
