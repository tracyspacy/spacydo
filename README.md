# spacydo 

> "I thought of objects being like biological cells and/or individual computers on a network, only able to communicate with messages."
— Alan Kay

---

**SPACYDO is a stack-based virtual machine with image-based persistence. The core abstraction is a TASK: a persistent entity carrying its own executable bytecode alongside its state. Tasks can observe and modify fields of other tasks and their own.**

#### VM TYPES
All stack values are 64-bit (`u64`) Nan-Boxed values encoding 6 distinct types: 

| Type | Description |
|------|-------------|
| `Null` | absence of value |
| `TRUE_VAL`, `FALSE_VAL` | boolean true / false |
| `U32` | unsigned 32-bit integer |
| `String` | reference into StringPool |
| `CallData` | reference into InstructionsPool |
| `MemSlice` | offset (25-bit) + size (25-bit) into scratch memory |

See [values.rs](src/values.rs)

#### VM INSTRUCTIONS

|Type | Instructions |
|-----|---------------|
|Stack Operations| `PUSH_U32`, `PUSH_STRING`, `PUSH_CALLDATA`, `PUSH_STATE`, `PUSH_MAX_STATES`, `DUP`, `SWAP`, `DROP`|
|Task Operations| `T_CREATE`, `T_GET_FIELD`, `T_SET_FIELD`, `T_DELETE`|
|Storage Operations| `S_SAVE`, `S_LEN`|
|Memory Operations|`M_SLICE`, `M_STORE`|
|Control Flow| `DO`, `LOOP`, `LOOP_INDEX`, `CALL`, `END_CALL`, `IF..THEN`|
|Logic operations| `EQ`, `NEQ`, `LT`, `GT`|

**Instruction set with description is here: [opcodes.rs](src/bytecode/opcodes.rs)**
VM instruction set is intentionally minimal and aiming to remain minimal in a future.
While bytecode instructions are expressive enough already, bytecode is verbose, so new instructions should be added.


#### What this enables

Because behavior lives inside the task rather than in application code, programs therefore consist of networks of interacting tasks whose behavior emerges from state transitions, and this behavior is updatable at runtime without application recompilation.

The same primitive naturally expresses:

**Workflows** — tasks that create, modify, chain, or destroy other tasks on state change. Behavior loaded from TOML at runtime, no recompilation required.

**State machines** — a task whose instructions encode its own transition rules. The traffic light example is a single task: instructions define red→green→yellow→red, no external FSM framework needed.

**Circuitry simulation** — the BCD decoder example implements a full Binary Coded Decimal decoder from Petzold's *Code*. Every NOT gate and AND gate is a persistent task with its own instructions that reads input task states and updates its own. 

#### Examples

| Example | What it demonstrates |
|---------|----------------------|
| [`examples/todo`](examples/todo) | Programmable tasks — chain, hide, self-destruct, conditional completion |
| [`examples/traffic_light`](examples/traffic_light) | Finite State machine as a single self-transitioning task |
| [`examples/bcd-decoder`](examples/bcd-decoder) | BCD decoder circuitry from "Code" by Charles Petzold book emulation with logic gates as Tasks |


**Various Usage examples**:

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

#### Current Scope / Known Issues:
- storage and VM are not thread-safe
- instruction set is not yet stabilized 
- task model is not yet stabilized
- nested loops limited to 2 levels
- nested calls limited to 2 frames
- type checking during assembly to bytecode should be improved
