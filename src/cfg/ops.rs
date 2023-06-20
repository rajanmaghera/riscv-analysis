use crate::parser::Inst;

pub enum MathOp {
    Add,
    And,
    Or,
    Sll,
    Slt,
    Sltu,
    Sra,
    Srl,
    Sub,
    Xor,
    Mul,
    Mulh,
    Mulhsu,
    Mulhu,
    Div,
    Divu,
    Rem,
    Remu,
}

// impl Inst -> MathOp
impl Inst {
    pub fn math_op(&self) -> Option<MathOp> {
        match self {
            Inst::Add | Inst::Addi => Some(MathOp::Add),
            Inst::And | Inst::Andi => Some(MathOp::And),
            Inst::Or | Inst::Ori => Some(MathOp::Or),
            Inst::Sll | Inst::Slli => Some(MathOp::Sll),
            Inst::Slt | Inst::Slti => Some(MathOp::Slt),
            Inst::Sltu | Inst::Sltiu => Some(MathOp::Sltu),
            Inst::Sra | Inst::Srai => Some(MathOp::Sra),
            Inst::Srl | Inst::Srli => Some(MathOp::Srl),
            Inst::Sub => Some(MathOp::Sub),
            Inst::Xor | Inst::Xori => Some(MathOp::Xor),
            Inst::Mul => Some(MathOp::Mul),
            Inst::Mulh => Some(MathOp::Mulh),
            Inst::Mulhsu => Some(MathOp::Mulhsu),
            Inst::Mulhu => Some(MathOp::Mulhu),
            Inst::Div | Inst::Divw => Some(MathOp::Div),
            Inst::Divu => Some(MathOp::Divu),
            Inst::Rem | Inst::Remw => Some(MathOp::Rem),
            Inst::Remu | Inst::Remuw => Some(MathOp::Remu),
            _ => None,
        }
    }

    // To allow for scalar operations only, like stack manipulation
    pub fn scalar_op(&self) -> Option<MathOp> {
        match self {
            Inst::Add | Inst::Addi => Some(MathOp::Add),
            Inst::Sub => Some(MathOp::Sub),
            _ => None,
        }
    }
}

impl MathOp {
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::cast_sign_loss)]
    pub fn operate(&self, x: i32, y: i32) -> i32 {
        match self {
            // TODO check bounds
            MathOp::Add => x + y,
            MathOp::And => x & y,
            MathOp::Or => x | y,
            MathOp::Sll => x << y,
            MathOp::Slt => i32::from(x < y),
            MathOp::Sltu => i32::from((x as u32) < (y as u32)),
            MathOp::Sra => x >> y,
            MathOp::Srl => (x as u32 >> y) as i32,
            MathOp::Sub => x - y,
            MathOp::Xor => x ^ y,
            MathOp::Mul => x * y,
            MathOp::Mulh | MathOp::Mulhsu => {
                // TODO BUG FIX
                let (x, y) = (i64::from(x), i64::from(y));
                ((x * y) >> 32) as i32
            }
            MathOp::Mulhu => {
                let (x, y) = (x as u64, y as u64);
                ((x * y) >> 32) as i32
            }
            MathOp::Div => x / y,
            MathOp::Divu => (x as u32 / y as u32) as i32,
            MathOp::Rem => x % y,
            MathOp::Remu => (x as u32 % y as u32) as i32,
        }
    }
}
