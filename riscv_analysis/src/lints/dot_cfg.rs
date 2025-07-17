use uuid::Uuid;

use crate::{
    cfg::{BasicBlock, Cfg, CfgNode, Function},
    parser::{HasIdentity, InstructionProperties, LabelString, ParserNode, With},
    passes::{DiagnosticManager, LintPass, PassConfiguration},
};
use std::io::Write;
use std::rc::Rc;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::PathBuf,
};

// Generates a CFG in dot format
pub struct DotCFGGenerationPass;
impl DotCFGGenerationPass {
    fn scan_leaders_and_calls(
        cfg: &Cfg,
        interprocedural_enabled: bool,
    ) -> Result<DotCFGGenerationPassInfo, DotCFGError> {
        let mut info = DotCFGGenerationPassInfo::new();
        let leaders = &mut info.leaders;
        let return_addresses = &mut info.return_addresses;
        let returns = &mut info.returns;
        let caller_info_map = &mut info.caller_info_map;
        for node in cfg {
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

            // If interprocedural_enabled is true and node has call target:
            // - node is terminator
            // - target is leader
            // Note: call targets are not considered successors
            if interprocedural_enabled {
                let call_target = node.calls_to();
                if let Some(label_string) = call_target {
                    node_is_terminator = true;

                    let call_target_instruction = cfg
                        .label_node_map
                        .get(label_string.as_str())
                        .ok_or_else(|| {
                            DotCFGError::CallTargetLabelNotInLabelNodeMap(label_string)
                        })?;
                    leaders.insert(call_target_instruction.id());

                    // Node should have one successor: the next instruction after the call
                    // The call target is not considered a successor
                    if succs.len() > 1 {
                        return Err(DotCFGError::CallHasMoreThanOneSuccessor(node.node()));
                    }
                    let return_address: &Rc<CfgNode> = succs
                        .iter()
                        .next()
                        .ok_or_else(|| DotCFGError::CallHasNoSuccessors(node.node()))?;

                    // Update return_addresses, returns, and caller_info_map
                    return_addresses.insert(return_address.id());
                    let target = Rc::clone(call_target_instruction);
                    let return_address = Rc::clone(return_address);

                    let target_label = target
                        .labels()
                        .iter()
                        .next()
                        .ok_or_else(|| DotCFGError::MissingCallTargetLabel(target.node()))?
                        .to_owned();
                    let called_function = Rc::clone(
                        cfg.functions()
                            .get(&target_label)
                            .ok_or_else(|| DotCFGError::CallTargetIsNotFunction(target.node()))?,
                    );
                    let called_function_return = called_function.exit().clone();
                    returns.insert(called_function_return.id());
                    let call_info = CallInfo::new(target, return_address, called_function_return);
                    caller_info_map.insert(node.id(), call_info);
                }
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

        Ok(info)
    }

    fn create_blocks(
        cfg: &Cfg,
        leaders: &HashSet<Uuid>,
        leader_ids_to_blocks: &mut HashMap<Uuid, BasicBlock>,
        node_ids_to_leader_ids: &mut HashMap<Uuid, Uuid>,
    ) {
        let mut current_block = BasicBlock::new_empty();
        for node in cfg {
            // If node is leader, add the previous block to the ids_to_blocks map and begin the next block
            if leaders.contains(&node.id()) {
                if !current_block.is_empty() {
                    leader_ids_to_blocks.insert(current_block.id(), current_block);
                }
                current_block = BasicBlock::new_empty();
            }

            // Add node to current block
            current_block.push(Rc::clone(&node));

            // Add node to node_ids_to_leader_ids map
            node_ids_to_leader_ids.insert(node.id(), current_block.id());
        }
        // Last block is not followed by a leader,
        // so it still needs to be added to the leader_ids_to_blocks map
        if !current_block.is_empty() {
            leader_ids_to_blocks.insert(current_block.id(), current_block);
        }
    }

    fn write_cfg_as_dot(
        cfg: &Cfg,
        dot_cfg_file: &mut File,
        info: &DotCFGGenerationPassInfo,
        interprocedural_enabled: bool,
    ) -> Result<(), DotCFGError> {
        let leaders = &info.leaders;
        let caller_info_map = &info.caller_info_map;
        // Maps each function to the number of times it is called in the code
        // This is used for uniquely numbering call sites on a per-function basis
        let mut call_site_nums: HashMap<Uuid, u32> = HashMap::new();

        // Begin DOT graph and set node style
        writeln!(dot_cfg_file, "digraph cfg {{").map_err(|_| DotCFGError::FileWriteError)?;
        writeln!(dot_cfg_file, "\tnode [shape=none, fontname=\"Courier\"];")
            .map_err(|_| DotCFGError::FileWriteError)?;

        // Write all functions (including internal edges) as subgraphs
        for (func_num, (label, func)) in cfg.functions().iter().enumerate() {
            DotCFGGenerationPass::write_function_as_dot_subgraph(
                func,
                func_num,
                label,
                dot_cfg_file,
                info,
            )?;
        }

        // Write all remaining nodes
        for node in cfg {
            let node_id = node.id();
            let current_block = info.get_block_containing_node_with_id(&node_id)?;
            let current_block_id = current_block.id();

            if interprocedural_enabled {
                // If interprocedural_enabled is true and node is call:
                // - Write dashed edge from current block to call target block
                // - Write dashed edge from block containing return instruction to block containing return address
                if let Some(CallInfo {
                    target,
                    return_address,
                    return_inst,
                }) = caller_info_map.get(&node_id)
                {
                    let target_id = target.id();
                    let return_address_id = return_address.id();
                    let return_inst_id = return_inst.id();

                    // Increment or initialize call_site_num (starts at 1)
                    let call_site_num = if let Some(n) = call_site_nums.get_mut(&target_id) {
                        *n += 1;
                        *n
                    } else {
                        call_site_nums.insert(target_id, 1);
                        1
                    };

                    // Write dashed edge from current block to call target block
                    let target_block_id = info.get_block_containing_node_with_id(&target_id)?.id();
                    writeln!(
                        dot_cfg_file,
                        "\t\"{current_block_id}\":p -> \"{target_block_id}\":p[style=\"dashed\", label=\"call from site {call_site_num}\"];"
                    )
                    .map_err(|_| DotCFGError::FileWriteError)?;

                    // Write dashed edge from block containing return instruction to block containing return address
                    let return_inst_block_id = info
                        .get_block_containing_node_with_id(&return_inst_id)?
                        .id();
                    let return_address_block_id = info
                        .get_block_containing_node_with_id(&return_address_id)?
                        .id();
                    writeln!(
                        dot_cfg_file,
                        "\t\"{return_inst_block_id}\":p -> \"{return_address_block_id}\":p[style=\"dashed\", label=\"return after call site {call_site_num}\"];"
                    )
                    .map_err(|_| DotCFGError::FileWriteError)?;
                }
            }

            // If node is not leader, skip it
            // If node is in a function, we already printed its basic block, so skip it
            if !leaders.contains(&node_id) || node.functions().iter().next().is_some() {
                continue;
            }

            // If node is leader, write the block
            let dot_cfg_string = current_block.dot_str(None);
            writeln!(dot_cfg_file, "\t{dot_cfg_string}")
                .map_err(|_| DotCFGError::FileWriteError)?;
            DotCFGGenerationPass::write_outgoing_edges_as_dot(
                current_block,
                dot_cfg_file,
                info,
                "\t",
            )?;
        }

        // End DOT graph
        writeln!(dot_cfg_file, "}}").map_err(|_| DotCFGError::FileWriteError)?;

        Ok(())
    }

    fn write_outgoing_edges_as_dot(
        block: &BasicBlock,
        file: &mut File,
        info: &DotCFGGenerationPassInfo,
        indent_str: &str,
    ) -> Result<(), DotCFGError> {
        let terminator = block
            .terminator()
            .ok_or_else(|| DotCFGError::BlockWithLeaderMissingTerminator(block.clone()))?;
        terminator.nexts().iter().try_for_each(|succ| {
            if info.leaders.contains(&succ.id()) {
                writeln!(
                    file,
                    "{indent_str}\"{}\":p -> \"{}\":p;",
                    block.id(),
                    succ.id()
                )
                .map_err(|_| DotCFGError::FileWriteError)?;
                Ok(())
            } else {
                Err(DotCFGError::SuccessorOfTerminatorIsNotLeader(succ.node()))
            }
        })?;
        Ok(())
    }

    fn escape_dot_str(str: &str) -> String {
        let chars = str.chars();
        let escaped: String = chars
            .map(|c| {
                match c {
                    '\n' => String::from("\\n"),  // escape
                    '"' => String::from("\\\""),  // escape
                    '\\' => String::from("\\\\"), // escape
                    other => String::from(other), // preserve other chars
                }
            })
            .collect();
        escaped
    }

    fn write_function_as_dot_subgraph(
        func: &Function,
        func_num: usize,
        label: &LabelString,
        dot_cfg_file: &mut File,
        info: &DotCFGGenerationPassInfo,
    ) -> Result<(), DotCFGError> {
        let leaders = &info.leaders;
        let func_color = DotCFGGenerationPass::get_function_color(func_num);

        let label = DotCFGGenerationPass::escape_dot_str(label.as_str());
        writeln!(dot_cfg_file, "\tsubgraph \"{label}\" {{")
            .map_err(|_| DotCFGError::FileWriteError)?;
        writeln!(dot_cfg_file, "\t\tlabel=\"function \\\"{label}\\\"\"")
            .map_err(|_| DotCFGError::FileWriteError)?;
        writeln!(dot_cfg_file, "\t\tcluster=true;").map_err(|_| DotCFGError::FileWriteError)?;
        writeln!(dot_cfg_file, "\t\tfontsize=\"28\";").map_err(|_| DotCFGError::FileWriteError)?;
        for node in func.nodes().iter() {
            // If node is not leader, skip it
            if !leaders.contains(&node.id()) {
                continue;
            }

            // Identify the block and print it in DOT format
            let leader = node;
            let current_block = info.get_block_containing_node_with_id(&leader.id())?;
            let dot_cfg_string = current_block.dot_str(Some(func_color));
            writeln!(dot_cfg_file, "\t\t{dot_cfg_string};")
                .map_err(|_| DotCFGError::FileWriteError)?;

            DotCFGGenerationPass::write_outgoing_edges_as_dot(
                current_block,
                dot_cfg_file,
                info,
                "\t\t",
            )?;
        }
        writeln!(dot_cfg_file, "\t}}").map_err(|_| DotCFGError::FileWriteError)?;
        Ok(())
    }

    fn create_dot_cfg(
        cfg: &Cfg,
        config: &DotCFGGenerationPassConfiguration,
    ) -> Result<(), DotCFGError> {
        let dot_cfg_path = config.get_dot_cfg_path();
        let mut dot_cfg_file = File::create(dot_cfg_path)
            .map_err(|_| DotCFGError::FailedToCreateFile(dot_cfg_path.clone()))?;

        let mut info =
            DotCFGGenerationPass::scan_leaders_and_calls(cfg, config.interprocedural_enabled)?;
        DotCFGGenerationPass::create_blocks(
            cfg,
            &info.leaders,
            &mut info.leader_ids_to_blocks,
            &mut info.node_ids_to_leader_ids,
        );
        DotCFGGenerationPass::write_cfg_as_dot(
            cfg,
            &mut dot_cfg_file,
            &info,
            config.interprocedural_enabled,
        )
    }

    /// Get a color string for a function given its number.
    /// Just round-robin selects from a list of colors.
    ///
    /// Rust translation of the [corresponding LLVM function](https://github.com/llvm/llvm-project/blob/968d38d1d7d9de2d5717457876bba2663b36f620/llvm/lib/Support/GraphWriter.cpp#L91).
    fn get_function_color(function_number: usize) -> &'static str {
        const NUM_COLORS: usize = 20;
        const COLORS: [&str; NUM_COLORS] = [
            "aaaaaa7f", "aa00007f", "00aa007f", "aa55007f", "0055ff7f", "aa00aa7f", "00aaaa7f",
            "5555557f", "ff55557f", "55ff557f", "ffff557f", "5555ff7f", "ff55ff7f", "55ffff7f",
            "ffaaaa7f", "aaffaa7f", "ffffaa7f", "aaaaff7f", "ffaaff7f", "aaffff7f",
        ]; // Colors in RGBA
        #[allow(
            clippy::indexing_slicing,
            reason = "function number is non-negative and NUM_COLORS = COLORS.len()"
        )]
        COLORS[function_number % NUM_COLORS]
    }
}
impl LintPass<DotCFGGenerationPassConfiguration> for DotCFGGenerationPass {
    fn run(cfg: &Cfg, _errors: &mut DiagnosticManager, config: &DotCFGGenerationPassConfiguration) {
        if !config.get_enabled() {
            return;
        }

        // TODO proper error handling
        DotCFGGenerationPass::create_dot_cfg(cfg, config).expect("Failed to create DOT CFG");
    }
}
#[derive(Default)] // pass should be disabled by default
pub struct DotCFGGenerationPassConfiguration {
    /// Is the pass enabled?
    enabled: bool,
    /// The path of the file to write the CFG to
    dot_cfg_path: PathBuf,
    /// Should the graph terminate basic blocks after calls
    /// and include dashed edges for function calls/returns?
    interprocedural_enabled: bool,
}
impl PassConfiguration for DotCFGGenerationPassConfiguration {
    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
impl DotCFGGenerationPassConfiguration {
    #[must_use]
    pub fn get_dot_cfg_path(&self) -> &PathBuf {
        &self.dot_cfg_path
    }

