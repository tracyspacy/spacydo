mod bytecode;
mod errors;
mod inlinevec;
mod pools;
mod storage;
mod values;
mod vm;

pub use errors::{VMError, VMResult};
pub use storage::task_types::{Task, TaskField, TaskState};
pub use values::*;
pub use vm::VM;
pub mod prelude {
    pub use crate::{Task, TaskField, TaskState, VM, VMError, VMResult};
}
