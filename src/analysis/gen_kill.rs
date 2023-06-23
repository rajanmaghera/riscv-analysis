use std::collections::HashSet;

use crate::parser::{IArithType, Node, RegSets, Register};

use super::AvailableValue;

impl Node {
    // TODO BIG FIX for all different types of conditional/unconditional jumps
    // These defs are used to help start some functional analysis
    pub fn kill_reg_value(&self) -> HashSet<Register> {
        match self.clone() {
            Node::FuncEntry(_) => RegSets::caller_saved(),
            Node::JumpLink(x) => {
                let mut set = RegSets::caller_saved();
                set.insert(x.rd.data);
                set
            }
            _ => self.kill_reg(),
        }
    }

    pub fn kill_reg(&self) -> HashSet<Register> {
        let regs: HashSet<Register> = match self.clone() {
            Node::FuncEntry(_) => RegSets::callee_saved(),
            Node::JumpLink(_) if self.calls_to().is_some() => HashSet::new(),
            _ => self
                .stores_to()
                .map(|x| vec![x.data].into_iter().collect())
                .unwrap_or_default(),
        };
        regs.into_iter()
            .filter(|x| *x != Register::X0)
            .collect::<HashSet<_>>()
    }

    pub fn gen_reg(&self) -> HashSet<Register> {
        let regs: HashSet<Register> = match self {
            Node::JumpLinkR(_) if self.is_return() => RegSets::callee_saved(),
            _ => self.reads_from().into_iter().map(|x| x.data).collect(),
        };
        regs.into_iter()
            .filter(|x| *x != Register::X0)
            .collect::<HashSet<_>>()
    }

    pub fn gen_stack_value(&self) -> Option<(i32, AvailableValue)> {
        match self {
            Node::Store(expr) => {
                if expr.rs1 == Register::X2 {
                    Some((
                        expr.imm.data.0,
                        AvailableValue::RegisterWithScalar(expr.rs2.data, 0),
                    ))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    pub fn gen_reg_value(&self) -> Option<(Register, AvailableValue)> {
        match self {
            // function entry case is handled separately
            Node::LoadAddr(expr) => Some((
                expr.rd.data,
                AvailableValue::Address(expr.name.data.clone()),
            )),
            Node::Load(expr) => Some((
                expr.rd.data,
                AvailableValue::MemoryAtRegister(expr.rs1.data, expr.imm.data.0),
            )),
            Node::IArith(expr) => {
                if expr.rs1 == Register::X0 {
                    match expr.inst.data {
                        IArithType::Addi
                        | IArithType::Addiw
                        | IArithType::Xori
                        | IArithType::Ori => {
                            Some((expr.rd.data, AvailableValue::Constant(expr.imm.data.0)))
                        }
                        IArithType::Andi
                        | IArithType::Slli
                        | IArithType::Slliw
                        | IArithType::Srai
                        | IArithType::Sraiw
                        | IArithType::Srli
                        | IArithType::Srliw => Some((expr.rd.data, AvailableValue::Constant(0))),
                        IArithType::Slti => todo!(),
                        IArithType::Sltiu => todo!(),
                        IArithType::Auipc => todo!(),
                    }
                } else {
                    None
                }
            }
            Node::Arith(expr) => {
                if expr.rs1 == Register::X0 && expr.rs2 == Register::X0 {
                    Some((expr.rd.data, AvailableValue::Constant(0)))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
