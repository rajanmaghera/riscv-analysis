use crate::analysis::{LocValueMap, Location, Value};
use crate::cfg::MathOp;
use crate::parser::{RVInst, RVInstructionNode};

fn is_identity_on_right_side(op: &MathOp, y: i32) -> bool {
    match op {
        MathOp::And if y == i32::MIN => true,
        MathOp::Or if y == i32::MAX => true,
        MathOp::Add | MathOp::Sll | MathOp::Sra | MathOp::Srl | MathOp::Sub | MathOp::Xor
            if y == 0 =>
        {
            true
        }
        MathOp::Mul
        | MathOp::Mulh
        | MathOp::Mulhsu
        | MathOp::Mulhu
        | MathOp::Div
        | MathOp::Divu
            if y == 1 =>
        {
            true
        }
        _ => false,
    }
}

fn is_identity_on_left_side(op: &MathOp, x: i32) -> bool {
    match op {
        MathOp::Add | MathOp::Sll | MathOp::Sra | MathOp::Srl | MathOp::Xor if x == 0 => true,
        MathOp::Mul | MathOp::Mulhu | MathOp::Mulh | MathOp::Mulhsu if x == 1 => true,
        MathOp::And | MathOp::Or if x == i32::MAX => true,
        _ => false,
    }
}

fn operate_arithmetic_on_value_types(op: &MathOp, a: Value, b: Value) -> Value {
    match (a, b) {
        (Value::Undefined, _) | (_, Value::Undefined) => Value::Undefined,
        (Value::Unknown, _) | (_, Value::Unknown) => Value::Unknown,
        (Value::Const(x), Value::Const(y)) => Value::Const(op.operate(x, y)),
        (Value::InitialStackPointer(x), Value::Const(y)) => match op {
            MathOp::Add | MathOp::Sub => Value::InitialStackPointer(op.operate(x, y)),
            _ if is_identity_on_right_side(op, y) => Value::InitialStackPointer(x),
            _ => Value::Unknown,
        },
        (Value::Const(x), Value::InitialStackPointer(y)) => match op {
            MathOp::Add => Value::InitialStackPointer(op.operate(x, y)),
            _ if is_identity_on_left_side(op, x) => Value::InitialStackPointer(y),
            _ => Value::Unknown,
        },
        (Value::InitialStackPointer(x), Value::InitialStackPointer(y)) => match op {
            MathOp::Sub => Value::Const(op.operate(x, y)),
            MathOp::And | MathOp::Or | MathOp::Xor if x == y => Value::InitialStackPointer(x),
            MathOp::Div | MathOp::Divu if x == y => Value::Const(1),
            _ => Value::Unknown,
        },
        (Value::Initial(rx), Value::Initial(ry)) => match op {
            MathOp::Sub if rx == ry => Value::Const(0),
            MathOp::And | MathOp::Or | MathOp::Xor if rx == ry => Value::Initial(rx),
            MathOp::Div | MathOp::Divu if rx == ry => Value::Const(1),
            _ => Value::UnknownConst,
        },
        (Value::Initial(rx), Value::Const(y)) => match op {
            _ if is_identity_on_right_side(op, y) => Value::Initial(rx),
            _ => Value::UnknownConst,
        },
        (Value::Const(x), Value::Initial(ry)) => match op {
            _ if is_identity_on_left_side(op, x) => Value::Initial(ry),
            _ => Value::UnknownConst,
        },
        #[allow(clippy::match_same_arms)]
        (Value::InitialStackPointer(_), _) | (_, Value::InitialStackPointer(_)) => Value::Unknown,
        (Value::UnknownConst, _) | (_, Value::UnknownConst) => Value::UnknownConst,
    }
}

/// Symbolic execution function.
///
/// This execution function is independent of calling conventions; it only does what the
/// execution of the instruction actually does. This MUST be a monotonic function.
///
/// # Panics
///
/// Panics if there is a programmer error
pub fn symbolically_execute(map: &mut LocValueMap, instruction: &RVInstructionNode) {
    match instruction {
        RVInstructionNode::Arith(arith) => {
            let a = map.get(&Location::Register(*arith.rs1.get()));
            let b = map.get(&Location::Register(*arith.rs2.get()));
            let output =
                operate_arithmetic_on_value_types(&instruction.inst().math_op().unwrap(), a, b);
            map.insert(&Location::Register(*arith.rd.get()), output);
        }
        RVInstructionNode::IArith(iarith) => {
            let a = map.get(&Location::Register(*iarith.rs1.get()));
            let b = if let Some(imm) = iarith.imm.get().value() {
                Value::Const(imm)
            } else {
                Value::UnknownConst
            };
            let inst = instruction.inst();
            // TODO fix:
            let output = if inst == RVInst::Lui {
                operate_arithmetic_on_value_types(&MathOp::Sll, b, Value::Const(20))
            } else {
                // TODO remove panic
                operate_arithmetic_on_value_types(&instruction.inst().math_op().unwrap(), a, b)
            };

            map.insert(&Location::Register(*iarith.rd.get()), output);
        }
        RVInstructionNode::JumpLink(jump_link) => {
            map.insert(
                &Location::Register(*jump_link.rd.get()),
                Value::UnknownConst,
            );
        }
        RVInstructionNode::JumpLinkR(jump_link_r) => map.insert(
            &Location::Register(*jump_link_r.rd.get()),
            Value::UnknownConst,
        ),
        RVInstructionNode::Basic(_) | RVInstructionNode::Branch(_) => {
            // Do nothing
        }
        RVInstructionNode::Store(store) => {
            let address = map.get(&Location::Register(*store.rs1.get()));
            let value = map.get(&Location::Register(*store.rs2.get()));
            match (address, store.imm.value()) {
                (Value::Unknown, _) | (_, None) => {
                    map.join_memory_range_into_stack(i32::MIN..i32::MAX, value);
                }
                (Value::InitialStackPointer(x), Some(imm)) => {
                    map.insert(&Location::StackPointerOffset32(x + imm), value);
                }
                (Value::Undefined, _) => {
                    map.insert_memory_range_into_stack(i32::MIN..i32::MAX, Value::Undefined);
                }
                (Value::Const(_) | Value::Initial(_) | Value::UnknownConst, _) => {}
            }
        }
        RVInstructionNode::Load(load) => {
            let address = map.get(&Location::Register(*load.rs1.get()));
            let value = match (address, load.imm.value()) {
                (Value::Unknown, _) | (_, None) => Value::Unknown,
                (Value::InitialStackPointer(x), Some(imm)) => {
                    map.get(&Location::StackPointerOffset32(x + imm))
                }
                (Value::Initial(_) | Value::Const(_) | Value::UnknownConst, _) => {
                    Value::UnknownConst
                }
                (Value::Undefined, _) => Value::Undefined,
            };
            map.insert(&Location::Register(*load.rd.get()), value);
        }
        RVInstructionNode::LoadAddr(load_addr) => {
            map.insert(
                &Location::Register(*load_addr.rd.get()),
                Value::UnknownConst,
            );
        }
        RVInstructionNode::Csr(_) => {
            panic!()
        }
        RVInstructionNode::CsrI(_) => {
            panic!()
        }
    }
}
