use super::{Register, RegisterProperties};

impl RegisterProperties for Register {
    fn is_const_zero(&self) -> bool {
        self == &Register::X0
    }

    fn is_stack_pointer(&self) -> bool {
        self == &Register::X2
    }
}
