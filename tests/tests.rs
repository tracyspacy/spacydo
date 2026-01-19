use serial_test::serial;
use spacydo::{TRUE_VAL, Task, TaskStatus, VM, VMError, to_u32_val};
use std::fs;

fn clear_storage() {
    let _ = fs::remove_file("tasks.bin");
}

#[test]
#[serial] //?
fn test_push_u32() {
    let mut vm = VM::init("PUSH_U32 1234567890").unwrap();
    let stack = vm.run().unwrap();
    let unboxed = vm.unbox(stack).unwrap();
    assert_eq!(unboxed[0].as_u32().unwrap(), 1234567890);
}

#[test]
#[serial]
fn test_string_interning_reuses_index() {
    clear_storage();
    let mut vm = VM::init("PUSH_STRING hello PUSH_STRING hello").unwrap();
    let stack = vm.run().unwrap();

    assert_eq!(stack.len(), 2);
    assert_eq!(stack[0], stack[1]); // same intern index
}

#[test]
#[serial]
fn test_push_string() {
    clear_storage();
    let mut vm = VM::init("PUSH_STRING hello").unwrap();
    let stack = vm.run().unwrap();
    //first string internes to 0 index
    let unboxed = vm.unbox(stack).unwrap();
    assert_eq!("hello", unboxed[0].as_str().unwrap());
}

#[test]
#[serial]
fn test_if_then_true() {
    let mut vm = VM::init("PUSH_U32 100 PUSH_U32 100 EQ IF PUSH_U32 1 THEN").unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![to_u32_val(1)]);
}

#[test]
#[serial]
fn test_if_then_false() {
    let mut vm = VM::init("PUSH_U32 100 PUSH_U32 100 NEQ IF PUSH_U32 1 THEN").unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![]);
}

#[test]
#[serial]
fn test_if_then_true_nested() {
    let mut vm = VM::init(
        "PUSH_U32 100 PUSH_U32 100 EQ IF PUSH_U32 1 PUSH_U32 1 EQ IF PUSH_U32 2 THEN THEN",
    )
    .unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![to_u32_val(2)]);
}

#[test]
#[serial]
fn test_if_then_false_nested() {
    let mut vm = VM::init(
        "PUSH_U32 100 PUSH_U32 99 EQ IF PUSH_U32 1 PUSH_U32 1 EQ IF PUSH_U32 2 THEN THEN PUSH_U32 3", 
    )
    .unwrap(); // only 3 on stack, no 2 since if drops and jumps to then
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![to_u32_val(3)]);
}

// instruction disassembly test, probably remove later
#[test]
#[serial] //?
fn test_disassembly_if_then() {
    let instructions =
        "PUSH_U32 100 PUSH_U32 100 EQ IF PUSH_U32 1 PUSH_U32 1 EQ IF PUSH_U32 2 THEN THEN";
    let vm = VM::init(instructions).unwrap();
    let disassembled_bytecode = vm.disassemble_bytecode().unwrap();
    dbg!(&instructions);
    dbg!(&disassembled_bytecode);
    assert_eq!(instructions, disassembled_bytecode);
}

#[test]
#[serial] //?
fn test_dup() {
    let mut vm = VM::init("PUSH_U32 100 DUP").unwrap();
    let stack = vm.run().unwrap();
    let unboxed = vm.unbox(stack).unwrap();
    let stack_u32: Vec<u32> = unboxed.iter().map(|v| v.as_u32().unwrap()).collect();
    assert_eq!(stack_u32, vec![100, 100]);
}

#[test]
#[serial] //?
fn test_swap() {
    let mut vm = VM::init("PUSH_U32 1 PUSH_U32 2 SWAP").unwrap();
    let stack = vm.run().unwrap();
    let unboxed = vm.unbox(stack).unwrap();
    let stack_u32: Vec<u32> = unboxed.iter().map(|v| v.as_u32().unwrap()).collect();

    assert_eq!(stack_u32, vec![2, 1]);
}

#[test]
#[serial] //?
fn test_dup_stack_underflow() {
    let mut vm = VM::init("DUP").unwrap();
    let result = vm.run();
    assert!(matches!(result, Err(VMError::StackUnderflow)));
}

