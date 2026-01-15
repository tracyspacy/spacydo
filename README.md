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

### Recent updates:
- VM now uses NaN-boxing technique - see [values.rs](src/values.rs)
- All stack values are 64-bit (`u64`), but they encode 5 distinct types:
  - Boolean (`TRUE_VAL`, `FALSE_VAL`)
  - `STRING_VAL`
  - `CALLDATA_VAL` 
  - `U32_VAL` (replaces the previous `U8`/`U64`)  
- `PUSH_U8` and `PUSH_U64` replaced with single `PUSH_U32`
- Added `TypeMismatch` and `InvalidType` errors for tagged value validation
- new tests added



**Instruction set with description is here: [opcodes.rs](src/bytecode/opcodes.rs)**

**Instruction categories:**

**Stack Operations**: `PUSH_U32`, `PUSH_STRING`, `PUSH_CALLDATA`, `DUP`, `SWAP`, `DROP_IF`

**Task Operations**: `T_CREATE`, `T_GET_FIELD`, `T_SET_FIELD`, `T_DELETE`

**Storage Operations**: `S_SAVE`, `S_LEN`

**Control Flow**: `DO`, `LOOP`, `LOOP_INDEX`, `CALL`, `END_CALL`, `IF..THEN`

**Comparison**: `EQ`, `NEQ`, `LT`, `GT`

### Current Scope / Known Issues:
- storage and VM are not thread-safe
- nested loops are not safe
- execution frame has no specified limit
- instruction set is not stabilized (which instructions to add?)
- task model is not stabilized (which fields to add?)
- type checking during assembly to bytecode should be improved
