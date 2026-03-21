use serial_test::serial;
use spacydo::{Return, Task, TaskState, VM, VMError, VMResult};

use std::fs;

fn clear_storage() {
    let _ = fs::remove_file("tasks.bin");
}

#[test]
#[serial] //?
fn test_push_u32() {
    let bytecode = VM::dot2bin("PUSH_U32 1234567890").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let unboxed: Vec<_> = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    assert_eq!(unboxed[0].as_u32().unwrap(), 1234567890);
}

#[test]
#[serial]
fn test_push_string() {
    clear_storage();
    let bytecode = VM::dot2bin("PUSH_STRING hello").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    //first string internes to 0 index
    let unboxed: Vec<_> = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    assert_eq!(unboxed[0].as_str().unwrap(), "hello");
}

#[test]
#[serial]
fn test_if_then_true() {
    let bytecode = VM::dot2bin("PUSH_U32 100 PUSH_U32 100 EQ IF PUSH_U32 1 THEN").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let unboxed: Vec<_> = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    assert_eq!(unboxed[0].as_u32().unwrap(), 1);
}

#[test]
#[serial]
fn test_if_then_false() {
    let bytecode = VM::dot2bin("PUSH_U32 100 PUSH_U32 100 NEQ IF PUSH_U32 1 THEN").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack.as_slice(), []);
}

#[test]
#[serial]
fn test_if_then_true_nested() {
    let bytecode = VM::dot2bin(
        "PUSH_U32 100 PUSH_U32 100 EQ IF PUSH_U32 1 PUSH_U32 1 EQ IF PUSH_U32 2 THEN THEN",
    )
    .unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let unboxed: Vec<_> = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    assert_eq!(unboxed[0].as_u32().unwrap(), 2);
}

#[test]
#[serial]
fn test_if_then_false_nested() {
    let bytecode = VM::dot2bin("PUSH_U32 100 PUSH_U32 99 EQ IF PUSH_U32 1 PUSH_U32 1 EQ IF PUSH_U32 2 THEN THEN PUSH_U32 3").unwrap();
    let mut vm = VM::init(bytecode).unwrap(); // only 3 on stack, no 2 since if drops and jumps to then
    let stack = vm.run().unwrap();
    let unboxed = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    assert_eq!(unboxed[0].as_u32().unwrap(), 3);
}
/*
// instruction disassembly test, probably remove later
#[test]
#[serial] //?
fn test_disassembly_if_then() {
    let instructions =
        "PUSH_U32 100 PUSH_U32 100 EQ IF PUSH_U32 1 PUSH_U32 1 EQ IF PUSH_U32 2 THEN THEN";
    let bytecode = VM::dot2bin(instructions).unwrap();
    let vm = VM::init(bytecode).unwrap();
    let disassembled_bytecode = vm.bin2dot().unwrap();
    dbg!(&instructions);
    dbg!(&disassembled_bytecode);
    assert_eq!(instructions, disassembled_bytecode);
}
*/
#[test]
#[serial] //?
fn test_dup() {
    let bytecode = VM::dot2bin("PUSH_U32 100 DUP").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let unboxed = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    let stack_u32: Vec<u32> = unboxed.iter().map(|v| v.as_u32().unwrap()).collect();
    assert_eq!(stack_u32, vec![100, 100]);
}

#[test]
#[serial] //?
fn test_swap() {
    let bytecode = VM::dot2bin("PUSH_U32 1 PUSH_U32 2 SWAP").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let unboxed = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    let stack_u32: Vec<u32> = unboxed.iter().map(|v| v.as_u32().unwrap()).collect();
    assert_eq!(stack_u32, vec![2, 1]);
}

#[test]
#[serial] //?
fn test_dup_stack_underflow() {
    let bytecode = VM::dot2bin("DUP").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let result = vm.run();
    assert!(matches!(result, Err(VMError::StackUnderflow)));
}

#[test]
#[serial] //?
fn test_eq_true() {
    let bytecode = VM::dot2bin("PUSH_U32 161 PUSH_U32 161 EQ").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let unboxed = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    assert!(unboxed[0].as_bool().unwrap());
}

