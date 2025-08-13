use super::{RVRegister, RegisterProperties};

impl RegisterProperties for RVRegister {
    fn is_const_zero(&self) -> bool {
        self == &RVRegister::X0
    }

    fn is_stack_pointer(&self) -> bool {
        self == &RVRegister::X2
    }

    fn is_return_addr(&self) -> bool {
        self == &RVRegister::X1
    }

    fn is_initial_register(&self) -> bool {
        matches!(
            self,
            RVRegister::X1
                | RVRegister::X2
                | RVRegister::X8
                | RVRegister::X9
                | RVRegister::X18
                | RVRegister::X19
                | RVRegister::X20
                | RVRegister::X21
                | RVRegister::X22
                | RVRegister::X23
                | RVRegister::X24
                | RVRegister::X25
                | RVRegister::X26
                | RVRegister::X27
        )
    }
}
