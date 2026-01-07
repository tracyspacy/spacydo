#[derive(Debug, Clone)]
pub enum VMError {
    // Storage errors
    TaskNotFound(u32),
    StorageWriteError,
    StorageReadError,

    // Pool errors
    InvalidStringIndex(usize),
    InvalidInstructionsIndex(usize),

    // Stack errors
    StackUnderflow,
    WriteError,
    StackOverflow,

    // Control flow errors to add later

    //CallStackOverflow,
    //CallStackUnderflow,

    //tagged values errors
    TypeMismatch,
    InvalidType,

    // Bytecode errors
    EmptyInstructions,
    InvalidStatus(u32),
    InvalidTaskField(u32),
    UnexpectedEOB,
    UnexpectedEOI {
        command: usize,
        context: &'static str,
    },
    InvalidUINT {
        command: usize,
        value: String,
    },
    MalformedCalldata {
        command: usize,
        context: &'static str,
    },
    UnknownOpcode {
        opcode: String,
    },
}

pub type VMResult<T> = Result<T, VMError>;