#[test]
#[serial] //?
fn test_eq_true() {
    let mut vm = VM::init("PUSH_U32 161 PUSH_U32 161 EQ").unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![TRUE_VAL]); // TRUE - think hot to display simplier like 1 and 0 again
}

#[test]
#[serial]
fn test_neq_true() {
    clear_storage();
    let mut vm = VM::init("PUSH_U32 162 PUSH_U32 222 NEQ").unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![TRUE_VAL]);
}

#[test]
#[serial]
fn test_lt_true() {
    clear_storage();
    let mut vm = VM::init("PUSH_U32 0 PUSH_U32 1 LT").unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![TRUE_VAL]);
}

#[test]
#[serial]
fn test_gt_true() {
    clear_storage();
    let mut vm = VM::init("PUSH_U32 1 PUSH_U32 0 GT").unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![TRUE_VAL]);
}

#[test]
#[serial] //?
fn test_drop_if_true() {
    let mut vm = VM::init("PUSH_U32 999 PUSH_U32 2 PUSH_U32 1 GT DROP_IF").unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![]); // 999 was dropped
}

#[test]
#[serial] //?
fn test_drop_if_false() {
    let mut vm = VM::init("PUSH_U32 999 PUSH_U32 2 PUSH_U32 3 GT DROP_IF").unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![to_u32_val(999)]); // nan boxed 999 kept
}

#[test]
#[serial]
fn test_create_task() {
    clear_storage();
    let ops = "PUSH_STRING TestTask PUSH_STATUS 0 PUSH_CALLDATA [ ] T_CREATE";
    let mut vm = VM::init(ops).unwrap();
    vm.run().unwrap();
    let printed_task = vm.print_task(0).unwrap();
    let test_task = Task {
        id: 0,
        title: "TestTask".to_string(),
        status: TaskStatus::NotComplete,
        instructions: "".to_string(),
    };
    assert_eq!(test_task.id, printed_task.id);
    assert_eq!(test_task.title, printed_task.title);
    assert_eq!(test_task.status, printed_task.status);
}

#[test]
#[serial]
fn test_get_task_field_title() {
    clear_storage();
    let ops = "PUSH_STRING MyTask1 PUSH_STATUS 2 PUSH_CALLDATA [ ] T_CREATE \
                   PUSH_U32 0 PUSH_TASK_FIELD 0 T_GET_FIELD";
    let mut vm = VM::init(ops).unwrap();
    let stack = vm.run().unwrap();

    // Should get title string index
    assert_eq!(stack.len(), 1);
    let test_task = vm.print_task(0).unwrap();
    assert_eq!(test_task.title, "MyTask1");
}

#[test]
#[serial]
fn test_get_task_field_status() {
    clear_storage();
    let ops = "PUSH_STRING MyTask1 PUSH_STATUS 0 PUSH_CALLDATA [ ] T_CREATE \
                   PUSH_U32 0 PUSH_TASK_FIELD 1 T_GET_FIELD";
    let mut vm = VM::init(ops).unwrap();
    let stack = vm.run().unwrap();

    assert_eq!(stack, vec![to_u32_val(0)]);
    let test_task = vm.print_task(0).unwrap();
    assert_eq!(test_task.status, TaskStatus::NotComplete); //same as previous NotComplete == 0
}

#[test]
#[serial]
fn test_set_task_field_status() {
    clear_storage();
    let ops = "PUSH_STRING TestTask PUSH_STATUS 0 PUSH_CALLDATA [ ] T_CREATE \
                   PUSH_STATUS 2 PUSH_U32 0 PUSH_TASK_FIELD 1 T_SET_FIELD \
                   PUSH_U32 0 PUSH_TASK_FIELD 1 T_GET_FIELD";
    let mut vm = VM::init(ops).unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![to_u32_val(2)]);
}

#[test]
#[serial]
fn test_delete_task() {
    clear_storage();
    let ops = "PUSH_STRING TaskToDelete PUSH_STATUS 2 PUSH_CALLDATA [ ] T_CREATE S_LEN \
                   PUSH_U32 0 T_DELETE S_LEN";
    let mut vm = VM::init(ops).unwrap();
    let stack = vm.run().unwrap();

    assert_eq!(stack, vec![to_u32_val(1), to_u32_val(0)]); // 1 task after task create and 0 tasks remain after delete
}

