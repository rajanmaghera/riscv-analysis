use crate::{
    parser::{InstructionProperties, RegSets},
    passes::{CfgError, GenerationPass},
};
use std::collections::HashSet;

use super::HasGenKillInfo;

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
                // live_out[n] = U live_in[s] for all s in next[n]
                let live_out = node
                    .nexts()
                    .clone()
                    .into_iter()
                    .map(|x| x.live_in())
                    .reduce(|acc, x| acc | x)
                    .unwrap_or_default();
                changed |= node.set_live_out(live_out);

                if let Some((func, _)) = node.calls_to_from_cfg(cfg) {
                    // live_in[F_exit] = live_in[F_exit] U gen[F_exit] U live_out[n]
                    // We take the union of the existing live_in to match multiple call sites
                    let func_exit_live_in = (node.live_out()) | func.exit().live_in();
                    changed |= func.exit().set_live_in(func_exit_live_in);

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
                        - RegSets::caller_saved())
                        | (func.exit().u_def() & RegSets::ret());

                    // live_in[n] = (live_in[F_entry] & argument-registers) U (live_out[n] - kill[n])
                    // kill[n] = caller-saved
                    let live_in_temp = node.live_out() - node.kill_reg();
                    let live_in = (func.entry().live_out() & RegSets::argument())
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
                        - RegSets::caller_saved())
                        | rets;

                    // live_in[n] = (live_out[n] - caller-saved) U ecall_args U ecall_ins
                    // ecall_args = X17 (a7) in every case U inputs to the ecall if known by available value analysis, otherwise empty
                    let live_in = (node.live_out() - RegSets::caller_saved())
                        | RegSets::ecall_always_argument()
                        | args;
                    changed |= node.set_live_in(live_in);
                    changed |= node.set_u_def(u_def);
                } else if node.is_return() {
                    // live_in[n] = live_in[n] U gen[n]
                    let live_in = node.live_in() | node.gen_reg();
                    changed |= node.set_live_in(live_in);

                    // u_def[n] = AND u_def[s] for all s in prev[n]
                    let u_def = node
                        .prevs()
                        .clone()
                        .into_iter()
                        .filter(|x| visited.contains(x))
                        .map(|x| x.u_def())
                        .reduce(|acc, x| acc & x)
                        .unwrap_or_default();
                    changed |= node.set_u_def(u_def);
                } else if node.is_function_entry() {
                    // live_in[n] = gen[n] U (live_out[n] - kill[n])
                    let live_in = (node.live_out() - node.kill_reg()) | node.gen_reg();

                    // u_def[n] = live_in[n] AND argument-registers
                    let u_def = live_in & RegSets::argument();

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
