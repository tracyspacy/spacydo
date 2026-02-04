# spacydo - task engine virtual machine

### Why?

Imagine a simple todo app with just 4 CRUD primitives (create, read, update, delete), yet it has more features than prominent task management apps.

Spacydo makes this possible because tasks contain executable code. This means a minimal client with 4 primitive functions can have unlimited features - each task programs its own behavior.

##### Concept:Minimal task model + programmable behaviour.

While the instruction set is minimal, it already demonstrates powerful programmable functionality. Not only can basic task creation, deletion, and filtering be unique or customized, tasks can have their own executable instructions, enabling programmable behavior for each task:
- filtering tasks based on conditions
- tasks that modify the state of other tasks
- tasks that create other tasks
- recurring tasks
- auto-deleting tasks on status change
- etc

Creation of simple task without own executable instructions:
```
PUSH_STRING TASK1 PUSH_STATUS 0  PUSH_CALLDATA [ ] T_CREATE
```

Task that create a subtask when called:

```PUSH_STRING Parent PUSH_STATUS 0 \
   PUSH_CALLDATA [ PUSH_STRING Child PUSH_STATUS 0 PUSH_CALLDATA [ ] T_CREATE END_CALL ] \
   T_CREATE \
   PUSH_U32 0 CALL
```

Potentially, a todo client based on spacydo could be extensible through programming rather than constrained by a fixed feature set, allowing developers and users to define task behavior through adding or modifying instructions.

While it would benefit from a dedicated examples section, the best way to play with it right now is the tests:
```
cargo test
```

### Example:

```
let ops = "PUSH_U32 2 PUSH_U32 2 EQ IF PUSH_U32 3 THEN PUSH_U32 4 PUSH_STRING HELLO PUSH_U32 42 PUSH_U32 42 EQ PUSH_CALLDATA [ PUSH_U32 11 END_CALL ]";
// inits vm with instructions
let mut vm = VM::init(ops)?;

// executes instructions and returns raw stack with NaN-boxed values
let raw_stack = vm.run()?;


// unboxing raw values returns: 
// [U32(3), U32(4), String("HELLO"), Bool(true), CallData("PUSH_U32 11 END_CALL")]
let unboxed_stack = vm.unbox(&stack).collect::<VMResult<Vec<_>>>()?;

// returns 3u32
let _val:u32 = unboxed_stack[0].as_u32()?;

// returns "PUSH_U32 11 END_CALL"
let _calldata:&str = unboxed_stack[4].as_calldata()?;

// example with memory write

let ops_mem =
        "PUSH_U32 0 PUSH_U32 5 M_SLICE PUSH_U32 5 PUSH_U32 0 DO LOOP_INDEX LOOP_INDEX M_STORE LOOP";

let mut vm = VM::init(ops_mem)?;
let raw_stack = vm.run()?;

// get memslice offset and size (offset,size) = (0,5,)
let (offset, size) = vm.unbox(&raw_stack).next().unwrap()?.as_mem_slice()?;

// get values from memory : [0, 1, 2, 3, 4]
let memory_values: Vec<u32> = vm
        .return_memory(offset, size)
        .map(|r| r.unwrap().as_u32().unwrap())
        .collect();


// example with memory write containing Null values

let ops_mem_null_vals = "PUSH_U32 0 PUSH_U32 5 M_SLICE PUSH_U32 1 PUSH_U32 1 M_STORE PUSH_U32 3 PUSH_U32 3 M_STORE";

let mut vm = VM::init(ops_mem)?;
let raw_stack = vm.run()?;

// get memslice offset and size (offset,size) = (0,5,)
let (offset, size) = vm.unbox(&raw_stack).next().unwrap()?.as_mem_slice()?;

// returns  vec![Return::Null, Return::U32(1), Return::Null, Return::U32(3),]
let memory = vm.return_memory(offset, size).collect::<VMResult<Vec<_>>>()?; 

// returns [1,3]
let filtered: Vec<u32> = vm.return_memory(offset, size).filter_map(|r| match r.unwrap() {
            Return::U32(val) => Some(val),
            _ => None,
        })
        .collect();

```



### Recent updates:
- VM now uses NaN-boxing technique - see [values.rs](src/values.rs)
- All stack values are 64-bit (`u64`), but they encode 6 distinct types:
  - `Null`
  - Boolean (`TRUE_VAL`, `FALSE_VAL`)
  - `STRING_VAL`
  - `CALLDATA_VAL` 
  - `U32_VAL`
  - `MEM_SLICE_VAL` - (offset: 25 bits, size: 25 bits) 
- Added `InlineVec` - a vector-like backed by fixed size array data structure. Stack, control stack, call stack, jump stack now use InlineVec with specified limits
- VM has memory now (heap). Memory is simple Vec<Value>, grows dynamically, but technically length is restricted by mem_slice_val format: 25 bits payload for offset and size  
- new tests added



**Instruction set with description is here: [opcodes.rs](src/bytecode/opcodes.rs)**

**Instruction categories:**

**Stack Operations**: `PUSH_U32`, `PUSH_STRING`, `PUSH_CALLDATA`, `DUP`, `SWAP`, `DROP_IF`

**Task Operations**: `T_CREATE`, `T_GET_FIELD`, `T_SET_FIELD`, `T_DELETE`

**Storage Operations**: `S_SAVE`, `S_LEN`

**Memory Operations**: `M_SLICE`, `M_STORE`

**Control Flow**: `DO`, `LOOP`, `LOOP_INDEX`, `CALL`, `END_CALL`, `IF..THEN`

**Comparison**: `EQ`, `NEQ`, `LT`, `GT`

### Current Scope / Known Issues:
- storage and VM are not thread-safe
- instruction set is not stabilized (which instructions to add?)
- task model is not stabilized (which fields to add?)
- type checking during assembly to bytecode should be improved
