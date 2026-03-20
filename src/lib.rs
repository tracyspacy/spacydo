mod bytecode;
#[cfg(feature = "dot")]
mod dot;
mod errors;
mod inlinevec;
mod memory;
mod pools;
mod storage;
mod values;
mod vm;

pub use errors::{VMError, VMResult};
pub(crate) use memory::LinearMemory;
pub use storage::task_types::{Task, TaskField, TaskState};
pub use values::*;
pub use vm::VM;
pub mod prelude {
    pub use crate::{Task, TaskField, TaskState, VM, VMError, VMResult};
}
