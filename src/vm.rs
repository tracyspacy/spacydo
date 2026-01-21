use crate::VMError;
use crate::bytecode::{assembler::assemble, disassembler::disassemble, helpers::*, opcodes::*};
use crate::errors::VMResult;
use crate::inlinevec::InlineVec;
use crate::pools::{InstructionsPool, StringPool};
use crate::storage::{storage::Storage, task_types::*};
use crate::values::*;

const STACK_LIMIT: usize = 1_000;
const CONTROL_STACK_LIMIT: usize = 2;
const CALL_STACK_LIMIT: usize = 2;

type Stack = InlineVec<Value, STACK_LIMIT>;
type ControlStack = InlineVec<(u64, u64, u64), CONTROL_STACK_LIMIT>;
type CallStack = InlineVec<InstructionsFrame, CALL_STACK_LIMIT>;

#[derive(Debug, Clone, Copy, Default)]
struct InstructionsFrame {
    instructions_ref: u32,
    pc: usize,
}

#[derive(Debug)]
pub struct VM {
    stack: Stack,
    control_stack: ControlStack, //loops limit and index
    storage: Storage,
    pool: StringPool,
    instructions_pool: InstructionsPool,
    call_stack: CallStack,
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
        let mut call_stack = CallStack::default();
        let call_frame = InstructionsFrame {
            instructions_ref: program_ref,
            pc: 0,
        };
        call_stack.push(call_frame)?;

