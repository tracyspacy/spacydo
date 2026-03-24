use crate::errors::{VMError, VMResult};
use crate::memory::LinearMemory;
use crate::pools::InstructionsPool;
use crate::values::{TAG_STRING, to_fat_pointer};
#[derive(Debug, Clone)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub state: TaskState,
    pub instructions: Vec<u8>,
}

#[derive(Debug, Clone)]
pub(crate) struct TaskVM {
    pub id: u32,
    pub title: (u32, u16),
    pub state: TaskState,
    pub instructions_ref: u32,
}
impl TaskVM {
    pub(crate) fn from_task(
        task: Task,
        memory: &mut LinearMemory,
        instructions_pool: &mut InstructionsPool,
    ) -> VMResult<Self> {
        let title = task.title.as_bytes();
        let (offset, size) =
            to_fat_pointer(memory.alloc(title.len() as u16, TAG_STRING as u8, title)?)?;
        let inst_ref = instructions_pool.intern_instructions(task.instructions);

        Ok(Self {
            id: task.id,
            title: (offset, size),
            state: task.state,
            instructions_ref: inst_ref,
        })
    }
    pub(crate) fn to_task(
        &self,
        memory: &LinearMemory,
        instructions_pool: &InstructionsPool,
    ) -> VMResult<Task> {
        let instructions = instructions_pool
            .get(self.instructions_ref as usize)?
            .to_vec();

        Ok(Task {
            id: self.id,
            title: memory
                .get_slice_as_str(self.title.0, self.title.1)?
                .to_string(),
            state: self.state,
            instructions,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TaskState {
    pub len: u8,
    pub state: u8,
}

impl TaskState {
    pub(crate) fn default(len: u8) -> VMResult<Self> {
        if len == 0 {
            return Err(VMError::MaxStatesError);
        }
        Ok(Self { len, state: 0 })
    }
    pub(crate) fn get_state(&self) -> u8 {
        self.state
    }

    pub(crate) fn set_state(&mut self, new_state: u32) -> VMResult<()> {
        if new_state >= self.len as u32 || new_state >= u8::MAX as u32 {
            return Err(VMError::InvalidStatus(new_state));
        }
        self.state = new_state as u8;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TaskField {
    Title = 0,
    State = 1,
    Instructions = 2,
}

impl TryFrom<u32> for TaskField {
    type Error = VMError;

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(TaskField::Title),
            1 => Ok(TaskField::State),
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
