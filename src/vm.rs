use crate::VMError;
use crate::bytecode::{assembler::assemble, helpers::*, opcodes::*};
use crate::errors::VMResult;
use crate::pools::{InstructionsPool, StringPool};
use crate::storage::{storage::Storage, task_types::*};

const TRUE: u64 = 1; //move to constants.rs?
const FALSE: u64 = 0; //move to constants.rs?

#[derive(Debug, Clone, Copy)]
struct InstructionsFrame {
    instructions_ref: u64,
    pc: usize,
}

#[derive(Debug)]
pub struct VM {
    stack: Vec<u64>,
    control_stack: Vec<u64>, //loops limit and index
    storage: Storage,
    pool: StringPool,
    instructions_pool: InstructionsPool,
    call_stack: Vec<InstructionsFrame>, // limit to 2 ? main and task innenr instructions ?
}

/// VM is NOT thread-safe.
impl VM {
    pub fn init(instructions: &str) -> VMResult<Self> {
        if instructions.is_empty() {
            return Err(VMError::EmptyInstructions);
        }
        let mut pool = StringPool::default();
        let mut vm_instructions = InstructionsPool::default();
        let program_ops = assemble(instructions, &mut pool, &mut vm_instructions)?;
        let program_ref = vm_instructions.intern_instructions(program_ops);
        let storage = Storage::load(&mut pool, &mut vm_instructions)?;

        let call_frame = InstructionsFrame {
            instructions_ref: program_ref,
            pc: 0,
        };
        Ok(Self {
            stack: Vec::new(),
            control_stack: Vec::new(),
            storage,
            pool,
            instructions_pool: vm_instructions,
            call_stack: vec![call_frame],
        })
    }

    pub fn print_task(&self, v: u64) -> VMResult<Task> {
        self.storage
            .resolve_task(v, &self.pool, &self.instructions_pool)
    }

