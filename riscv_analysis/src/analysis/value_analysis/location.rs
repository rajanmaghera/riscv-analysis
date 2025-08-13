use crate::parser::RVRegister;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Location {
    Register(RVRegister),
    StackPointerOffset32(i32),
}