        Ok(Self {
            stack: Stack::default(),
            control_stack: ControlStack::default(),
            storage,
            pool,
            instructions_pool: vm_instructions,
            call_stack: call_stack,
        })
    }

    pub fn print_task(&self, id: u32) -> VMResult<Task> {
        self.storage
            .resolve_task(id, &self.pool, &self.instructions_pool)
    }
    // for test purposes, probably remove later
    pub fn disassemble_bytecode(&self) -> VMResult<String> {
        let bytecode_ref = self
            .call_stack
            .last()
            .ok_or(VMError::StackUnderflow)?
            .instructions_ref;
        let bytecode = self.instructions_pool.get(bytecode_ref as usize)?;
        disassemble(bytecode, &self.pool, &self.instructions_pool)
    }

    pub fn run(&mut self) -> VMResult<Stack> {
        //std::vec::Drain<'_, u64>
        // check this
        let mut instructions_ref = self
            .call_stack
            .last()
            .ok_or(VMError::StackUnderflow)?
            .instructions_ref;
        let mut pc = self.call_stack.last().ok_or(VMError::StackUnderflow)?.pc;
        let mut instructions = self.instructions_pool.get(instructions_ref as usize)?;
        while let Some(&op) = instructions.get(pc) {
            //println!("After {:?}: stack = {:?}", op, self.stack);
            pc += 1;
            //dbg!(&self.stack.len());
            match op {
                PUSH_U32 => {
                    let val = prepare_u32_from_be_checked(instructions, pc)?;
                    //push_stack(&mut self.stack, to_u32_val(val))?;
                    self.stack.push(to_u32_val(val))?;
                    pc += 4; //magic number
                }
                PUSH_STRING => {
                    let val = prepare_u32_from_be_checked(instructions, pc)?;
                    // push_stack(&mut self.stack, to_string_val(val))?;
                    self.stack.push(to_string_val(val))?;

                    pc += 4; //magic number
                }
                PUSH_CALLDATA => {
                    let val = prepare_u32_from_be_checked(instructions, pc)?;
                    //push_stack(&mut self.stack, to_calldata_val(val))?;
                    self.stack.push(to_calldata_val(val))?;
                    pc += 4; //magic number
                }

                PUSH_STATUS | PUSH_TASK_FIELD => {
                    let val = instructions[pc] as u32;
                    // push_stack(&mut self.stack, to_u32_val(val))?; //safer get fn
                    self.stack.push(to_u32_val(val))?;
                    pc += 1;
                }

                T_CREATE => {
                    //let instructions_ref = to_u32(pop_stack(&mut self.stack)?);
                    let instructions_ref = to_u32(self.stack.pop()?);
                    //let raw_status = to_u32(pop_stack(&mut self.stack)?);
                    let raw_status = to_u32(self.stack.pop()?);

                    let status = TaskStatus::try_from(raw_status)?;

                    // let title = to_u32(pop_stack(&mut self.stack)?);
                    let title = to_u32(self.stack.pop()?);
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
                    // let field_byte = to_u32(pop_stack(&mut self.stack)?);
                    let field_byte = to_u32(self.stack.pop()?);
                    let field = TaskField::try_from(field_byte)?;
                    //let id = to_u32(pop_stack(&mut self.stack)?);
                    let id = to_u32(self.stack.pop()?);
                    let task = &self.storage.get(id)?; // handle error
                    match field {
                        TaskField::Title => self.stack.push(to_string_val(task.title))?,
                        TaskField::Status => self.stack.push(to_u32_val(task.status as u32))?,

                        TaskField::Instructions => {
                            self.stack.push(to_calldata_val(task.instructions_ref))?
                        }
                    }
                }
                //maybe a bit confusing, that value to set to comes firts to be last to pop:
                // PUSH_STATUS 2 - push status value, may be PUSH_STRING for title
                // PUSH_U64 0 - push task id
                // PUSH_TASK_FIELD 1 - push task field! to change
                // T_SET_FIELD
                T_SET_FIELD => {
                    let field_byte = to_u32(self.stack.pop()?);
                    let field = TaskField::try_from(field_byte)?;
                    let id = to_u32(self.stack.pop()?);

                    let task = &mut self.storage.get_mut(id)?;
                    match field {
                        TaskField::Title => task.title = to_u32(self.stack.pop()?),
                        TaskField::Status => {
                            let v = to_u32(self.stack.pop()?);
                            task.status = TaskStatus::try_from(v)?;
                        }
                        TaskField::Instructions => {
                            task.instructions_ref = to_u32(self.stack.pop()?);
                        }
                    }
                    // push_stack(&mut self.stack, id)?;
                }
                T_DELETE => {
                    let id = to_u32(self.stack.pop()?);
                    self.storage.delete(id)?;
                }
                S_SAVE => self.storage.save(&self.pool, &self.instructions_pool)?,
                S_LEN => self.stack.push(to_u32_val(self.storage.len() as u32))?,

                DO => {
                    let index = to_u32(self.stack.pop()?);
                    let limit = to_u32(self.stack.pop()?);
                    self.control_stack
                        .push((pc as u64, index as u64, limit as u64))?;
                }
                LOOP => {
                    //dbg!("***L**O**O**P***{}", &self.stack);
                    let (addr, mut index, limit) = self.control_stack.pop()?;
                    if index + 1 < limit {
                        index += 1;
                        pc = addr as usize;
                        self.control_stack
                            .push((pc as u64, index as u64, limit as u64))?;
                    }
                }
                LOOP_INDEX => {
                    if let Some(last_val) = self.control_stack.last() {
                        let idx = last_val.1;
                        self.stack.push(to_u32_val(idx as u32))?;
                    } else {
                        return Err(VMError::StackUnderflow);
                    }
                } // check logic for nested loops

                //remove or repurpose rn it just validates if taskvm exists (relict opcode)
                S_LOAD => {
                    /*
                    let index = pop_stack(&mut self.stack)?;
                    if self.storage.exists(index) {
                        push_stack(&mut self.stack, index)?;
                    }
                    */
                }

                // forth style if .. then
                JUMP_IF_FALSE => {
                    if self.stack.pop()? == FALSE_VAL {
                        let val = prepare_u32_from_be_checked(instructions, pc)?;
                        dbg!(&val);
                        pc = val as usize;
                    } else {
                        // skiping jump destination which is u32 ie 4 bytes
                        pc += 4;
                    }
                }

                DUP => {
                    let v = self.stack.last().ok_or(VMError::StackUnderflow)?;
                    self.stack.push(v)?
                }

                SWAP => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    self.stack.push(b)?;
                    self.stack.push(a)?;
                }

                EQ => {
                    let right = self.stack.pop()?;
                    let left = self.stack.pop()?;
                    self.stack.push(value_eq(left, right)?)?;
                }

                NEQ => {
                    let right = self.stack.pop()?;
                    let left = self.stack.pop()?;
                    self.stack.push(value_neq(left, right)?)?;
                }

                LT => {
                    let right = self.stack.pop()?;
                    let left = self.stack.pop()?;
                    self.stack.push(value_cmp(left, right, true)?)?;
                }

                GT => {
                    let right = self.stack.pop()?;
                    let left = self.stack.pop()?;
                    self.stack.push(value_cmp(left, right, false)?)?;
                }

                //drops if true
                DROP_IF => {
                    let condition = self.stack.pop()?;
                    if condition == TRUE_VAL {
                        self.stack.pop()?;
                    }
                }

                CALL => {
                    let id = to_u32(self.stack.pop()?);

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
                        let _ = self.call_stack.push(frame);
                        instructions_ref = frame.instructions_ref;
                        instructions = self.instructions_pool.get(instructions_ref as usize)?;
                        pc = frame.pc;
                    }
                }
                END_CALL => {
                    if self.call_stack.len() > 1 {
                        self.call_stack.pop()?;
                        let frame = self.call_stack.last().ok_or(VMError::StackUnderflow)?;
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
    // unbox NaN-boxed values on stack.
    //
    pub fn unbox<'a>(&'a self, stack: Stack) -> VMResult<Vec<Return<'a>>> {
        stack
            .as_slice()
            .iter()
            .map(|&v| match get_value_type(v)? {
                ValueType::U32 => Ok(Return::U32(to_u32(v))),
                ValueType::Bool => Ok(Return::Bool(v == TRUE_VAL)),
                ValueType::String => Ok(Return::String(self.pool.resolve(to_u32(v) as usize)?)),
                ValueType::CallData => {
                    let bytecode = self.instructions_pool.get(to_u32(v) as usize)?;
                    Ok(Return::CallData(disassemble(
                        bytecode,
                        &self.pool,
                        &self.instructions_pool,
                    )?))
                }
            })
            .collect()
    }
}
