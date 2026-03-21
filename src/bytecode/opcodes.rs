/*
## Instruction Set

### STACK

PUSH_U32 <u32> - Push u32 value
PUSH_STATE <u32> - Push state value
PUSH_MAX_STATES <u32> - Push states len - ie amount of possible states
example: PUSH_MAX_STATES 5 - means possible states are [0,1,2,3,4]
PUSH_TASK_FIELD <u32> - Push task field value: 0 - title 1 -status 2- instructions
PUSH_CALLDATA <String with format> - Push task own instructions. Should follow format : [ ] - for empty instructions,
for non empty should end with END_CALL - PUSH_CALLDATA [ PUSH_U64 42 END_CALL ]
DUP - duplicates last() value on stack
SWAP - Exchange the top two stack items. [1,2] -> [2,1]

### TASKs
T_CREATE - pop instructions reference (CallData) -> pop task status -> pop title -> form TaskVM and adds to Storage (without saving to persistant storage).
example:  PUSH_STRING TestTask PUSH_STATUS 0 PUSH_CALLDATA [ ] T_CREATE

T_GET_FIELD - pop task field byte -> pop task id -> push task field on stack
example: PUSH_U32 0 PUSH_TASK_FIELD 1 T_GET_FIELD

T_SET_FIELD - pop task field byte -> pop task id -> pop new field value to set -> set new task field value (not type safe!)
example: PUSH_STATUS 2 PUSH_U32 0 PUSH_TASK_FIELD 1 T_SET_FIELD

T_DELETE - pop task id -> delete task by id from storage
comment: there is [task0,task1,task2..] - PUSH_U32 1 T_DELETE -> tasks_vm in storage are Vec<Option<TaskVM>> =>
=> [task0,none,task2] while persistant storage stores only [task0,task1] and next id

### LOGICAL
EQ - pop right -> pop left -> push True if equal or False if not
NEQ - pop right -> pop left -> push True if not equal or False if equal
LT - pop right -> pop left -> push True if left is less than right
GT - pop right -> pop left -> push True if left is greater that right
DROP - pop from stack

### STORAGE
S_SAVE - Save all tasks to disk
S_LEN - Push total task count to stack
S_LOAD - (!ignore relic code) now verifies if task exist

### CONTROL
DO-LOOP
DO -(declare loop)  pop index -> pop limit -> push current pc to CONTROL STACK -> push index to CONTROL STACK -> push limit to CONTROL STACK
Control stack after: [ loop_start_pc, index, limit ]
LOOP - (execute loop) -> pop limit -> pop index -> pop pc -> compare next index < limit (loop not over) -> increment index -> jump to old pc -> push pc to CONTROL STACK -> push index to CONTROL STACK -> push limit to CONTROL STACK
(if loop is over just continue with next pc, not jump back)
example: PUSH_U64 100 PUSH_U32 0 DO PUSH_U64 99 LOOP (same as (0..100).for_each(|_| v.push(99)); )
LOOP_INDEX - push current loop index to stack (! rn is not safe for nested loops )
JUMP_IF_FALSE - forth like if..then - pop true or false value, if false, jump to jump destinnation (after then)

CALL (executes tasks instructions) - pop task id -> Save the current pc into the current call frame -> pushes new instructions frame from task -> switches contexts to tasks instructions => vm.run is on new pc and matches different set of instructions (see vm.rs).
END_CALL (returns to main context) - pop current frame from CALL_STACK (task instructions) -> switches context back to main call frame (see vmr.rs).
example: PUSH_U32 0 DUP CALL (execute task 0 instructions)

### MEMORY
VM Memory is linear bytes array Vec<u8>, grows dynamically , but technically length is restricted by offset 25 bits -> max addressable offset is 2^25-1= 33_554_431
each memory slice is represented by nan-boxed vec [offset:25 bits][size:16bits][tag:3]
where *offset* is starting byte address in linear memory, *size* is number of bytes , *tag* - vector element type (u32,byte)
M_STA - Memory Store At - allocates bytes to memory and returns nan-boxed vec_val on stack [offset:25 bits][size:16bits][tag:3]
M_MUT -Memory Mutate At - mutates memory at address in existing memory slice - takes 3 parameters:  vec, index, value  -> pop value, pop index, peek vec -> writes value at index to memory slice.
Important, vec[offset:25 bits][size:16bits][tag:3] remains on stack!

*/

// TODO: since new opcodes will be added , need to reserve some space in categories.
pub const PUSH_U32: u8 = 0x01;
// vacant opcode
// pub const PUSH_STRING: u8 = 0x02;
pub const PUSH_STATE: u8 = 0x03;
pub const PUSH_CALLDATA: u8 = 0x04;
pub const PUSH_TASK_FIELD: u8 = 0x05;
pub const PUSH_MAX_STATES: u8 = 0x06;
//
pub const T_CREATE: u8 = 0x07;
pub const T_GET_FIELD: u8 = 0x08;
pub const T_SET_FIELD: u8 = 0x09;
pub const T_DELETE: u8 = 0x0a;
//
pub const S_SAVE: u8 = 0x0b;
pub const S_LOAD: u8 = 0x0c; // relict id, should be removed or repurpose -> Pop index -> push task - op checks if task by id exists
pub const S_LEN: u8 = 0x0d;
//
pub const DO: u8 = 0x0e; // (limit, index)
pub const LOOP: u8 = 0x0f;
pub const LOOP_INDEX: u8 = 0x10;
pub const CALL: u8 = 0x11;
pub const END_CALL: u8 = 0x12;
pub const DROP: u8 = 0x13; //
pub const DUP: u8 = 0x14;
pub const SWAP: u8 = 0x15;
//
pub const EQ: u8 = 0x16; //compares 2 element on stack and pushes either true or false, format [value,reference value]
pub const NEQ: u8 = 0x17;
pub const LT: u8 = 0x18; // "less than" - [left,right] returns true only if left is less than right
pub const GT: u8 = 0x19; // "greater than" - [left,right] returns true only if left is greater than right

pub const JUMP_IF_FALSE: u8 = 0x1a; //

pub const M_STA: u8 = 0x1b;
pub const M_MUTA: u8 = 0x1c;
