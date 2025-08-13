use crate::parser::RVInst;

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
impl RVInst {
    #[must_use]
    pub fn math_op(self) -> Option<MathOp> {
        match self {
            RVInst::Add | RVInst::Addi => Some(MathOp::Add),
            RVInst::And | RVInst::Andi => Some(MathOp::And),
            RVInst::Or | RVInst::Ori => Some(MathOp::Or),
            RVInst::Sll | RVInst::Slli => Some(MathOp::Sll),
            RVInst::Slt | RVInst::Slti => Some(MathOp::Slt),
            RVInst::Sltu | RVInst::Sltiu => Some(MathOp::Sltu),
            RVInst::Sra | RVInst::Srai => Some(MathOp::Sra),
            RVInst::Srl | RVInst::Srli => Some(MathOp::Srl),
            RVInst::Sub => Some(MathOp::Sub),
            RVInst::Xor | RVInst::Xori => Some(MathOp::Xor),
            RVInst::Mul => Some(MathOp::Mul),
            RVInst::Mulh => Some(MathOp::Mulh),
            RVInst::Mulhsu => Some(MathOp::Mulhsu),
            RVInst::Mulhu => Some(MathOp::Mulhu),
            RVInst::Div | RVInst::Divw => Some(MathOp::Div),
            RVInst::Divu => Some(MathOp::Divu),
            RVInst::Rem | RVInst::Remw => Some(MathOp::Rem),
            RVInst::Remu | RVInst::Remuw => Some(MathOp::Remu),
            _ => None,
        }
    }

    // To allow for scalar operations only, like stack manipulation
    #[must_use]
    pub fn scalar_op(self) -> Option<MathOp> {
        match self {
            RVInst::Add | RVInst::Addi => Some(MathOp::Add),
            RVInst::Sub => Some(MathOp::Sub),
            _ => None,
        }
    }
}

impl MathOp {
    #[allow(clippy::cast_sign_loss)]
    #[must_use]
    pub fn operate(&self, x: i32, y: i32) -> i32 {
        match self {
            MathOp::Add => x.wrapping_add(y),
            MathOp::And => x & y,
            MathOp::Or => x | y,
            MathOp::Sll => x << y,
            MathOp::Slt => i32::from(x < y),
            MathOp::Sltu => i32::from((x as u32) < (y as u32)),
            MathOp::Sra => x >> y,
            #[allow(clippy::cast_possible_wrap)]
            MathOp::Srl => (x as u32 >> y) as i32,
            MathOp::Sub => x - y,
            MathOp::Xor => x ^ y,
            MathOp::Mul => x.wrapping_mul(y),
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
                #[allow(clippy::cast_possible_wrap)]
                _ => (x as u32 / y as u32) as i32,
            },
            MathOp::Rem => match y {
                0 => x,
                _ => x % y,
            },
            MathOp::Remu => match y {
                0 => x,
                #[allow(clippy::cast_possible_wrap)]
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