#[test]
#[serial]
fn test_neq_true() {
    clear_storage();
    let bytecode = VM::dot2bin("PUSH_U32 162 PUSH_U32 222 NEQ").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let unboxed = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    assert!(unboxed[0].as_bool().unwrap());
}

#[test]
#[serial]
fn test_lt_true() {
    clear_storage();
    let bytecode = VM::dot2bin("PUSH_U32 0 PUSH_U32 1 LT").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let unboxed = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    assert!(unboxed[0].as_bool().unwrap());
}

#[test]
#[serial]
fn test_gt_true() {
    clear_storage();
    let bytecode = VM::dot2bin("PUSH_U32 1 PUSH_U32 0 GT").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let unboxed = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    assert!(unboxed[0].as_bool().unwrap());
}

#[test]
#[serial] //?
fn test_drop_if_true() {
    let bytecode = VM::dot2bin("PUSH_U32 999 PUSH_U32 2 PUSH_U32 1 GT IF DROP THEN").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack.as_slice(), []); // 999 was dropped
}

#[test]
#[serial] //?
fn test_drop_if_false() {
    let bytecode = VM::dot2bin("PUSH_U32 999 PUSH_U32 2 PUSH_U32 3 GT IF DROP THEN").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let unboxed = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    assert_eq!(unboxed[0].as_u32().unwrap(), 999);
}

//memory test
#[test]
#[serial] //?
fn test_write_memory() {
    // memory slice 0..100 => in loop 0..100 fills slice with loop index  => stack contains slice (0,100) , memory vec![0..100]
    let bytecode =
        VM::dot2bin("NEW_VEC_U32 100 PUSH_U32 100 PUSH_U32 0 DO LOOP_INDEX LOOP_INDEX M_MUTA LOOP")
            .unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let unboxed = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    // since vec u32 -> each elemen u32 == 4 bytes -> 100*4
    assert_eq!(unboxed[0].as_vec_u32().unwrap(), (0, 400));
    let memory: Vec<u32> = vm
        .return_memory(0, 400)
        .map(|r| r.unwrap().as_u32().unwrap())
        .collect();
    let right: Vec<u32> = (0..100).collect();
    dbg!(&memory);
    dbg!(&right);
    assert_eq!(memory, right);
}

#[test]
#[serial]
fn test_write_memory_null_vals() {
    let bytecode =
        VM::dot2bin("NEW_VEC_U32 5 PUSH_U32 1 PUSH_U32 1 M_MUTA PUSH_U32 3 PUSH_U32 3 M_MUTA")
            .unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let raw_stack = vm.run().unwrap();
    let (offset, size) = vm
        .unbox(&raw_stack)
        .next()
        .unwrap()
        .unwrap()
        .as_vec_u32()
        .unwrap();
    let memory = vm
        .return_memory(offset, size as u32)
        .collect::<VMResult<Vec<_>>>()
        .unwrap();
    assert_eq!(
        memory,
        vec![
            Return::U32(0),
            Return::U32(1),
            Return::U32(0),
            Return::U32(3),
            Return::U32(0),
        ]
    );
}

/* #[test]
#[serial]
// max vec u32 size value is 2^16/4
fn test_write_memory_error() {
    let bytecode = VM::dot2bin("NEW_VEC_U32 17000").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let err = vm.run();
    assert!(matches!(err, Err(VMError::MSliceParamOverflow)));
} */

#[test]
#[serial]
fn test_create_task() {
    clear_storage();
    let ops = "PUSH_STRING TestTask PUSH_MAX_STATES 3 PUSH_CALLDATA [ ] T_CREATE";
    let bytecode = VM::dot2bin(ops).unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    vm.run().unwrap();
    let printed_task = vm.print_task(0).unwrap();
    let test_task = Task {
        id: 0,
        title: "TestTask".to_string(),
        state: TaskState { len: 3, state: 0 },
        instructions: Vec::new(),
    };
    assert_eq!(test_task.id, printed_task.id);
    assert_eq!(test_task.title, printed_task.title);
    assert_eq!(test_task.state, printed_task.state);
}