    pub fn run(&mut self) -> VMResult<Vec<u64>> {
        //std::vec::Drain<'_, u64>
        let frame_idx = self.call_stack.len() - 1;
        let mut instructions_ref = self.call_stack[frame_idx].instructions_ref;
        let mut pc = self.call_stack[frame_idx].pc;
        let mut instructions = self.instructions_pool.get(instructions_ref as usize)?;
        while let Some(&op) = instructions.get(pc) {
            // println!("After {:?}: stack = {:?}", op, self.stack);
            pc += 1;
            //dbg!(&self.stack.len());
            match op {
                PUSH_U8 => {
                    //push u8 on stack
                    push_stack(&mut self.stack, instructions[pc] as u64)?; //safer get fn
                    pc += 1;
                }
                PUSH_U64 | PUSH_STRING | PUSH_CALLDATA => {
                    //push u64 on stack
                    let val = prepare_u64_from_be_checked(instructions, pc)?;
                    push_stack(&mut self.stack, val)?;
                    pc += 8; //magic number
                }

                PUSH_STATUS | PUSH_TASK_FIELD => {
                    push_stack(&mut self.stack, instructions[pc] as u64)?; //safer get fn
                    pc += 1;
                }

                T_CREATE => {
                    let instructions_ref = pop_stack(&mut self.stack)?;
                    let raw_status = pop_stack(&mut self.stack)? as u8;
                    let status = TaskStatus::try_from(raw_status)?;
                    let title = pop_stack(&mut self.stack)?;
                    let id = self.storage.next_id;

                    let task = TaskVM {
                        id,
                        title,
                        status,
                        instructions_ref,
                    };
                    self.storage.add(task);
                    //return id?
                }

                T_GET_FIELD => {
                    let field_byte = pop_stack(&mut self.stack)? as u8;
                    let field = TaskField::try_from(field_byte)?;
                    let id = pop_stack(&mut self.stack)?;

                    let task = &self.storage.get(id)?; // handle error
                    match field {
                        TaskField::Title => push_stack(&mut self.stack, task.title)?,
                        TaskField::Status => push_stack(&mut self.stack, task.status as u64)?,
                        TaskField::Instructions => {
                            push_stack(&mut self.stack, task.instructions_ref)?
                        }
                    }
                }
                //maybe a bit confusing, that value to set to comes firts to be last to pop:
                // PUSH_STATUS 2 - push status value, may be PUSH_STRING for title
                // PUSH_U64 0 - push task id
                // PUSH_TASK_FIELD 1 - push task field! to change
                // T_SET_FIELD
                T_SET_FIELD => {
                    let field_byte = pop_stack(&mut self.stack)? as u8;
                    let field = TaskField::try_from(field_byte)?;
                    let id = pop_stack(&mut self.stack)?;

                    let task = &mut self.storage.get_mut(id)?;
                    match field {
                        TaskField::Title => task.title = pop_stack(&mut self.stack)?,
                        TaskField::Status => {
                            let v = pop_stack(&mut self.stack)? as u8;
                            task.status = TaskStatus::try_from(v)?;
                        }
                        TaskField::Instructions => {
                            task.instructions_ref = pop_stack(&mut self.stack)?;
                        }
                    }
                    // push_stack(&mut self.stack, id)?;
                }
                T_DELETE => {
                    let id = pop_stack(&mut self.stack)?;
                    self.storage.delete(id)?;
                }
                S_SAVE => self.storage.save(&self.pool, &self.instructions_pool)?,
                S_LEN => push_stack(&mut self.stack, self.storage.len() as u64)?,

                DO => {
                    let index = pop_stack(&mut self.stack)?;
                    let limit = pop_stack(&mut self.stack)?;
                    self.control_stack.push(pc as u64);
                    self.control_stack.push(index);
                    self.control_stack.push(limit);
                }
                LOOP => {
                    //dbg!("***L**O**O**P***{}", &self.stack);
                    if let (Some(limit), Some(mut index), Some(addr)) = (
                        self.control_stack.pop(),
                        self.control_stack.pop(),
                        self.control_stack.pop(),
                    ) && index + 1 < limit
                    {
                        index += 1;
                        pc = addr as usize;
                        self.control_stack.push(addr);
                        self.control_stack.push(index);
                        self.control_stack.push(limit);
                    }
                }
                LOOP_INDEX => push_stack(
                    &mut self.stack,
                    self.control_stack[self.control_stack.len() - 2],
                )?, // check logic for nested loops

                //remove or repurpose rn it just validates if taskvm exists (relict opcode)
                S_LOAD => {
                    let index = pop_stack(&mut self.stack)?;
                    if self.storage.exists(index) {
                        push_stack(&mut self.stack, index)?;
                    }
                }

                DUP => {
                    let v = *self.stack.last().ok_or(VMError::StackUnderflow)?;
                    push_stack(&mut self.stack, v)?;
                }

                EQ => {
                    let right = pop_stack(&mut self.stack)?;
                    let left = pop_stack(&mut self.stack)?;
                    push_stack(&mut self.stack, if left == right { TRUE } else { FALSE })?;
                }

                NEQ => {
                    let right = pop_stack(&mut self.stack)?;
                    let left = pop_stack(&mut self.stack)?;
                    push_stack(&mut self.stack, if left != right { TRUE } else { FALSE })?;
                }

                LT => {
                    let right = pop_stack(&mut self.stack)?;
                    let left = pop_stack(&mut self.stack)?;
                    push_stack(&mut self.stack, if left < right { TRUE } else { FALSE })?;
                }

                GT => {
                    let right = pop_stack(&mut self.stack)?;
                    let left = pop_stack(&mut self.stack)?;
                    push_stack(&mut self.stack, if left > right { TRUE } else { FALSE })?;
                }

                //drops if true
                DROP_IF => {
                    let condition = pop_stack(&mut self.stack)?;
                    if condition == TRUE {
                        pop_stack(&mut self.stack)?;
                    }
                }

                CALL => {
                    let id = pop_stack(&mut self.stack)?;

                    let task = &self.storage.get(id)?;

                    if let Some(caller_frame) = self.call_stack.last_mut() {
                        caller_frame.pc = pc;
                    }

                    let frame = InstructionsFrame {
                        instructions_ref: task.instructions_ref,
                        pc: 0,
                    };

                    if !self
                        .instructions_pool
                        .get(task.instructions_ref as usize)?
                        .is_empty()
                    {
                        self.call_stack.push(frame);
                        instructions_ref = frame.instructions_ref;
                        instructions = self.instructions_pool.get(instructions_ref as usize)?;
                        pc = frame.pc;
                    }
                }
                END_CALL => {
                    if self.call_stack.len() > 1 {
                        pop_stack(&mut self.call_stack)?;
                        let frame = last_stack(&self.call_stack)?;
                        instructions_ref = frame.instructions_ref;
                        instructions = self.instructions_pool.get(instructions_ref as usize)?;
                        pc = frame.pc;
                    }
                }

                _ => {}
            }
        }
        // self.stack.drain(..)
        Ok(std::mem::take(&mut self.stack))
    }
}