#[test]
#[serial]
fn test_task_with_simple_calldata() {
    clear_storage();
    let ops = "PUSH_STRING TestTask PUSH_STATUS 2 \
               PUSH_CALLDATA [ PUSH_U32 42 END_CALL ] T_CREATE \
               PUSH_U32 0 DUP CALL"; // DUP is to keep task id
    let mut vm = VM::init(ops).unwrap();
    let stack = vm.run().unwrap();

    assert_eq!(stack, vec![to_u32_val(0), to_u32_val(42)]);
}

//assemble error handling
#[test]
#[serial]
fn test_task_not_found() {
    clear_storage();
    let mut vm = VM::init("PUSH_U32 99999 PUSH_TASK_FIELD 0 T_GET_FIELD").unwrap();
    let result = vm.run();
    assert!(matches!(result, Err(VMError::TaskNotFound(99999))));
}

#[test]
#[serial] //?
fn test_push_u32_overflow() {
    let err = VM::init("PUSH_U32 4294967296").unwrap_err();
    assert!(matches!(err, VMError::InvalidUINT { .. }));
}

#[test]
#[serial] //?
fn test_empty_instructions() {
    let result = VM::init("");
    assert!(matches!(result, Err(VMError::EmptyInstructions)));
}

#[test]
#[serial] //?
fn test_unknown_opcode() {
    let result = VM::init("INVALID_OP");
    assert!(matches!(result, Err(VMError::UnknownOpcode { .. })));
}

#[test]
#[serial] //?
fn test_missing_u32_val() {
    let result = VM::init("PUSH_U32");
    assert!(matches!(result, Err(VMError::UnexpectedEOI { .. })));
}

#[test]
#[serial] //?
fn test_malformed_calldata_missing_start_bracket() {
    let result = VM::init("PUSH_CALLDATA PUSH_U32 1 ");
    assert!(matches!(
        result,
        Err(VMError::MalformedCalldata {
            command: 0,
            context: "expected [ after PUSH_CALLDATA",
        })
    ));
}

#[test]
#[serial] //?
fn test_malformed_if_then_missing_if() {
    let result = VM::init("PUSH_U32 1 PUSH_U32 1 EQ PUSH_U32 3 THEN");
    assert!(matches!(result, Err(VMError::MalformedIfThen { .. })));
}

#[test]
#[serial] //?
fn test_malformed_if_then_missing_then() {
    let result = VM::init("PUSH_U32 1 PUSH_U32 1 EQ IF PUSH_U32 3");
    assert!(matches!(result, Err(VMError::MalformedIfThen { .. })));
}

#[test]
#[serial] //?
fn test_eoi_calldata_missing_end_bracket() {
    let result = VM::init("PUSH_CALLDATA [ PUSH_U32 1 ");
    assert!(matches!(
        result,
        Err(VMError::UnexpectedEOI {
            command: 0,
            context: "missing closing ]",
        })
    ));
}

#[test]
#[serial] //?
fn test_invalid_status() {
    let ops = "PUSH_STRING Task PUSH_STATUS 99 PUSH_CALLDATA [ ] T_CREATE";
    let mut vm = VM::init(ops).unwrap();
    let result = vm.run();

    assert!(matches!(result, Err(VMError::InvalidStatus(99))));
}

#[test]
#[serial] //?
fn test_stack_overflow() {
    //loop 1_000_001 times push 99 - should stack overflow since limit 1 mil
    let mut vm = VM::init("PUSH_U32 1000001 PUSH_U32 0 DO PUSH_U32 99 LOOP").unwrap();
    let result = vm.run();
    assert!(matches!(result, Err(VMError::StackOverflow)));
}

#[test]
#[serial] //?
fn test_type_mismatch_eq() {
    let mut vm = VM::init("PUSH_U32 1 PUSH_STRING aaa EQ").unwrap();
    let result = vm.run();
    assert!(matches!(result, Err(VMError::TypeMismatch)));
}

