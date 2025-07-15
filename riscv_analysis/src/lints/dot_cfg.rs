use uuid::Uuid;

use crate::{
    cfg::{Cfg, CfgNode, BasicBlock},
    parser::{HasIdentity, InstructionProperties},
    passes::{DiagnosticManager, LintError, LintPass, PassConfiguration}
};
use std::{collections::{HashMap, HashSet}, fs::File, path::PathBuf};
use std::rc::Rc;
use std::io::Write;

// Generates a CFG in dot format
pub struct DotCFGGenerationPass;
impl LintPass<DotCFGGenerationPassConfiguration> for DotCFGGenerationPass {
    fn run(cfg: &Cfg, errors: &mut DiagnosticManager, config: &DotCFGGenerationPassConfiguration) {
        if !config.get_enabled() {
            return;
        }
        let dot_cfg_path = config.get_dot_cfg_path();
        let mut dot_cfg_file = File::create(dot_cfg_path).expect("Failed to create file at \"{}\" for DOT CFG");

        // Block leaders
        let mut leaders: HashSet<Uuid> = HashSet::new();
        // Return addresses (cfg nodes that will be returned to after a call)
        let mut return_addresses: HashSet<Uuid> = HashSet::new();
        // Returns (cfg node that returns to the caller)
        let mut returns: HashSet<Uuid> = HashSet::new();
        // Maps each target (cfg node representing entry point of function) to the cfg nodes that call it
        let mut target_to_callers_map: HashMap<Uuid, Vec<Rc<CfgNode>>> = HashMap::new();
        // Maps caller to tuple of:
        // - target (cfg node representing entry point of function)
        // - return address (cfg node that will be returned to after a call)
        // - return (cfg node that returns to the caller)
        let mut caller_info_map: HashMap<Uuid, (Rc<CfgNode>, Rc<CfgNode>, Rc<CfgNode>)> = HashMap::new();
        // Maps each function entry point to the number of times it is called in the code
        let mut call_counts: HashMap<Uuid, u32> = HashMap::new();
        // Maps each return address to its block leader
        let mut return_address_to_leader_map: HashMap<Uuid, Rc<CfgNode>> = HashMap::new();
        // Maps each return instruction to its block leader
        let mut return_inst_to_leader_map: HashMap<Uuid, Rc<CfgNode>> = HashMap::new();

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
            // - all successors of this node are leaders
            let jump_target = node.jumps_to();
            if jump_target.is_some() {
                node_is_terminator = true;
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

                // Node should have one successor: the next instruction after the call
                // The call target is not considered a successor
                assert!(succs.len() == 1);
                let return_address: &Rc<CfgNode> = succs.iter().next()
                    .expect("Call should have one successor"); // TODO push error and return

                // Update return_addresses, returns, and caller_info_map
                return_addresses.insert(return_address.id());
                let target = Rc::clone(call_target_instruction);
                let return_address = Rc::clone(return_address);

                let target_label = target.labels().iter().next()
                    .expect("Call target should have a label") // TODO push error and return
                    .to_owned();
                let called_function = Rc::clone(
                    cfg.functions()
                    .get(&target_label)
                    .expect("Call target should be a function") // TODO push error and return
                );
                let called_function_return = called_function.exit().clone();
                returns.insert(called_function_return.id());

                caller_info_map.insert(node.id(), (target, return_address, called_function_return));
                call_counts.insert(called_function.entry().id(), 0);
            }
            
            if node_is_leader {
                leaders.insert(node.id());
            }

            if node_is_terminator {
                // If node is terminator, all successors are leaders
                for succ in succs.iter() {
                    leaders.insert(succ.id());
                }
            }
        }

