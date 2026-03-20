use crate::bytecode::{helpers::*, opcodes::*};
#[cfg(feature = "dot")]
use crate::dot::{bin2dot::bin2dot, dot2bin::dot2bin};
use crate::errors::{VMError, VMResult};
use crate::inlinevec::InlineVec;
use crate::memory::LinearMemory;
use crate::pools::InstructionsPool;
use crate::storage::{storage::Storage, task_types::*};
use crate::values::*;

const STACK_LIMIT: usize = 1_000;
const CONTROL_STACK_LIMIT: usize = 2;
const CALL_STACK_LIMIT: usize = 2;
//signaling byte
//const WO_PAYLOAD: u8 = 0;
const W_PAYLOAD: u8 = 1;

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
    instructions_pool: InstructionsPool,
    call_stack: CallStack,
    memory: LinearMemory,
}

/// VM is NOT thread-safe.
impl VM {
    pub fn init(instructions: Vec<u8>) -> VMResult<Self> {
        if instructions.is_empty() {
            return Err(VMError::EmptyInstructions);
        }
        let mut memory = LinearMemory::new();
        let mut vm_instructions = InstructionsPool::default();
        let program_ref = vm_instructions.intern_instructions(instructions);
        let storage = Storage::load(&mut memory, &mut vm_instructions)?;
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
            instructions_pool: vm_instructions,
            call_stack,
            memory,
        })
    }

    pub fn print_task(&self, id: u32) -> VMResult<Task> {
        let task_vm = self.storage.get(id).ok_or(VMError::TaskNotFound(id))?;
        task_vm.to_task(&self.memory, &self.instructions_pool)
    }

    // remove
    // short term patch for vec u32
    pub fn return_memory<'a>(
        &'a self,
        offset: u32,
        size: u32,
    ) -> impl Iterator<Item = VMResult<Return<'a>>> {
        let bytes = self.memory.get_slice_bytes(offset, size as u16);
        bytes.chunks_exact(4).map(|chunk| {
            let val = u32::from_be_bytes(chunk.try_into().unwrap());
            Ok(Return::U32(val))
        })
    }

    #[cfg(feature = "dot")]
    pub fn dot2bin(instructions: &str) -> VMResult<Vec<u8>> {
        dot2bin(instructions)
    }
    #[cfg(feature = "dot")]
    // for test purposes, probably remove later
    pub fn bin2dot(&self) -> VMResult<String> {
        let bytecode_ref = self
            .call_stack
            .last()
            .ok_or(VMError::StackUnderflow)?
            .instructions_ref;
        let bytecode = self.instructions_pool.get(bytecode_ref as usize)?;
        bin2dot(bytecode)
    }

    pub fn run(&mut self) -> VMResult<Stack> {
        let mut instructions_ref = self
            .call_stack
            .last()
            .ok_or(VMError::StackUnderflow)?
            .instructions_ref;
        let mut pc = self.call_stack.last().ok_or(VMError::StackUnderflow)?.pc;
        let mut instructions = self.instructions_pool.get(instructions_ref as usize)?;

        while pc < instructions.len() {
            let op = instructions[pc];
            // println!("After {:?}: stack = {:?}", op, self.stack);
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
                    /* let size = instructions[pc] as usize;
                    pc += 1;
                    let val = self.pool.intern_bytes(&instructions[pc..pc + size])?;
                    self.stack.push(to_string_val(val))?;
                    pc += size; */
                }
                PUSH_CALLDATA => {
                    let size = prepare_u16_from_be_checked(instructions, pc)? as usize;
                    pc += 2; // for u16
                    let calldata_vec = instructions[pc..pc + size].to_vec();
                    pc += size;
                    let val = self.instructions_pool.intern_instructions(calldata_vec);
                    self.stack.push(to_calldata_val(val))?;
                    instructions = self.instructions_pool.get(instructions_ref as usize)?
                }

                PUSH_STATE | PUSH_MAX_STATES | PUSH_TASK_FIELD => {
                    let val = instructions[pc] as u32;
                    self.stack.push(to_u32_val(val))?;
                    pc += 1;
                }

                T_CREATE => {
                    let instructions_ref = to_u32(self.stack.pop()?);
                    let max_states = to_u32(self.stack.pop()?);
                    // while should not allow on assembly carefully check, if somehow allows bigger than u8 ->
                    // -> it will trucate 3 msb and leave 1 full ie 255
                    let state = TaskState::default(max_states as u8)?;
                    let title = to_fat_pointer(self.stack.pop()?)?;
                    let id = self.storage.next_id;

                    let task = TaskVM {
                        id,
                        title,
                        state,
                        instructions_ref,
                    };
                    self.storage.add(task);
                    //return id?
                }

                T_GET_FIELD => {
                    let field_byte = to_u32(self.stack.pop()?);
                    let field = TaskField::try_from(field_byte)?;
                    let id = to_u32(self.stack.pop()?);
                    let task = &self.storage.get(id).ok_or(VMError::TaskNotFound(id))?;
                    match field {
                        TaskField::Title => self
                            .stack
                            .push(to_string_vec_val(task.title.0, task.title.1)?)?,
                        TaskField::State => {
                            self.stack.push(to_u32_val(task.state.get_state() as u32))?
                        }

                        TaskField::Instructions => {
                            self.stack.push(to_calldata_val(task.instructions_ref))?
                        }
                    }
                }
                //maybe a bit confusing, that value to set to comes firts to be last to pop:
                // PUSH_STATE 2 - push status value, may be PUSH_STRING for title
                // PUSH_U64 0 - push task id
                // PUSH_TASK_FIELD 1 - push task field! to change
                // T_SET_FIELD
                T_SET_FIELD => {
                    let field_byte = to_u32(self.stack.pop()?);
                    let field = TaskField::try_from(field_byte)?;
                    let id = to_u32(self.stack.pop()?);

                    let task = &mut self.storage.get_mut(id)?;
                    match field {
                        TaskField::Title => task.title = to_fat_pointer(self.stack.pop()?)?,
                        TaskField::State => {
                            let v = to_u32(self.stack.pop()?);
                            task.state.set_state(v)?
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
                S_SAVE => self.storage.save(&self.memory, &self.instructions_pool)?,
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
                        self.control_stack.push((pc as u64, index, limit))?;
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
                        //dbg!(&val);
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
                // accepts size in Bytes . same for string and vec u32
                //
                M_STA => {
                    let offset = prepare_u32_from_be_checked(instructions, pc)?;
                    pc += 4;
                    let size = prepare_u16_from_be_checked(instructions, pc)?;
                    pc += 2;
                    let tag = instructions[pc];
                    pc += 1;
                    let signaling_byte = instructions[pc];
                    pc += 1;
                    let mut payload: &[u8] = &[];
                    if signaling_byte == W_PAYLOAD {
                        payload = &instructions[pc..pc + size as usize];
                        pc += size as usize;
                    }
                    let val = self.memory.alloc_manual(offset, size, tag, payload)?;
                    self.stack.push(val)?;
                }
                //mutate at address
                //
                M_MUTA => {
                    //need to add check for byte values
                    let payload = to_u32(self.stack.pop()?);
                    let index = to_u32(self.stack.pop()?);
                    let value = self.stack.last().ok_or(VMError::StackUnderflow)?;
                    let (offset, size) = to_fat_pointer(value)?;
                    let tag = tag(value)?;
                    self.memory.mut_vec(offset, size, index, payload, tag)?
                }

                DROP => {
                    self.stack.pop()?;
                }

                CALL => {
                    let id = to_u32(self.stack.pop()?);

                    if let Some(task) = &self.storage.get(id) {
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
        //resetting pc of the main callframe to 0
        // this would allow us run initialized vm multiple times
        // while it add convenience and allow reuse of initialized vm and bytecode,
        // we need keep eye on it if it may cause some risks
        self.call_stack
            .last_mut()
            .ok_or(VMError::StackUnderflow)?
            .pc = 0;
        Ok(std::mem::take(&mut self.stack))
    }

    #[inline]
    fn unbox_value<'a>(&'a self, val: Value) -> VMResult<Return<'a>> {
        match get_value_type(val)? {
            ValueType::U32 => Ok(Return::U32(to_u32(val))),
            ValueType::Bool => Ok(Return::Bool(val == TRUE_VAL)),
            ValueType::String => {
                let (offset, size) = to_fat_pointer(val)?;
                let str = &self.memory.get_slice_as_str(offset, size)?;
                Ok(Return::String(str))
            }
            ValueType::CallData => {
                let bytecode = self.instructions_pool.get(to_u32(val) as usize)?;
                Ok(Return::CallData(bytecode))
            }
            /* ValueType::MemSlice => {
                let (offset, size) = to_mem_slice(val)?;
                Ok(Return::MemSlice(offset, size))
            } */
            ValueType::VecU32 => {
                //keep as it is for now
                let (offset, size) = to_fat_pointer(val)?;
                Ok(Return::VecU32(offset, size))
            }
            ValueType::Null => Ok(Return::Null),
        }
    }

    pub fn unbox<'a>(&'a self, values: &'a Stack) -> impl Iterator<Item = VMResult<Return<'a>>> {
        values.as_slice().iter().map(|&v| self.unbox_value(v))
    }
}
