pub trait RegisterProperties {
    /// Returns true if the register is a zero register.
    ///
    /// A zero register is a register that always contains the value 0.
    /// Any operations to write to this register will either crash or
    /// be ignored.
    fn is_const_zero(&self) -> bool;

    /// Returns true if the register is a stack pointer.
    fn is_stack_pointer(&self) -> bool;
}
