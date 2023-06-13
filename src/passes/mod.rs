// TODO compile-time register vec sets
// TODO switch to types that take up zero space

// TODO should I be storing this map inside the blocks?
// tests for DirectionalCFG

mod pass;
pub use pass::*;

mod checks;
pub use checks::*;

mod errors;
pub use errors::*;
