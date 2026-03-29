# spacydo 

> "I thought of objects being like biological cells and/or individual computers on a network, only able to communicate with messages."
— Alan Kay

---

**SPACYDO is a stack-based virtual machine with image-based persistence. The core abstraction is a TASK: a persistent entity carrying its own executable bytecode alongside its state. Tasks can observe and modify fields of other tasks and their own.**

#### VM TYPES
All stack values are 64-bit (`u64`) Nan-Boxed values encoding 6 distinct types: 

| Type | Description |
|------|-------------|
| `TRUE_VAL`, `FALSE_VAL` | boolean true / false |
| `U32` | unsigned 32-bit integer |
| `CallData` | reference into InstructionsPool |
| `String` | [offset(25 bit) size(16 bit) tag(3 bits)] - fat pointer to a vector of u8 allocated to vm's linear memory  |
| `VecU32` | [offset (25 bit) size (16 bit) tag (3 bits) - fat pointer to a vector of u32 allocated to vm's linear memory |

See [values.rs](src/values.rs)

#### VM INSTRUCTIONS (dot - textual representation of the spacydo binary format )

|Type | Instructions |
|-----|---------------|
|Stack Operations| `PUSH_U32`, `PUSH_STRING`, `PUSH_CALLDATA`, `PUSH_STATE`, `PUSH_MAX_STATES`, `DUP`, `SWAP`, `DROP`|
|Task Operations| `T_CREATE`, `T_GET_FIELD`, `T_SET_FIELD`, `T_DELETE`|
|Storage Operations| `S_SAVE`, `S_LEN`|
|Memory Operations|`M_STI`, `M_ST`, `M_MUT`|
|Control Flow| `DO`, `LOOP`, `LOOP_INDEX`, `CALL`, `END_CALL`, `IF..THEN`|
|Logic operations| `EQ`, `NEQ`, `LT`, `GT`|
|Arithmetic operations| `MUL`, `MULI`|

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
let bytecode = VM::dot2bin(ops)?;
/* same without default features:
 let bytecode = vec![
        0x01, 0x00, 0x00, 0x00, 0x02, 0x01, 0x00, 0x00, 0x00, 0x02, 0x16, 0x1A, 0x00, 0x00, 0x00,
        0x15, 0x01, 0x00, 0x00, 0x00, 0x03, 0x01, 0x00, 0x00, 0x00, 0x04, 0x1B, 0x00, 0x05, 0x04,
        0x01, 0x48, 0x45, 0x4C, 0x4C, 0x4F, 0x01, 0x00, 0x00, 0x00, 0x2A, 0x01, 0x00, 0x00, 0x00,
        0x2A, 0x16, 0x04, 0x00, 0x06, 0x01, 0x00, 0x00, 0x00, 0x0B, 0x12,];
*/
// inits vm with instructions
let mut vm = VM::init(bytecode)?;

// executes instructions and returns raw stack with NaN-boxed values
let raw_stack = vm.run()?;


// unboxing raw values returns: 
// [U32(3), U32(4), String("HELLO"), Bool(true), CallData("PUSH_U32 11 END_CALL")]
let unboxed_stack = vm.unbox(&raw_stack).collect::<VMResult<Vec<_>>>()?;

// returns 3u32
let _val:u32 = unboxed_stack[0].as_u32()?;

// returns [1,0,0,0,11,18,] - Vec<u8>
let _calldata:&str = unboxed_stack[4].as_calldata()?;

// example with memory write
let ops_mem = "PUSH_U32 10 MULI 4 NEW_VEC_U32 PUSH_U32 10 PUSH_U32 0 DO LOOP_INDEX LOOP_INDEX M_MUTA LOOP";
let bytecode = VM::dot2bin(ops_mem).unwrap();
let mut vm = VM::init(bytecode).unwrap();
let raw_stack = vm.run().unwrap();

let _un = vm.unbox(&stack).next().unwrap().unwrap();
// get &[u32]
let vecu32 = _un.as_vec_u32().unwrap();

```

#### Current Scope / Known Issues:
- storage and VM are not thread-safe
- instruction set is not yet stabilized 
- task model is not yet stabilized
- nested loops limited to 2 levels
- nested calls limited to 2 frames