        // Create basic blocks
        let mut ids_to_blocks: HashMap<Uuid, BasicBlock> = HashMap::new();
        let mut current_block = BasicBlock::new_empty(); // need to initialize here to make compiler happy
        for node in cfg.iter() {
            // If node is leader, add the previous block to the ids_to_blocks map and begin the next block
            if leaders.contains(&node.id()) {
                if !current_block.is_empty() {
                    ids_to_blocks.insert(current_block.id(), current_block);
                }
                current_block = BasicBlock::new_empty();
            }

            // Add node to current block
            // It should not be in the current block already.
            assert!(current_block.push(Rc::clone(&node)));
            
            // Update return_address_to_leader_map if node is return address
            if return_addresses.contains(&node.id()) {
                return_address_to_leader_map.insert(
                    node.id(), 
                    Rc::clone(&current_block.leader()
                        .expect("Current block should have leader")  // TODO proper error handling
                    )
                );
            }

            // Update return_inst_to_leader_map if node is return
            if returns.contains(&node.id()) {
                return_inst_to_leader_map.insert(
                    node.id(),
                    Rc::clone(&current_block.leader()
                        .expect("Current block should have leader")  // TODO proper error handling
                    )
                );
            }
        }
        // Add last block to ids_to_blocks map
        if !current_block.is_empty() {
            ids_to_blocks.insert(current_block.id(), current_block);
        }

        // Begin DOT graph and set node style
        writeln!(dot_cfg_file, "digraph cfg {{").expect("File \"{}\" should be writable");
        writeln!(dot_cfg_file, "\tnode [shape=record, fontname=\"Courier\"];").expect("File \"{}\" should be writable");
        for node in cfg.iter() {
            // If node is not leader, skip it
            if !leaders.contains(&node.id()) {
                continue;
            }

            // If node is leader, identify the block and print it in DOT format
            let leader = node;
            let leader_id = leader.id();

            let current_block = ids_to_blocks.get(&leader_id)
                .expect("Leader id {} not found in ids_to_blocks HashMap"); // TODO push error and return

            let terminator = current_block.terminator()
                .expect("Nonempty block should have a terminator");  // TODO push error and return
            let terminator_id = terminator.id();

            writeln!(dot_cfg_file, "\t{}", current_block.dot_str()).expect("DOT CFG file should be writable");

            // Print call and return as dashed edges in DOT format
            if let Some((call_target, return_address, return_inst)) = caller_info_map.get(&terminator_id) {
                let call_count = call_counts.get_mut(&call_target.id())
                    .expect("Call target should be mapped to call count");  // TODO push error and return
                *call_count += 1;

                writeln!(dot_cfg_file, 
                    "\t\"{}\" -> \"{}\"[style=\"dashed\", label=\"call from site {}\"]",
                    current_block.id(),
                    call_target.id(),
                    call_count,
                ).expect("DOT CFG file should be writable");

                let return_inst_block_leader = return_inst_to_leader_map.get(&return_inst.id())
                    .expect("Return instruction should be mapped to its block leader");  // TODO push error and return
                let return_address_block_leader = return_address_to_leader_map.get(&return_address.id())
                    .expect("Return address should be mapped to its block leader");  // TODO push error and return

                writeln!(dot_cfg_file, 
                    "\t\"{}\" -> \"{}\"[style=\"dashed\", label=\"return after call site {}\"];",
                    return_inst_block_leader.id(),
                    return_address_block_leader.id(),
                    call_count,
                ).expect("DOT CFG file should be writable");
            }

            // Print outgoing edges to all successor basic blocks in DOT format
            let succs = terminator.nexts();
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
                writeln!(dot_cfg_file, 
                    "\t\"{}\" -> {{ \"{}\" }};",
                    current_block.id(),
                    succ_string
                ).expect("DOT CFG file should be writable");
            }
        }
        // End DOT graph
        writeln!(dot_cfg_file, "}}").expect("DOT CFG file should be writable");
    }
}
#[derive(Default)] // pass should be disabled by default
pub struct DotCFGGenerationPassConfiguration {
    /// Is the pass enabled?
    enabled: bool,
    /// The path of the file to write the CFG to
    dot_cfg_path: PathBuf,
}
impl PassConfiguration for DotCFGGenerationPassConfiguration {
    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled
    }
}
impl DotCFGGenerationPassConfiguration {
    pub fn get_dot_cfg_path(&self) -> &PathBuf {
        &self.dot_cfg_path
    }

    pub fn set_dot_cfg_path(&mut self, dot_cfg_path: PathBuf) {
        self.dot_cfg_path = dot_cfg_path
    }
}
