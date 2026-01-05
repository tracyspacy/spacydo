use serial_test::serial;
use spacydo::{Task, TaskStatus, VM, VMError};
use std::fs;

fn clear_storage() {
    let _ = fs::remove_file("tasks.bin");
}

#[test]
#[serial] //?
fn test_push_u8() {
    let mut vm = VM::init("PUSH_U8 10").unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![10]);
}

#[test]
#[serial] //?
fn test_push_u64() {
    let mut vm = VM::init("PUSH_U64 1234567890").unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![1234567890]);
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
#[serial] //?
fn test_dup() {
    let mut vm = VM::init("PUSH_U64 100 DUP").unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![100, 100]);
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
    let mut vm = VM::init("PUSH_U64 161 PUSH_U64 161 EQ").unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![1]); // TRUE
}

#[test]
#[serial]
fn test_neq_true() {
    clear_storage();
    let mut vm = VM::init("PUSH_U64 162 PUSH_U64 222 NEQ").unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![1]);
}

#[test]
#[serial]
fn test_lt_true() {
    clear_storage();
    let mut vm = VM::init("PUSH_U8 0 PUSH_U64 1 LT").unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![1]);
}

#[test]
#[serial]
fn test_gt_true() {
    clear_storage();
    let mut vm = VM::init("PUSH_U8 1 PUSH_U64 0 GT").unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![1]);
}

#[test]
#[serial] //?
fn test_drop_if_true() {
    let mut vm = VM::init("PUSH_U64 999 PUSH_U64 1 DROP_IF").unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![]); // 999 was dropped
}
#[test]
#[serial] //?
fn test_drop_if_false() {
    let mut vm = VM::init("PUSH_U64 999 PUSH_U64 0 DROP_IF").unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![999]); // 999 kept
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
                   PUSH_U64 0 PUSH_TASK_FIELD 0 T_GET_FIELD";
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
                   PUSH_U64 0 PUSH_TASK_FIELD 1 T_GET_FIELD";
    let mut vm = VM::init(ops).unwrap();
    let stack = vm.run().unwrap();

    assert_eq!(stack, vec![0]);
    let test_task = vm.print_task(0).unwrap();
    assert_eq!(test_task.status, TaskStatus::NotComplete); //same as previous NotComplete == 0
}

#[test]
#[serial]
fn test_set_task_field_status() {
    clear_storage();
    let ops = "PUSH_STRING TestTask PUSH_STATUS 0 PUSH_CALLDATA [ ] T_CREATE \
                   PUSH_STATUS 2 PUSH_U64 0 PUSH_TASK_FIELD 1 T_SET_FIELD \
                   PUSH_U64 0 PUSH_TASK_FIELD 1 T_GET_FIELD";
    let mut vm = VM::init(ops).unwrap();
    let stack = vm.run().unwrap();
    assert_eq!(stack, vec![2]);
}

#[test]
#[serial]
fn test_delete_task() {
    clear_storage();
    let ops = "PUSH_STRING TaskToDelete PUSH_STATUS 2 PUSH_CALLDATA [ ] T_CREATE S_LEN \
                   PUSH_U64 0 T_DELETE S_LEN";
    let mut vm = VM::init(ops).unwrap();
    let stack = vm.run().unwrap();

    assert_eq!(stack, vec![1, 0]); // 1 task after task create and 0 tasks remain after delete
}

#[test]
#[serial]
fn test_task_with_simple_calldata() {
    clear_storage();
    let ops = "PUSH_STRING TestTask PUSH_STATUS 2 \
               PUSH_CALLDATA [ PUSH_U64 42 END_CALL ] T_CREATE \
               PUSH_U64 0 DUP CALL"; // DUP is to keep task id
    let mut vm = VM::init(ops).unwrap();
    let stack = vm.run().unwrap();

    assert_eq!(stack, vec![0, 42]);
}

//assemble error handling
#[test]
#[serial]
fn test_task_not_found() {
    clear_storage();
    let mut vm = VM::init("PUSH_U64 99999 PUSH_TASK_FIELD 0 T_GET_FIELD").unwrap();
    let result = vm.run();
    assert!(matches!(result, Err(VMError::TaskNotFound(99999))));
}

#[test]
#[serial] //?
fn test_push_u8_overflow() {
    let err = VM::init("PUSH_U8 256").unwrap_err();
    let _value = "256".to_string();
    assert!(matches!(
        err,
        VMError::InvalidUINT {
            command: 1,
            value: _value
        }
    ));
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
    dbg!(&result);
    assert!(matches!(result, Err(VMError::UnknownOpcode { .. })));
}

#[test]
#[serial] //?
fn test_missing_u64_val() {
    let result = VM::init("PUSH_U64");
    assert!(matches!(result, Err(VMError::UnexpectedEOI { .. })));
}

#[test]
#[serial] //?
fn test_malformed_calldata_missing_start_bracket() {
    let result = VM::init("PUSH_CALLDATA PUSH_U64 1 ");
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
fn test_eoi_calldata_missing_end_bracket() {
    let result = VM::init("PUSH_CALLDATA [ PUSH_U64 1 ");
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
    let mut vm = VM::init("PUSH_U64 1000001 PUSH_U64 0 DO PUSH_U64 99 LOOP").unwrap();
    let result = vm.run();
    assert!(matches!(result, Err(VMError::StackOverflow)));
}

// complex tests

// Create task with title TargetTask -> Find task by id -> get it's task field 0 (Title) value -> push refference title TargetTask -> Compare string values -> drop task from stack if true
#[test]
#[serial]
fn test_conditional_task_filtering() {
    clear_storage();
    let ops = "PUSH_STRING TargetTask PUSH_STATUS 2 PUSH_CALLDATA [ ] T_CREATE \
                   PUSH_U64 0 DUP PUSH_TASK_FIELD 0 T_GET_FIELD \
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
                   S_LEN PUSH_U64 0 DO LOOP_INDEX LOOP";
    let mut vm = VM::init(ops).unwrap();
    let stack = vm.run().unwrap();
    dbg!(&stack);
    assert_eq!(stack, vec![0, 1, 2]);
}

#[test]
#[serial]
fn test_task_creates_subtask() {
    clear_storage();
    // Task with calldata that creates another task
    let ops = "PUSH_STRING Parent PUSH_STATUS 2 \
               PUSH_CALLDATA [ PUSH_STRING Child PUSH_STATUS 0 PUSH_CALLDATA [ ] T_CREATE END_CALL ] \
               T_CREATE \
               PUSH_U8 0 CALL \
               S_LEN";
    let mut vm = VM::init(ops).unwrap();
    let stack = vm.run().unwrap();
    // let task = vm.print_task(1).unwrap();
    // dbg!(&task);
    dbg!(&stack);
    assert_eq!(stack, vec![2]);
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
        assert_eq!(stack, vec![2]);
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

        assert_eq!(stack, vec![3]);
    }

    {
        //deleting 2nd task
        let ops_delete = "PUSH_U64 1 T_DELETE S_SAVE";
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