    pub fn set_dot_cfg_path(&mut self, dot_cfg_path: PathBuf) {
        self.dot_cfg_path = dot_cfg_path;
    }

    #[must_use]
    pub fn get_interprocedural_enabled(&self) -> bool {
        self.interprocedural_enabled
    }

    pub fn set_interprocedural_enabled(&mut self, interprocedural_enabled: bool) {
        self.interprocedural_enabled = interprocedural_enabled;
    }
}

struct DotCFGGenerationPassInfo {
    /// Block leaders
    leaders: HashSet<Uuid>,
    /// Return addresses (cfg nodes that will be returned to after a call)
    return_addresses: HashSet<Uuid>,
    /// Returns (cfg node that returns to the caller)
    returns: HashSet<Uuid>,
    /// Maps each caller to its target id, return address id, and return instruction id
    caller_info_map: HashMap<Uuid, CallInfo>,
    /// Maps the id of each block leader to the corresponding basic block
    leader_ids_to_blocks: HashMap<Uuid, BasicBlock>,
    /// Maps each cfg node id to the id of the leader of the block that contains it
    node_ids_to_leader_ids: HashMap<Uuid, Uuid>,
}
impl DotCFGGenerationPassInfo {
    #[must_use]
    fn new() -> Self {
        DotCFGGenerationPassInfo {
            leaders: HashSet::new(),
            return_addresses: HashSet::new(),
            returns: HashSet::new(),
            caller_info_map: HashMap::new(),
            leader_ids_to_blocks: HashMap::new(),
            node_ids_to_leader_ids: HashMap::new(),
        }
    }

