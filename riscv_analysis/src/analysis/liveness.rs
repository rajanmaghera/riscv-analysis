use super::HasGenKillInfo;
use crate::cfg::{Cfg, RegisterSet};
use crate::parser::LabelStringToken;
use crate::{
    parser::{HasRegisterSets, InstructionProperties, Register},
    passes::{CfgError, GenerationPass},
};
use std::collections::HashSet;

pub struct LivenessPass;
impl GenerationPass for LivenessPass {
    #[allow(clippy::too_many_lines)]
    fn run(cfg: &mut crate::cfg::Cfg) -> Result<(), Box<CfgError>> {
        let mut changed = true;
        #[allow(clippy::mutable_key_type)]
        let mut visited = HashSet::new();
        while changed {
            changed = false;
            for node in cfg.iter().rev() {
                if node.is_return() {
                    // live_out[F_exit] = live_out[F_exit] & return-registers
                    let live_out = node.live_out() & Register::return_set();
                    changed |= node.set_live_out(live_out);
                } else {
                    // live_out[n] = U live_in[s] for all s in next[n]
                    let live_out = node
                        .nexts()
                        .clone()
                        .into_iter()
                        .map(|x| x.live_in())
                        .reduce(|acc, x| acc | x)
                        .unwrap_or_default();
                    changed |= node.set_live_out(live_out);
                }

                if let Some((func, _)) = node.calls_to_from_cfg(cfg) {
                    for exit in func.exits().iter() {
                        // live_out[F_exit] = live_out[F_exit] U (live_out[n] & return-registers)
                        let func_exit_live_out =
                            (node.live_out() & Register::return_set()) | exit.live_out();
                        changed |= exit.set_live_out(func_exit_live_out);
                    }

                    // u_def[n] = (AND u_def[s] for all s in prev[n]) - kill[n] | (u_def[F_exit] AND return-registers)
                    // kill[n] = caller-saved
                    // NOTE: we use the UDEF_f because the udefs are all "candidates"
                    // for returns. If one happens to be the return, we can be sure
                    // that it is always defined. Otherwise, it is an error becuase
                    // we don't know if it is defined or not, so we could be reading
                    // a garbage value.
                    // TLDR: udef -> return values are a safeguard that the value
                    // has to come from the function.
                    let u_def = (node
                        .prevs()
                        .clone()
                        .into_iter()
                        .filter(|x| visited.contains(x))
                        .map(|x| x.u_def())
                        .reduce(|acc, x| acc & x)
                        .unwrap_or_default()
                        - Register::caller_saved_set())
                        | (func
                            .exits()
                            .iter()
                            .map(|x| x.u_def())
                            .reduce(|acc, x| acc & x)
                            .unwrap_or_default()
                            & Register::return_set());

                    // live_in[n] = (live_in[F_entry] & argument-registers) U (live_out[n] - caller-saved registers)
                    let live_in_temp = node.live_out() - node.kill_reg();
                    let live_in = (func.entry().live_out() & Register::argument_set())
                        | live_in_temp
                        | node.gen_reg();

                    changed |= node.set_live_in(live_in);
                    changed |= node.set_u_def(u_def);
                } else if node.is_ecall() {
                    let (args, rets) = node.known_ecall_signature().unwrap_or_default();

                    // u_def[n] = (AND u_def[s] for all s in prev[n]) - caller-saved | ecall_returns
                    let u_def = (node
                        .prevs()
                        .clone()
                        .into_iter()
                        .filter(|x| visited.contains(x))
                        .map(|x| x.u_def())
                        .reduce(|acc, x| acc & x)
                        .unwrap_or_default()
                        - Register::caller_saved_set())
                        | rets;

                    // live_in[n] = (live_out[n] - caller-saved) U ecall_args U ecall_ins
                    // ecall_args = X17 (a7) in every case U inputs to the ecall if known by available value analysis, otherwise empty
                    let live_in = (node.live_out() - Register::caller_saved_set())
                        | Register::ecall_always_argument_set()
                        | args;
                    changed |= node.set_live_in(live_in);
                    changed |= node.set_u_def(u_def);
                } else if node.is_function_entry() {
                    // live_in[n] = gen[n] U (live_out[n] - kill[n])
                    let live_in = (node.live_out() - node.kill_reg()) | node.gen_reg();

                    // u_def[n] = live_in[n] AND argument-registers
                    let u_def = live_in & Register::argument_set();

                    changed |= node.set_live_in(live_in);
                    changed |= node.set_u_def(u_def);
                } else {
                    // u_def[n] = AND u_def[s] for all s in prev[n] | kill[n]
                    let u_def = (node
                        .prevs()
                        .clone()
                        .into_iter()
                        .filter(|x| visited.contains(x))
                        .map(|x| x.u_def())
                        .reduce(|acc, x| acc & x)
                        .unwrap_or_default())
                        | node.kill_reg();

                    // live_in[n] = gen[n] U (live_out[n] - kill[n])
                    let live_in = (node.live_out() - node.kill_reg()) | node.gen_reg();

                    changed |= node.set_live_in(live_in);
                    changed |= node.set_u_def(u_def);
                }
                visited.insert(node);
            }
        }
        Ok(())
    }
}

impl LivenessPass {
    /// Given external information about a function's return registers, insert that information
    /// into the liveness passes.
    ///
    /// Some functions may be called externally, such as a "main" function. In order to ensure
    /// liveness analysis can correctly run, this function allows the user to insert extra information
    /// about the liveness of the return registers at the function's exit.
    ///
    /// Liveness analysis is a backwards analysis, that is it propagates information from the bottom
    /// to the top. Thus, it only needs return information and not argument registers.
    pub fn inject_return_registers_into_function(
        cfg: &mut Cfg,
        function_name_return: impl Iterator<Item = (LabelStringToken, RegisterSet)>,
    ) {
        // For each given function, union the live_out set to the input list of Registers
        for (func_name, registers) in function_name_return {
            if let Some(func) = cfg.functions().get(&func_name) {
                for exit_node in func.exits().iter() {
                    #[allow(unused_must_use)]
                    exit_node.set_live_out(exit_node.live_out() | registers);
                }
            }
        }
    }
}
