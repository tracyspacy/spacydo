use crate::errors::{VMError, VMResult};
use crate::pools::{InstructionsPool, StringPool};
use crate::storage::task_types::{Task, TaskVM};
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Debug)]
pub(crate) struct Storage {
    tasks_vm: Vec<Option<TaskVM>>,
    pub next_id: u32,
    alive: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct StorageData {
    tasks: Vec<Task>,
    next_id: u32,
}

/// storage is NOT thread-safe!
impl Storage {
    pub(crate) fn save(
        &mut self,
        string_pool: &StringPool,
        instructions_pool: &InstructionsPool,
    ) -> VMResult<()> {
        let f = File::create("tasks.bin").map_err(|_| VMError::StorageWriteError)?;
        let mut tasks = Vec::with_capacity(self.alive);
        for task_vm in self.tasks_vm.iter().flatten() {
            tasks.push(task_vm.to_task(string_pool, instructions_pool)?);
        }

        let data = StorageData {
            tasks,
            next_id: self.next_id,
        };
        //add context?
        bincode::serialize_into(f, &data).map_err(|_| VMError::StorageWriteError)?;
        Ok(())
    }

    pub(crate) fn resolve_task(
        &self,
        id: u32,
        string_pool: &StringPool,
        instructions_pool: &InstructionsPool,
    ) -> VMResult<Task> {
        let task_vm = &self.get(id)?;
        task_vm.to_task(string_pool, instructions_pool)
    }

    pub(crate) fn load(pool: &mut StringPool, op_pool: &mut InstructionsPool) -> VMResult<Self> {
        use std::io::ErrorKind;
        let data: StorageData = match File::open("tasks.bin") {
            Ok(file) => bincode::deserialize_from(file).map_err(|_| VMError::StorageReadError)?,
            Err(e) if e.kind() == ErrorKind::NotFound => StorageData {
                tasks: Vec::new(),
                next_id: 0,
            },
            Err(_) => return Err(VMError::StorageReadError),
        };

        let mut tasks_vm: Vec<Option<TaskVM>> = Vec::new();
        let mut alive = 0;

        for t in data.tasks {
            let task_vm = TaskVM::from_task(t, pool, op_pool)?;
            let id = task_vm.id as usize;

            if tasks_vm.len() <= id {
                tasks_vm.resize(id + 1, None);
            }
            //Task ids are restored from Task.id, not from vector index.
            tasks_vm[id] = Some(task_vm);
            alive += 1;
        }

        Ok(Self {
            tasks_vm,
            next_id: data.next_id,
            alive, //keep?
        })
    }

    pub(crate) fn add(&mut self, task: TaskVM) {
        let id = self.next_id;
        self.next_id += 1;

        if self.tasks_vm.len() <= id as usize {
            self.tasks_vm.resize(id as usize + 1, None);
        }

        self.tasks_vm[id as usize] = Some(task);
        self.alive += 1;
    }

    pub(crate) fn delete(&mut self, id: u32) -> VMResult<()> {
        let idx = id as usize;

        if let Some(task_vm) = self.tasks_vm.get_mut(idx)
            && task_vm.is_some()
        {
            *task_vm = None;
            self.alive -= 1;
            return Ok(());
        }
        Err(VMError::TaskNotFound(id))
    }

    pub(crate) fn get(&self, id: u32) -> VMResult<&TaskVM> {
        self.tasks_vm
            .get(id as usize)
            .and_then(|opt| opt.as_ref())
            .ok_or(VMError::TaskNotFound(id))
    }

    pub(crate) fn get_mut(&mut self, id: u32) -> VMResult<&mut TaskVM> {
        self.tasks_vm
            .get_mut(id as usize)
            .and_then(|opt| opt.as_mut())
            .ok_or(VMError::TaskNotFound(id))
    }

    pub(crate) fn len(&self) -> usize {
        self.alive
    }
}
