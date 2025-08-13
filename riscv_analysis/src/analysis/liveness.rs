use super::HasGenKillInfo;
use crate::cfg::{Cfg, RegisterSet};
use crate::parser::LabelStringToken;
use crate::{
    parser::{HasRegisterSets, InstructionProperties, RVRegister},
    passes::{CfgError, GenerationPass},
};

pub struct LivenessPass;
impl GenerationPass for LivenessPass {
    #[allow(clippy::too_many_lines)]
    fn run(cfg: &mut Cfg) -> Result<(), Box<CfgError>> {
        let mut changed = true;
        while changed {
            changed = false;
            for node in cfg.iter_source().rev() {
                if node.is_return() {
                    // live_out[F_exit] = live_out[F_exit] & return-registers
                    let live_out = if let Some(func) = node.functions().iter().next() {
                        (node.live_out()
                            | cfg
                                .get_call_sites(func)
                                .map(|x| x.live_out())
                                .reduce(|acc, x| acc | x)
                                .unwrap_or_default())
                            & RVRegister::return_set()
                    } else {
                        node.live_out() & RVRegister::return_set()
                    };
                    changed |= node.set_live_out(live_out);
                } else {
                    // live_out[n] = U live_in[s] for all s in next[n]
                    let live_out = cfg
                        .get_nexts(node.as_ref())
                        .map(|x| x.live_in())
                        .reduce(|acc, x| acc | x)
                        .unwrap_or_default();
                    changed |= node.set_live_out(live_out);
                }

                if let Some((func, _)) = node.calls_to_from_cfg(cfg) {
                    for exit in func.exits().iter() {
                        // live_out[F_exit] = live_out[F_exit] U (live_out[n] & return-registers)
                        let func_exit_live_out =
                            (node.live_out() & RVRegister::return_set()) | exit.live_out();
                        changed |= exit.set_live_out(func_exit_live_out);
                    }

                    // live_in[n] = (live_in[F_entry] & argument-registers) U (live_out[n] - caller-saved registers)
                    let live_in = (func.entry().live_in() & RVRegister::argument_set())
                        | (node.live_out() - RVRegister::caller_saved_set() - node.kill_reg())
                        | node.gen_reg();
                    changed |= node.set_live_in(live_in);
                } else if let Some((ext_func, _)) =
                    node.calls_to_some_external_function_from_cfg(cfg)
                {
                    // live_in[n] = (live_in[F_entry] & argument-registers) U (live_out[n] - caller-saved registers)
                    let live_in = (ext_func.arguments() & RVRegister::argument_set())
                        | (node.live_out() - RVRegister::caller_saved_set() - node.kill_reg())
                        | node.gen_reg();
                    changed |= node.set_live_in(live_in);
                } else if node.is_ecall() {
                    let (args, _) = node.known_ecall_signature().unwrap_or_default();

                    // live_in[n] = (live_out[n] - caller-saved) U ecall_args U ecall_ins
                    // ecall_args = X17 (a7) in every case U inputs to the ecall if known by available value analysis, otherwise empty
                    let live_in = (node.live_out() - RVRegister::caller_saved_set())
                        | RVRegister::ecall_always_argument_set()
                        | args;
                    changed |= node.set_live_in(live_in);
                } else {
                    // live_in[n] = gen[n] U (live_out[n] - kill[n])
                    let live_in = (node.live_out() - node.kill_reg()) | node.gen_reg();
                    changed |= node.set_live_in(live_in);
                }
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
        function_name_return: impl Iterator<Item = (LabelStringToken, RegisterSet, RegisterSet)>,
    ) {
        for (func_name, arguments, returns) in function_name_return {
            // For each call site that calls this function, union

            // For each given function, union the live_out set to the input list of return registers
            if let Some(func) = cfg.get_function(&func_name) {
                for exit_node in func.exits().iter() {
                    #[allow(unused_must_use)]
                    exit_node.set_live_out(exit_node.live_out() | returns);
                }
            }
        }
    }
}