#[test]
#[serial] //?
fn test_invalid_type_lt() {
    let mut vm = VM::init("PUSH_STRING aa PUSH_STRING aaa LT").unwrap();
    let result = vm.run();
    assert!(matches!(result, Err(VMError::InvalidType)));
}

// complex tests

// Create task with title TargetTask -> Find task by id -> get it's task field 0 (Title) value -> push refference title TargetTask -> Compare string values -> drop task from stack if true
#[test]
#[serial]
fn test_conditional_task_filtering() {
    clear_storage();
    let ops = "PUSH_STRING TargetTask PUSH_STATUS 2 PUSH_CALLDATA [ ] T_CREATE \
                   PUSH_U32 0 DUP PUSH_TASK_FIELD 0 T_GET_FIELD \
                   PUSH_STRING TargetTask EQ DROP_IF";
    let mut vm = VM::init(ops).unwrap();
    let stack = vm.run().unwrap();
    dbg!(&stack);
    assert_eq!(stack.len(), 0);
}

// create 3 tasks -> loop over tasks and push index (same as task id since based on s_len) to stack
#[test]
#[serial]
fn test_multiple_tasks_iteration() {
    clear_storage();
    let ops = "PUSH_STRING A PUSH_STATUS 0 PUSH_CALLDATA [ ] T_CREATE \
                   PUSH_STRING B PUSH_STATUS 0 PUSH_CALLDATA [ ] T_CREATE \
                   PUSH_STRING C PUSH_STATUS 0 PUSH_CALLDATA [ ] T_CREATE \
                   S_LEN PUSH_U32 0 DO LOOP_INDEX LOOP";
    let mut vm = VM::init(ops).unwrap();
    let stack = vm.run().unwrap();
    dbg!(&stack);
    assert_eq!(stack, vec![to_u32_val(0), to_u32_val(1), to_u32_val(2)]);
}

#[test]
#[serial]
fn test_task_creates_subtask() {
    clear_storage();
    // Task with calldata that creates another task
    let ops = "PUSH_STRING Parent PUSH_STATUS 2 \
               PUSH_CALLDATA [ PUSH_STRING Child PUSH_STATUS 0 PUSH_CALLDATA [ ] T_CREATE END_CALL ] \
               T_CREATE \
               PUSH_U32 0 CALL \
               S_LEN";
    let mut vm = VM::init(ops).unwrap();
    let stack = vm.run().unwrap();
    // let task = vm.print_task(1).unwrap();
    // dbg!(&task);
    dbg!(&stack);
    assert_eq!(stack, vec![to_u32_val(2)]);
}

#[test]
#[serial]
fn test_save_and_load() {
    clear_storage();
    // Create and save 2 tasks
    {
        let ops = "PUSH_STRING Task1 PUSH_STATUS 2 PUSH_CALLDATA [ ] T_CREATE \
                       PUSH_STRING Task2 PUSH_STATUS 1 PUSH_CALLDATA [ ] T_CREATE \
                       S_SAVE";
        let mut vm = VM::init(ops).unwrap();
        vm.run().unwrap();
    }
    // Load
    {
        let mut vm = VM::init("S_LEN").unwrap();
        let stack = vm.run().unwrap();
        assert_eq!(stack, vec![to_u32_val(2)]);
    }
}

#[test]
#[serial]
fn test_delete_complex() {
    clear_storage();
    // Create and save 3 tasks
    {
        let ops = "PUSH_STRING Task1 PUSH_STATUS 0 PUSH_CALLDATA [ ] T_CREATE \
                       PUSH_STRING Task2 PUSH_STATUS 1 PUSH_CALLDATA [ ] T_CREATE \
                       PUSH_STRING Task3 PUSH_STATUS 1 PUSH_CALLDATA [ ] T_CREATE
                       S_SAVE";
        let mut vm = VM::init(ops).unwrap();
        vm.run().unwrap();
    }
    // Load
    {
        let mut vm = VM::init("S_LEN").unwrap();
        let stack = vm.run().unwrap();

        assert_eq!(stack, vec![to_u32_val(3)]);
    }

    {
        //deleting 2nd task
        let ops_delete = "PUSH_U32 1 T_DELETE S_SAVE";
        let mut vm = VM::init(ops_delete).unwrap();
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
