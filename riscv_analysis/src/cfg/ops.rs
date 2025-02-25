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
    #[must_use]
    pub fn math_op(self) -> Option<MathOp> {
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
    #[must_use]
    pub fn scalar_op(self) -> Option<MathOp> {
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
    #[must_use]
    pub fn operate(&self, x: i32, y: i32) -> i32 {
        match self {
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
                let (x, y) = (i64::from(x), i64::from(y));
                ((x * y) >> 32) as i32
            }
            MathOp::Mulhu => {
                let (x, y) = (x as u64, y as u64);
                ((x * y) >> 32) as i32
            }
            // NOTE: The RISC-V spec doesn't trap for integer division by zero,
            // instead, RISC-V returns the following results for x / 0 (or x % 0):
            // - div   -1
            // - divu: 2^32 - 1
            // - rem:  x
            // - remu: x
            MathOp::Div => {
                match y {
                    0 => -1, // 2^32 - 1 as i32
                    _ => x / y,
                }
            }
            MathOp::Divu => match y {
                0 => -1,
                _ => (x as u32 / y as u32) as i32,
            },
            MathOp::Rem => match y {
                0 => x,
                _ => x % y,
            },
            MathOp::Remu => match y {
                0 => x,
                _ => (x as u32 % y as u32) as i32,
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::MathOp;

    #[allow(overflowing_literals)]
    #[test]
    fn bitwise() {
        assert_eq!(MathOp::And.operate(0xABCD_EF01, 0x1234_5678), 0x0204_4600);
        assert_eq!(MathOp::Or.operate(0xABCD_EF01, 0x1234_5678), 0xBBFD_FF79);
        assert_eq!(MathOp::Xor.operate(0xABCD_EF01, 0x1234_5678), 0xb9f9_b979);
    }

    #[test]
    fn div_zero() {
        assert_eq!(MathOp::Div.operate(12345678, 0), -1);
        assert_eq!(MathOp::Divu.operate(12345678, 0), -1);
        assert_eq!(MathOp::Rem.operate(12345678, 0), 12345678);
        assert_eq!(MathOp::Remu.operate(12345678, 0), 12345678);
    }
}