    fn get_block_containing_node_with_id(
        &self,
        node_id: &Uuid,
    ) -> Result<&BasicBlock, DotCFGError> {
        let leader_id = self
            .node_ids_to_leader_ids
            .get(node_id)
            .ok_or_else(|| DotCFGError::NodeIdNotInNodeIdsToLeaderIdsMap(*node_id))?;
        self.leader_ids_to_blocks
            .get(leader_id)
            .ok_or_else(|| DotCFGError::LeaderIdNotInLeaderIdsToBlocksMap(*leader_id))
    }
}

struct CallInfo {
    /// cfg node representing entry point of function
    target: Rc<CfgNode>,
    /// cfg node that will be returned to after the call
    return_address: Rc<CfgNode>,
    /// cfg node that returns to the caller
    return_inst: Rc<CfgNode>,
}
impl CallInfo {
    #[must_use]
    pub fn new(target: Rc<CfgNode>, return_address: Rc<CfgNode>, return_inst: Rc<CfgNode>) -> Self {
        CallInfo {
            target,
            return_address,
            return_inst,
        }
    }
}

#[derive(Debug)]
enum DotCFGError {
    BlockWithLeaderMissingTerminator(BasicBlock),
    CallHasMoreThanOneSuccessor(ParserNode),
    CallHasNoSuccessors(ParserNode),
    CallTargetIsNotFunction(ParserNode),
    CallTargetLabelNotInLabelNodeMap(With<LabelString>),
    FailedToCreateFile(PathBuf),
    FileWriteError,
    LeaderIdNotInLeaderIdsToBlocksMap(Uuid),
    MissingCallTargetLabel(ParserNode),
    NodeIdNotInNodeIdsToLeaderIdsMap(Uuid),
    SuccessorOfTerminatorIsNotLeader(ParserNode),
}

impl std::fmt::Display for DotCFGError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DotCFGError::BlockWithLeaderMissingTerminator(block) => {
                write!(
                    f,
                    "Block has leader, so it is nonempty, but has no terminator:\n{block}"
                )
            }
            DotCFGError::CallHasMoreThanOneSuccessor(node) => {
                write!(
                    f,
                    "Call has more than one successor, return address could not be determined: {node}"
                )
            }
            DotCFGError::CallHasNoSuccessors(node) => {
                write!(
                    f,
                    "Call has no successors, return address could not be determined: {node}"
                )
            }
            DotCFGError::CallTargetIsNotFunction(node) => {
                write!(f, "Call target is not a function: {node}")
            }
            DotCFGError::CallTargetLabelNotInLabelNodeMap(label) => {
                write!(
                    f,
                    "Call target label {label} is not in the label -> node map"
                )
            }
            DotCFGError::FailedToCreateFile(path) => {
                write!(
                    f,
                    "Failed to create file at \"{}\" for DOT CFG",
                    path.display()
                )
            }
            DotCFGError::FileWriteError => {
                write!(f, "Failed to write to file")
            }
            DotCFGError::LeaderIdNotInLeaderIdsToBlocksMap(leader_id) => {
                write!(
                    f,
                    "Leader id \"{leader_id}\" is not in the leader ids -> blocks map"
                )
            }
            DotCFGError::NodeIdNotInNodeIdsToLeaderIdsMap(node_id) => {
                write!(
                    f,
                    "Node id \"{node_id}\" is not in the node ids -> leader ids map"
                )
            }
            DotCFGError::MissingCallTargetLabel(node) => {
                write!(f, "Call target does not have a label: {node}")
            }
            DotCFGError::SuccessorOfTerminatorIsNotLeader(node) => {
                write!(
                    f,
                    "CFG node is the successor of a terminator but not a block leader: {node}"
                )
            }
        }
    }
}
