mod bytecode;
mod errors;
mod pools;
mod storage;
mod vm;

pub use errors::{VMError, VMResult};
pub use storage::task_types::{Task, TaskField, TaskStatus};
pub use vm::VM;

pub mod prelude {
    pub use crate::{Task, TaskField, TaskStatus, VM, VMError, VMResult};
}
