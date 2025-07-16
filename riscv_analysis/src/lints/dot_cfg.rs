use uuid::Uuid;

use crate::{
    cfg::{BasicBlock, Cfg, CfgNode},
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
    fn scan_leaders_and_calls(cfg: &Cfg) -> Result<DotCFGGenerationPassInfo, DotCFGError> {
        let mut info = DotCFGGenerationPassInfo::new();
        let leaders = &mut info.leaders;
        let return_addresses = &mut info.return_addresses;
        let returns = &mut info.returns;
        let target_to_callers_map = &mut info.target_to_callers_map;
        let caller_info_map = &mut info.caller_info_map;
        let call_counts = &mut info.call_counts;
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

            // If node has call target:
            // - node is terminator
            // - target is leader
            // Note: call targets are not considered successors
            let call_target = node.calls_to();
            if let Some(label_string) = call_target {
                node_is_terminator = true;

                let call_target_instruction = cfg
                    .label_node_map
                    .get(label_string.as_str())
                    .ok_or_else(|| DotCFGError::CallTargetLabelNotInLabelNodeMap(label_string))?;
                leaders.insert(call_target_instruction.id());

                // Update target_to_callers_map
                if let Some(callers) = target_to_callers_map.get_mut(&call_target_instruction.id())
                {
                    callers.push(Rc::clone(&node));
                } else {
                    let callers: Vec<Rc<CfgNode>> = vec![Rc::clone(&node)];
                    target_to_callers_map.insert(call_target_instruction.id(), callers);
                }

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

        Ok(info)
    }

    fn identify_blocks_and_map_returns_to_leaders(
        cfg: &Cfg,
        info: &mut DotCFGGenerationPassInfo,
    ) -> Result<(), DotCFGError> {
        let leaders = &info.leaders;
        let return_addresses = &info.return_addresses;
        let returns = &info.returns;
        let ids_to_blocks = &mut info.ids_to_blocks;
        let return_address_to_leader_map = &mut info.return_address_to_leader_map;
        let return_inst_to_leader_map = &mut info.return_inst_to_leader_map;

        let mut current_block = BasicBlock::new_empty(); // need to initialize here to make compiler happy
        for node in cfg {
            // If node is leader, add the previous block to the ids_to_blocks map and begin the next block
            if leaders.contains(&node.id()) {
                if !current_block.is_empty() {
                    ids_to_blocks.insert(current_block.id(), current_block);
                }
                current_block = BasicBlock::new_empty();
            }

            // Add node to current block
            // It should not be in the current block already.
            if !current_block.push(Rc::clone(&node)) {
                return Err(DotCFGError::PushedDuplicateNodeIntoBlock(node.node()));
            }

            // Update return_address_to_leader_map if node is return address
            if return_addresses.contains(&node.id()) {
                let leader = current_block
                    .leader()
                    .ok_or_else(|| DotCFGError::MissingBlockLeader(current_block.clone()))?;
                return_address_to_leader_map.insert(node.id(), Rc::clone(&leader));
            }

            // Update return_inst_to_leader_map if node is return
            if returns.contains(&node.id()) {
                let leader = current_block
                    .leader()
                    .ok_or_else(|| DotCFGError::MissingBlockLeader(current_block.clone()))?;
                return_inst_to_leader_map.insert(node.id(), Rc::clone(&leader));
            }
        }
        // Add last block to ids_to_blocks map
        if !current_block.is_empty() {
            ids_to_blocks.insert(current_block.id(), current_block);
        }

        Ok(())
    }

    fn write_cfg_as_dot(
        cfg: &Cfg,
        dot_cfg_file: &mut File,
        info: &mut DotCFGGenerationPassInfo,
    ) -> Result<(), DotCFGError> {
        let leaders = &info.leaders;
        let ids_to_blocks = &info.ids_to_blocks;
        let caller_info_map = &info.caller_info_map;
        let call_counts = &mut info.call_counts;
        let return_inst_to_leader_map = &info.return_inst_to_leader_map;
        let return_address_to_leader_map = &info.return_address_to_leader_map;
        let function_to_color_map: HashMap<Uuid, &'static str> = cfg
            .functions()
            .values()
            .enumerate()
            .map(|(f_num, f)| (f.id(), DotCFGGenerationPass::get_function_color(f_num)))
            .collect();

        // Begin DOT graph and set node style
        writeln!(dot_cfg_file, "digraph cfg {{").map_err(|_| DotCFGError::FileWriteError)?;
        writeln!(dot_cfg_file, "\tnode [shape=record, fontname=\"Courier\"];")
            .map_err(|_| DotCFGError::FileWriteError)?;
        for node in cfg {
            // If node is not leader, skip it
            if !leaders.contains(&node.id()) {
                continue;
            }

            // If node is leader, identify the block and print it in DOT format
            let leader = node;
            let leader_id = leader.id();

            let current_block = ids_to_blocks
                .get(&leader_id)
                .ok_or_else(|| DotCFGError::LeaderNotInLeadersToBlocksMap(leader.node()))?;

            let terminator = current_block.terminator().ok_or_else(|| {
                DotCFGError::BlockWithLeaderMissingTerminator(current_block.clone())
            })?;
            let terminator_id = terminator.id();

            if let Some(function) = leader.functions().iter().next() {
                let fill_color = function_to_color_map.get(&function.id()).ok_or_else(|| {
                    DotCFGError::FunctionLeaderNotInFunctionToColorMap(leader.node())
                })?;
                let dot_cfg_string = current_block.dot_str(Some(fill_color));
                writeln!(dot_cfg_file, "\t{dot_cfg_string}")
                    .map_err(|_| DotCFGError::FileWriteError)?;
            } else {
                let dot_cfg_string = current_block.dot_str(None);
                writeln!(dot_cfg_file, "\t{dot_cfg_string}")
                    .map_err(|_| DotCFGError::FileWriteError)?;
            }

            // Print call and return as dashed edges in DOT format
            if let Some(CallInfo {
                target,
                return_address,
                return_inst,
            }) = caller_info_map.get(&terminator_id)
            {
                let call_count = call_counts
                    .get_mut(&target.id())
                    .ok_or_else(|| DotCFGError::CallTargetNotInCallCountMap(target.node()))?;
                *call_count += 1;

                writeln!(
                    dot_cfg_file,
                    "\t\"{}\" -> \"{}\"[style=\"dashed\", label=\"call from site {}\"];",
                    current_block.id(),
                    target.id(),
                    call_count,
                )
                .map_err(|_| DotCFGError::FileWriteError)?;

                let return_inst_block_leader = return_inst_to_leader_map
                    .get(&return_inst.id())
                    .ok_or_else(|| {
                        DotCFGError::ReturnInstNotInReturnInstToLeaderMap(return_inst.node())
                    })?;
                let return_address_block_leader = return_address_to_leader_map
                    .get(&return_address.id())
                    .ok_or_else(|| {
                        DotCFGError::ReturnAddressNotInReturnAddressToLeaderMap(return_inst.node())
                    })?;

                writeln!(
                    dot_cfg_file,
                    "\t\"{}\" -> \"{}\"[style=\"dashed\", label=\"return after call site {}\"];",
                    return_inst_block_leader.id(),
                    return_address_block_leader.id(),
                    call_count,
                )
                .map_err(|_| DotCFGError::FileWriteError)?;
            }

            // Print outgoing edges to all successor basic blocks in DOT format
            let succs = terminator.nexts();
            let succ_strings: Vec<String> = succs
                .iter()
                .map(|succ| {
                    if ids_to_blocks.contains_key(&succ.id()) {
                        Ok(succ.id().to_string())
                    } else {
                        Err(DotCFGError::SuccessorOfTerminatorIsNotLeader(succ.node()))
                    }
                })
                .collect::<Result<Vec<String>, DotCFGError>>()?;
            let succ_string = succ_strings.join("\" \"");

            if !succs.is_empty() {
                writeln!(
                    dot_cfg_file,
                    "\t\"{}\" -> {{ \"{}\" }};",
                    current_block.id(),
                    succ_string
                )
                .map_err(|_| DotCFGError::FileWriteError)?;
            }
        }
        // End DOT graph
        writeln!(dot_cfg_file, "}}").map_err(|_| DotCFGError::FileWriteError)?;

        Ok(())
    }

    fn create_dot_cfg(
        cfg: &Cfg,
        config: &DotCFGGenerationPassConfiguration,
    ) -> Result<(), DotCFGError> {
        let dot_cfg_path = config.get_dot_cfg_path();
        let mut dot_cfg_file = File::create(dot_cfg_path)
            .map_err(|_| DotCFGError::FailedToCreateFile(dot_cfg_path.clone()))?;

        let mut info = DotCFGGenerationPass::scan_leaders_and_calls(cfg)?;
        DotCFGGenerationPass::identify_blocks_and_map_returns_to_leaders(cfg, &mut info)?;
        DotCFGGenerationPass::write_cfg_as_dot(cfg, &mut dot_cfg_file, &mut info)
    }

    /// Get a color string for a function given its number.
    /// Just round-robin selects from a list of colors.
    ///
    /// Rust translation of the [corresponding LLVM function](https://github.com/llvm/llvm-project/blob/968d38d1d7d9de2d5717457876bba2663b36f620/llvm/lib/Support/GraphWriter.cpp#L91).
    fn get_function_color(function_number: usize) -> &'static str {
        const NUM_COLORS: usize = 20;
        const COLORS: [&str; NUM_COLORS] = [
            "aaaaaa", "aa0000", "00aa00", "aa5500", "0055ff", "aa00aa", "00aaaa", "555555",
            "ff5555", "55ff55", "ffff55", "5555ff", "ff55ff", "55ffff", "ffaaaa", "aaffaa",
            "ffffaa", "aaaaff", "ffaaff", "aaffff",
        ];
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
}

struct DotCFGGenerationPassInfo {
    /// Block leaders
    leaders: HashSet<Uuid>,
    /// Return addresses (cfg nodes that will be returned to after a call)
    return_addresses: HashSet<Uuid>,
    /// Returns (cfg node that returns to the caller)
    returns: HashSet<Uuid>,
    /// Maps each target (cfg node representing entry point of function) to the cfg nodes that call it
    target_to_callers_map: HashMap<Uuid, Vec<Rc<CfgNode>>>,
    /// Maps each caller to its target, return address, and return instruction
    caller_info_map: HashMap<Uuid, CallInfo>,
    /// Maps each function entry point to the number of times it is called in the code
    call_counts: HashMap<Uuid, u32>,
    /// Maps leader ids to basic blocks
    ids_to_blocks: HashMap<Uuid, BasicBlock>,
    /// Maps each return address to its block leader
    return_address_to_leader_map: HashMap<Uuid, Rc<CfgNode>>,
    /// Maps each return instruction to its block leader
    return_inst_to_leader_map: HashMap<Uuid, Rc<CfgNode>>,
}
impl DotCFGGenerationPassInfo {
    #[must_use]
    fn new() -> Self {
        DotCFGGenerationPassInfo {
            leaders: HashSet::new(),
            return_addresses: HashSet::new(),
            returns: HashSet::new(),
            target_to_callers_map: HashMap::new(),
            caller_info_map: HashMap::new(),
            call_counts: HashMap::new(),
            ids_to_blocks: HashMap::new(),
            return_address_to_leader_map: HashMap::new(),
            return_inst_to_leader_map: HashMap::new(),
        }
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
    CallTargetNotInCallCountMap(ParserNode),
    FailedToCreateFile(PathBuf),
    FileWriteError,
    FunctionLeaderNotInFunctionToColorMap(ParserNode),
    LeaderNotInLeadersToBlocksMap(ParserNode),
    MissingBlockLeader(BasicBlock),
    MissingCallTargetLabel(ParserNode),
    PushedDuplicateNodeIntoBlock(ParserNode),
    ReturnInstNotInReturnInstToLeaderMap(ParserNode),
    ReturnAddressNotInReturnAddressToLeaderMap(ParserNode),
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
            DotCFGError::CallTargetNotInCallCountMap(node) => {
                write!(
                    f,
                    "Call target is not in the call target -> call count map: {node}"
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
            DotCFGError::FunctionLeaderNotInFunctionToColorMap(node) => {
                write!(
                    f,
                    "Function leader is not in the function -> color map: {node}",
                )
            }
            DotCFGError::LeaderNotInLeadersToBlocksMap(node) => {
                write!(f, "Leader is not in the leaders -> blocks map: {node}")
            }
            DotCFGError::MissingBlockLeader(block) => {
                write!(f, "Block does not have leader:\n{block}")
            }
            DotCFGError::MissingCallTargetLabel(node) => {
                write!(f, "Call target does not have a label: {node}")
            }
            DotCFGError::PushedDuplicateNodeIntoBlock(node) => {
                write!(
                    f,
                    "Attempted to push a node into a block that already contains that node: {node}"
                )
            }
            DotCFGError::ReturnInstNotInReturnInstToLeaderMap(node) => {
                write!(
                    f,
                    "Return instruction is not in the return instruction -> return instruction block leader map: {node}"
                )
            }
            DotCFGError::ReturnAddressNotInReturnAddressToLeaderMap(node) => {
                write!(
                    f,
                    "Return address is not in the return address -> return address block leader map: {node}"
                )
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
