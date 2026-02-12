use crate::bytecode::{assembler::assemble, disassembler::disassemble};
use crate::errors::{VMError, VMResult};
use crate::pools::{InstructionsPool, StringPool};

#[derive(Debug, Clone)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub status: TaskStatus,
    pub instructions: String,
}

#[derive(Debug, Clone)]
pub(crate) struct TaskVM {
    pub id: u32,
    pub title: u32,
    pub status: TaskStatus,
    pub instructions_ref: u32,
}
impl TaskVM {
    pub(crate) fn from_task(
        task: Task,
        strings: &mut StringPool,
        instructions_pool: &mut InstructionsPool,
    ) -> VMResult<Self> {
        let title_idx = strings.intern_string(task.title);

        let bytecode = assemble(&task.instructions, strings, instructions_pool)?;
        let inst_ref = instructions_pool.intern_instructions(bytecode);

        Ok(Self {
            id: task.id,
            title: title_idx,
            status: task.status,
            instructions_ref: inst_ref,
        })
    }
    pub(crate) fn to_task(
        &self,
        strings: &StringPool,
        instructions_pool: &InstructionsPool,
    ) -> VMResult<Task> {
        let code = instructions_pool.get(self.instructions_ref as usize)?;
        let instructions = disassemble(code, strings, instructions_pool)?;

        Ok(Task {
            id: self.id,
            title: strings.resolve(self.title as usize)?.to_string(),
            status: self.status,
            instructions,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    NotComplete = 0,
    InProgress = 1,
    Complete = 2,
}

impl TryFrom<u32> for TaskStatus {
    type Error = VMError;

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(TaskStatus::NotComplete),
            1 => Ok(TaskStatus::InProgress),
            2 => Ok(TaskStatus::Complete),
            _ => Err(VMError::InvalidStatus(v)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TaskField {
    Title = 0,
    Status = 1,
    Instructions = 2,
}

impl TryFrom<u32> for TaskField {
    type Error = VMError;

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(TaskField::Title),
            1 => Ok(TaskField::Status),
            2 => Ok(TaskField::Instructions),
            _ => Err(VMError::InvalidTaskField(v)),
        }
    }
}

#[derive(Debug)]
pub(crate) struct StorageData {
    pub(crate) tasks: Vec<Task>,
    pub(crate) next_id: u32,
}