#[test]
#[serial]
fn test_get_task_field_title() {
    clear_storage();
    let ops = "PUSH_STRING MyTask1 PUSH_MAX_STATES 2 PUSH_CALLDATA [ ] T_CREATE \
                   PUSH_U32 0 PUSH_TASK_FIELD 0 T_GET_FIELD";
    let bytecode = VM::dot2bin(ops).unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let stack_slice = stack.as_slice();
    // Should get title string index
    assert_eq!(stack_slice.len(), 1);
    let test_task = vm.print_task(0).unwrap();
    assert_eq!(test_task.title, "MyTask1");
}

#[test]
#[serial]
fn test_get_task_field_status() {
    clear_storage();

    let ops = "PUSH_STRING MyTask1 PUSH_MAX_STATES 3 PUSH_CALLDATA [ ] T_CREATE \
                   PUSH_U32 0 PUSH_TASK_FIELD 1 T_GET_FIELD";
    let bytecode = VM::dot2bin(ops).unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let unboxed = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    assert_eq!(unboxed[0].as_u32().unwrap(), 0);
    let test_task = vm.print_task(0).unwrap();
    assert_eq!(test_task.state.state, 0); //same as previous NotComplete == 0
}

#[test]
#[serial]
fn test_set_task_field_status() {
    clear_storage();
    let ops = "PUSH_STRING TestTask PUSH_MAX_STATES 3 PUSH_CALLDATA [ ] T_CREATE \
                   PUSH_STATE 2 PUSH_U32 0 PUSH_TASK_FIELD 1 T_SET_FIELD \
                   PUSH_U32 0 PUSH_TASK_FIELD 1 T_GET_FIELD";
    let bytecode = VM::dot2bin(ops).unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let unboxed = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    assert_eq!(unboxed[0].as_u32().unwrap(), 2);
}

#[test]
#[serial]
fn test_delete_task() {
    clear_storage();
    let ops = "PUSH_STRING TaskToDelete PUSH_MAX_STATES 3 PUSH_CALLDATA [ ] T_CREATE \
                   PUSH_U32 0 T_DELETE";
    let bytecode = VM::dot2bin(ops).unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let _stack = vm.run().unwrap();
    let result = vm.print_task(0);
    assert!(matches!(result, Err(VMError::TaskNotFound(0))));
}

#[test]
#[serial]
fn test_task_with_simple_calldata() {
    clear_storage();
    let ops = "PUSH_STRING TestTask PUSH_MAX_STATES 3 \
               PUSH_CALLDATA [ PUSH_U32 42 END_CALL ] T_CREATE \
               PUSH_U32 0 DUP CALL"; // DUP is to keep task id
    let bytecode = VM::dot2bin(ops).unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let unboxed = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    let stack_u32: Vec<u32> = unboxed.iter().map(|v| v.as_u32().unwrap()).collect();
    assert_eq!(stack_u32, vec![0, 42]);
}

//assemble error handling
#[test]
#[serial]
fn test_task_not_found() {
    clear_storage();
    let bytecode = VM::dot2bin("PUSH_U32 99999 PUSH_TASK_FIELD 0 T_GET_FIELD").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let result = vm.run();
    assert!(matches!(result, Err(VMError::TaskNotFound(99999))));
}

#[test]
#[serial] //?
fn test_push_u32_overflow() {
    let bytecode = VM::dot2bin("PUSH_U32 4294967296").unwrap_err();
    assert!(matches!(bytecode, VMError::InvalidUINT { .. }));
}

#[test]
#[serial] //?
fn test_empty_instructions() {
    let bytecode: Vec<u8> = Vec::new();
    let result = VM::init(bytecode);
    assert!(matches!(result, Err(VMError::EmptyInstructions)));
}

#[test]
#[serial] //?
fn test_unknown_opcode() {
    let bytecode = VM::dot2bin("INVALID_OP");
    assert!(matches!(bytecode, Err(VMError::UnknownOpcode { .. })));
}

#[test]
#[serial] //?
fn test_missing_u32_val() {
    let bytecode = VM::dot2bin("PUSH_U32");
    assert!(matches!(bytecode, Err(VMError::UnexpectedEOI { .. })));
}

#[test]
#[serial] //?
fn test_malformed_calldata_missing_start_bracket() {
    let bytecode = VM::dot2bin("PUSH_CALLDATA PUSH_U32 1 ");
    assert!(matches!(
        bytecode,
        Err(VMError::MalformedCalldata {
            command: 0,
            context: "expected [ after PUSH_CALLDATA",
        })
    ));
}

