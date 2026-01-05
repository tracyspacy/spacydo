/*
## Instruction Set

### STACK
PUSH_U8 <u8> - Push u8 value
PUSH_U64 <u64> - Push u64 value
PUSH_STRING <String> - Push String value
PUSH_STATUS <u8> - Push status value: 0-not complete 1 - in progress 2 -complete
PUSH_TASK_FIELD <u8> - Push task field value: 0 - title 1 -status 2- instructions
PUSH_CALLDATA <String with format> - Push task own instructions. Should follow format : [ ] - for empty instructions,
for non empty should end with END_CALL - PUSH_CALLDATA [ PUSH_U64 42 END_CALL ]
DUP - duplicates last() value on stack

### TASKs
T_CREATE - pop instructions reference (CallData) -> pop task status -> pop title -> form TaskVM and adds to Storage (without saving to persistant storage).
example:  PUSH_STRING TestTask PUSH_STATUS 0 PUSH_CALLDATA [ ] T_CREATE

T_GET_FIELD - pop task field byte -> pop task id -> push task field on stack
example: PUSH_U64 0 PUSH_TASK_FIELD 1 T_GET_FIELD

T_SET_FIELD - pop task field byte -> pop task id -> pop new field value to set -> set new task field value (not type safe!)
example: PUSH_STATUS 2 PUSH_U64 0 PUSH_TASK_FIELD 1 T_SET_FIELD

T_DELETE - pop task id -> delete task by id from storage
comment: there is [task0,task1,task2..] - PUSH_U64 1 T_DELETE -> tasks_vm in storage are Vec<Option<TaskVM>> =>
=> [task0,none,task2] while persistant storage stores only [task0,task1] and next id

### LOGICAL
EQ - pop right -> pop left -> push True if equal or False if not
NEQ - pop right -> pop left -> push True if not equal or False if equal
LT - pop right -> pop left -> push True if left is less than right
GT - pop right -> pop left -> push True if left is greater that right
DROP_IF - pop condition (TRUE/FALSE) -> pop from stack if TRUE

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
example: PUSH_U64 100 PUSH_U64 0 DO PUSH_U64 99 LOOP (same as (0..100).for_each(|_| v.push(99)); )
LOOP_INDEX - push current loop index to stack (! rn is not safe for nested loops )

CALL (executes tasks instructions) - pop task id -> Save the current pc into the current call frame -> pushes new instructions frame from task -> switches contexts to tasks instructions => vm.run is on new pc and matches different set of instructions (see vm.rs).
END_CALL (returns to main context) - pop current frame from CALL_STACK (task instructions) -> switches context back to main call frame (see vmr.rs).
example: PUSH_U64 0 DUP CALL (execute task 0 instructions)
*/

pub const PUSH_U8: u8 = 0x00; // +1 byte value
pub const PUSH_U64: u8 = 0x01; // +8 bytes value
pub const PUSH_STRING: u8 = 0x02; // +8 bytes value -stringpool index u64
pub const PUSH_STATUS: u8 = 0x03; // +1 byte value
pub const PUSH_CALLDATA: u8 = 0x04; // +8 bytes value -instructionpool index u64 ?
pub const PUSH_TASK_FIELD: u8 = 0x05; // +1 byte value
//
pub const T_CREATE: u8 = 0x06;
pub const T_GET_FIELD: u8 = 0x07;
pub const T_SET_FIELD: u8 = 0x08;
pub const T_DELETE: u8 = 0x09;
//
pub const S_SAVE: u8 = 0x0a;
pub const S_LOAD: u8 = 0x0b; // relict id, should be removed or repurpose -> Pop index -> push task - op checks if task by id exists
pub const S_LEN: u8 = 0x0c;
//
pub const DO: u8 = 0x0d; // (limit, index)
pub const LOOP: u8 = 0x0e;
pub const LOOP_INDEX: u8 = 0x0f;
pub const CALL: u8 = 0x10;
pub const END_CALL: u8 = 0x11;
pub const DROP_IF: u8 = 0x12; //if last is true drops value
//
pub const DUP: u8 = 0x13;
//
pub const EQ: u8 = 0x14; //compares 2 element on stack and pushes either true or false, format [value,reference value]
pub const NEQ: u8 = 0x15;
pub const LT: u8 = 0x16; // "less than" - [left,right] returns true only if left is less than right  
pub const GT: u8 = 0x17; // "greater than" - [left,right] returns true only if left is greater than right
