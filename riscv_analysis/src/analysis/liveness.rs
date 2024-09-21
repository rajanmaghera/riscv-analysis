use crate::{
    cfg::RegisterSet,
    parser::RegSets,
    passes::{CfgError, GenerationPass},
};
use std::collections::HashSet;

use super::CustomClonedSets;

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
                    .reduce(|acc, x| acc.union_c(&x))
                    .unwrap_or_default();
                node.set_live_out(live_out);

                if let Some((func, _)) = node.calls_to(cfg) {
                    // live_in[F_exit] = live_in[F_exit] U gen[F_exit] (live_out[n] AND u_def[F_exit])
                    // We take the union of the existing live_in to match multiple call sites
                    let func_exit_live_in = node
                        .live_out()
                        .intersection_c(&func.exit().u_def())
                        .union_c(&func.exit().live_in())
                        .union_c(&func.exit().node().gen_reg());

                    if func_exit_live_in != func.exit().live_in() {
                        changed = true;
                        func.exit().set_live_in(func_exit_live_in.clone());
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
                    let u_def = node
                        .prevs()
                        .clone()
                        .into_iter()
                        .filter(|x| visited.contains(x))
                        .map(|x| x.u_def())
                        .reduce(|acc, x| acc.intersection_c(&x))
                        .unwrap_or_default()
                        .difference_c(&RegSets::caller_saved())
                        .union_c(&(func.exit().u_def().intersection_c(&RegSets::ret())));

                    // live_in[n] = (live_in[F] & argument-registers) U (live_out[n] - kill[n])
                    // kill[n] = caller-saved
                    let live_in_temp = node.live_out().difference_c(&RegSets::caller_saved());
                    let live_in = func
                        .entry()
                        .live_out()
                        .intersection_c(&RegSets::argument())
                        .union_c(&live_in_temp);

                    if live_in != node.live_in() {
                        changed = true;
                        node.set_live_in(live_in);
                    }
                    if u_def != node.u_def() {
                        changed = true;
                        node.set_u_def(u_def);
                    }
                } else if node.node().is_ecall() {
                    let signature = node.known_ecall_signature();
                    let args = signature
                        .clone()
                        .map_or(RegisterSet::new(), |(args, _)| args);
                    let rets = signature
                        .clone()
                        .map_or(RegisterSet::new(), |(_, rets)| rets);

                    // u_def[n] = (AND u_def[s] for all s in prev[n]) - caller-saved | ecall_returns
                    let u_def = node
                        .prevs()
                        .clone()
                        .into_iter()
                        .filter(|x| visited.contains(x))
                        .map(|x| x.u_def())
                        .reduce(|acc, x| acc.intersection_c(&x))
                        .unwrap_or_default()
                        .difference_c(&RegSets::caller_saved())
                        .union_c(&rets);

                    // live_in[n] = (live_out[n] - caller-saved) U ecall_args U ecall_ins
                    // ecall_args = X17 (a7) in every case U inputs to the ecall if known by available value analysis, otherwise empty
                    let live_in = node
                        .live_out()
                        .difference_c(&RegSets::caller_saved())
                        .union_c(&RegSets::ecall_always_argument())
                        .union_c(&args);

                    if live_in != node.live_in() {
                        changed = true;
                        node.set_live_in(live_in);
                    }
                    if u_def != node.u_def() {
                        changed = true;
                        node.set_u_def(u_def);
                    }
                } else if node.node().is_return() {
                    // u_def[n] = AND u_def[s] for all s in prev[n]
                    let u_def = node
                        .prevs()
                        .clone()
                        .into_iter()
                        .filter(|x| visited.contains(x))
                        .map(|x| x.u_def())
                        .reduce(|acc, x| acc.intersection_c(&x))
                        .unwrap_or_default();

                    if u_def != node.u_def() {
                        changed = true;
                        node.set_u_def(u_def);
                    }
                } else if node.node().is_function_entry() {
                    // live_in[n] = gen[n] U (live_out[n] - kill[n])
                    let live_in = node
                        .live_out()
                        .difference_c(&node.node().kill_reg())
                        .union_c(&node.node().gen_reg());

                    // u_def[n] = live_in[n] AND argument-registers
                    let u_def = live_in.intersection_c(&RegSets::argument());

                    if live_in != node.live_in() {
                        changed = true;
                        node.set_live_in(live_in);
                    }
                    if u_def != node.u_def() {
                        changed = true;
                        node.set_u_def(u_def);
                    }
                } else {
                    // u_def[n] = AND u_def[s] for all s in prev[n] | kill[n]
                    let u_def = node
                        .prevs()
                        .clone()
                        .into_iter()
                        .filter(|x| visited.contains(x))
                        .map(|x| x.u_def())
                        .reduce(|acc, x| acc.intersection_c(&x))
                        .unwrap_or_default()
                        .union_c(&node.node().kill_reg());

                    // live_in[n] = gen[n] U (live_out[n] - kill[n])
                    let live_in = node
                        .live_out()
                        .difference_c(&node.node().kill_reg())
                        .union_c(&node.node().gen_reg());

                    if live_in != node.live_in() {
                        changed = true;
                        node.set_live_in(live_in);
                    }
                    if u_def != node.u_def() {
                        changed = true;
                        node.set_u_def(u_def);
                    }
                }
                visited.insert(node);
            }
        }
        Ok(())
    }
}