#[test]
#[serial] //?
fn test_malformed_if_then_missing_if() {
    let bytecode = VM::dot2bin("PUSH_U32 1 PUSH_U32 1 EQ PUSH_U32 3 THEN");
    assert!(matches!(bytecode, Err(VMError::StackUnderflow)));
}

#[test]
#[serial] //?
fn test_malformed_if_then_missing_then() {
    let bytecode = VM::dot2bin("PUSH_U32 1 PUSH_U32 1 EQ IF PUSH_U32 3");
    assert!(matches!(bytecode, Err(VMError::MalformedIfThen { .. })));
}

#[test]
#[serial] //?
fn test_eoi_calldata_missing_end_bracket() {
    let bytecode = VM::dot2bin("PUSH_CALLDATA [ PUSH_U32 1 ");
    assert!(matches!(
        bytecode,
        Err(VMError::UnexpectedEOI {
            command: 0,
            context: "missing closing ]",
        })
    ));
}

#[test]
#[serial] //?
fn test_invalid_status() {
    let ops = "PUSH_STRING Task PUSH_MAX_STATES 3 PUSH_CALLDATA [ ] T_CREATE \
        PUSH_STATE 99 PUSH_U32 0 PUSH_TASK_FIELD 1 T_SET_FIELD";
    let bytecode = VM::dot2bin(ops).unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let result = vm.run();
    assert!(matches!(result, Err(VMError::InvalidStatus(99))));
}

#[test]
#[serial]
fn nested_loop_test() {
    let bytecode =
        VM::dot2bin("PUSH_U32 2 PUSH_U32 0 DO PUSH_U32 3 PUSH_U32 0 DO LOOP_INDEX LOOP LOOP")
            .unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let raw_stack = vm.run().unwrap();
    let unboxed = vm.unbox(&raw_stack).collect::<VMResult<Vec<_>>>().unwrap();
    let result_vec: Vec<u32> = unboxed.into_iter().map(|v| v.as_u32().unwrap()).collect();
    assert_eq!(result_vec, vec![0, 1, 2, 0, 1, 2]);
}

#[test]
#[serial]
fn nested_loop_overflow() {
    //current Control stack limit is 2
    let bytecode = VM::dot2bin("PUSH_U32 2 PUSH_U32 0 DO PUSH_U32 3 PUSH_U32 0 DO PUSH_U32 4 PUSH_U32 0 DO LOOP_INDEX LOOP LOOP LOOP").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let result = vm.run();
    assert!(matches!(result, Err(VMError::StackOverflow)));
}

#[test]
#[serial] //?
fn test_stack_overflow() {
    //loop 1_000_001 times push 99 - should stack overflow since limit 1 mil
    let bytecode = VM::dot2bin("PUSH_U32 1000001 PUSH_U32 0 DO PUSH_U32 99 LOOP").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let result = vm.run();
    assert!(matches!(result, Err(VMError::StackOverflow)));
}

#[test]
#[serial] //?
fn test_type_mismatch_eq() {
    let bytecode = VM::dot2bin("PUSH_U32 1 PUSH_STRING aaa EQ").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let result = vm.run();
    assert!(matches!(result, Err(VMError::TypeMismatch)));
}

#[test]
#[serial] //?
fn test_invalid_type_lt() {
    let bytecode = VM::dot2bin("PUSH_STRING aa PUSH_STRING aaa LT").unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let result = vm.run();
    assert!(matches!(result, Err(VMError::InvalidType)));
}

// complex tests

// Create task with title TargetTask -> Find task by id -> get it's task field 0 (Title) value -> push refference title TargetTask -> Compare string values -> drop task from stack if true
#[test]
#[serial]
fn test_conditional_task_filtering() {
    clear_storage();
    let ops = "PUSH_STRING TargetTask PUSH_MAX_STATES 3 PUSH_CALLDATA [ ] T_CREATE \
                   PUSH_U32 0 DUP PUSH_TASK_FIELD 0 T_GET_FIELD \
                   PUSH_STRING TargetTask EQ IF DROP THEN";
    let bytecode = VM::dot2bin(ops).unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let stack_slice = stack.as_slice();
    //dbg!(&stack);
    assert_eq!(stack_slice.len(), 0);
}

