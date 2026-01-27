#[derive(Debug, Clone)]
pub enum VMError {
    // Storage errors
    TaskNotFound(u32),
    StorageWriteError,
    StorageReadError,

    // Pool errors
    InvalidStringIndex(usize),
    InvalidInstructionsIndex(usize),

    //add context?
    // Stack errors
    StackUnderflow,
    WriteError,
    StackOverflow,

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
    MalformedIfThen {
        context: &'static str,
    },
}

pub type VMResult<T> = Result<T, VMError>;
