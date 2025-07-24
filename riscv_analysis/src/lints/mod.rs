mod checks;
pub use checks::*;

mod instruction_in_text;
pub use instruction_in_text::*;

mod overlapping_function;
pub use overlapping_function::*;

mod control_flow;
pub use control_flow::*;

mod ecall;
pub use ecall::*;

mod dead_value;
pub use dead_value::*;