// create 3 tasks -> loop over tasks and push index (same as task id since based on s_len) to stack
#[test]
#[serial]
fn test_multiple_tasks_iteration() {
    clear_storage();
    let ops = "PUSH_STRING A PUSH_MAX_STATES 3 PUSH_CALLDATA [ ] T_CREATE \
                   PUSH_STRING B PUSH_MAX_STATES 4 PUSH_CALLDATA [ ] T_CREATE \
                   PUSH_STRING C PUSH_MAX_STATES 5 PUSH_CALLDATA [ ] T_CREATE \
                   S_LEN PUSH_U32 0 DO LOOP_INDEX LOOP";
    let bytecode = VM::dot2bin(ops).unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let unboxed = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    let stack_u32: Vec<u32> = unboxed.iter().map(|v| v.as_u32().unwrap()).collect();
    assert_eq!(stack_u32, vec![0, 1, 2]);
}

#[test]
#[serial]
fn test_task_creates_subtask() {
    clear_storage();
    // Task with calldata that creates another task
    let ops = "PUSH_STRING Parent PUSH_MAX_STATES 3 \
               PUSH_CALLDATA [ PUSH_STRING Child PUSH_MAX_STATES 2 PUSH_CALLDATA [ ] T_CREATE END_CALL ] \
               T_CREATE \
               PUSH_U32 0 CALL \
               S_LEN";
    let bytecode = VM::dot2bin(ops).unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let unboxed = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
    assert_eq!(unboxed[0].as_u32().unwrap(), 2);
}

#[test]
#[serial]
fn test_save_and_load() {
    clear_storage();
    // Create and save 2 tasks
    {
        let ops = "PUSH_STRING Task1 PUSH_MAX_STATES 3 PUSH_CALLDATA [ ] T_CREATE \
                       PUSH_STRING Task2 PUSH_MAX_STATES 2 PUSH_CALLDATA [ ] T_CREATE \
                       S_SAVE";
        let bytecode = VM::dot2bin(ops).unwrap();
        let mut vm = VM::init(bytecode).unwrap();
        vm.run().unwrap();
    }
    // Load
    {
        let bytecode = VM::dot2bin("S_LEN").unwrap();
        let mut vm = VM::init(bytecode).unwrap();
        let stack = vm.run().unwrap();
        let unboxed = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
        assert_eq!(unboxed[0].as_u32().unwrap(), 2);
    }
}

#[test]
#[serial]
fn test_delete_complex() {
    clear_storage();
    // Create and save 3 tasks
    {
        let ops = "PUSH_STRING Task1 PUSH_MAX_STATES 3 PUSH_CALLDATA [ ] T_CREATE \
                       PUSH_STRING Task2 PUSH_MAX_STATES 3 PUSH_CALLDATA [ ] T_CREATE \
                       PUSH_STRING Task3 PUSH_MAX_STATES 3 PUSH_CALLDATA [ ] T_CREATE
                       S_SAVE";
        let bytecode = VM::dot2bin(ops).unwrap();
        let mut vm = VM::init(bytecode).unwrap();
        vm.run().unwrap();
    }
    // Load
    {
        let bytecode = VM::dot2bin("S_LEN").unwrap();
        let mut vm = VM::init(bytecode).unwrap();
        let stack = vm.run().unwrap();
        let unboxed = vm.unbox(&stack).collect::<VMResult<Vec<_>>>().unwrap();
        assert_eq!(unboxed[0].as_u32().unwrap(), 3);
    }

    {
        //deleting 2nd task
        let ops_delete = "PUSH_U32 1 T_DELETE S_SAVE";
        let bytecode = VM::dot2bin(ops_delete).unwrap();
        let mut vm = VM::init(bytecode).unwrap();
        vm.run().unwrap();
        let t0 = vm.print_task(0);
        let t1 = vm.print_task(1);
        let t2 = vm.print_task(2);
        assert!(matches!(t0, Ok(..)));
        assert!(matches!(t1, Err(VMError::TaskNotFound(1))));
        assert!(matches!(t2, Ok(..)));
        // after deletion tasks_vm : task1 , none, task2
    }
}
