use std::rc::Rc;

use crate::analysis::{symbolically_execute, LocValueMap, Location, Value};
use crate::gen::FunctionMarkupPass;
use crate::parser::{HasRegisterSets, InstructionProperties, RVInst, RegisterProperties};
use crate::parser::{RVInstructionNode, RVRegister};
use crate::passes::{CfgError, GenerationPass};

fn will_maybe_not_halt(node: &RVInstructionNode, map: &LocValueMap) -> bool {
    if node.is_ecall() {
        !matches!(
            map.get(&Location::Register(RVRegister::ecall_type())),
            Value::Undefined | Value::Const(10 | 93)
        )
    } else if node.inst() == RVInst::Ebreak {
        false
    } else {
        true
    }
}

fn get_join_elements_for_function_entry_instructions() -> LocValueMap {
    let mut map = RVRegister::all()
        .into_iter()
        .map(|r| {
            if r.is_initial_register() {
                (r, Value::Initial(r))
            } else {
                (r, Value::UnknownConst)
            }
        })
        .collect::<LocValueMap>();
    map.join_memory_range_into_stack(i32::MIN..i32::MAX, Value::UnknownConst);
    debug_assert_eq!(
        map.get(&Location::Register(RVRegister::stack_pointer())),
        Value::InitialStackPointer(0)
    );
    map
}

pub struct AvailableValuePass;
impl GenerationPass for AvailableValuePass {
    fn run(cfg: &mut crate::cfg::Cfg) -> Result<(), Box<CfgError>> {
        let func_entry_map = get_join_elements_for_function_entry_instructions();
        let mut changed = true;
        while changed {
            changed = false;
            let mut nodes_to_promote = Vec::new();
            for node in cfg.iter_source() {
                let mut val_in = cfg
                    .get_prevs(node.as_ref())
                    .map(|x| x.real_val_out())
                    .reduce(|mut acc, x| {
                        acc.join(&x);
                        acc
                    })
                    .unwrap_or_default();

                if node.is_function_entry() || node.is_program_entry() {
                    val_in.join(&func_entry_map);
                }

                let mut val_out = val_in.clone();
                symbolically_execute(&mut val_out, &node.node());

                if node.calls_to().is_some() {
                    val_out.join_registers(
                        RVRegister::caller_saved_set()
                            .into_iter()
                            .map(|r| (r, Value::Unknown)),
                    );
                    match val_in.get(&Location::Register(RVRegister::stack_pointer())) {
                        Value::Unknown => {
                            val_out.join_memory_range_into_stack(i32::MIN..i32::MAX, Value::Unknown)
                        }
                        Value::InitialStackPointer(x) => {
                            val_out.join_memory_range_into_stack(i32::MIN..x - 4, Value::Unknown)
                        }
                        _ => {}
                    }
                }

                // If a maybe terminating instruction will not terminate
                if node.might_terminate() && will_maybe_not_halt(&node.node(), &val_out) {
                    nodes_to_promote.push(Rc::clone(node));
                }

                changed |= node.set_real_val_in(val_in);
                changed |= node.set_real_val_out(val_out);
            }
            for node in nodes_to_promote {
                changed |= cfg.promote_provisional_nexts_to_real(&node);
                FunctionMarkupPass::update_function_mappings(cfg)?;
            }
        }
        Ok(())
    }
}
