use spacydo::{VM, VMResult};
use std::{
    io::{Write, stdout},
    thread, time,
};

// this task is a simple state machine which has own state transition rules in calldata that executes on each task's calldata call
// calldata logic: check task's state -> transit to next -> save
// state transition rule: 0->1->2->0

const CREATE_TRAFFIC_LIGHT: &str = "PUSH_STRING TRAFFIC_LIGHT PUSH_MAX_STATES 3 PUSH_CALLDATA \
   [ PUSH_U32 0 PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 0 EQ \
   IF PUSH_STATE 1 PUSH_U32 0 PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE END_CALL THEN  \
   PUSH_U32 0 PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 1 EQ \
   IF PUSH_STATE 2 PUSH_U32 0 PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE END_CALL THEN  \
   PUSH_U32 0 PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 2 EQ \
   IF PUSH_STATE 0 PUSH_U32 0 PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE END_CALL THEN  \
   END_CALL ] T_CREATE S_SAVE";

fn call() -> VMResult<u8> {
    let mut vm = VM::init("PUSH_U32 0 CALL")?;
    vm.run()?;
    let task = vm.print_task(0)?;
    let state = task.state.state;
    Ok(state)
}

fn main() {
    for _ in 0..100 {
        match call() {
            Ok(s) => {
                let c = match s {
                    0 => "🟢⚫️⚫️",
                    1 => "⚫️🟡⚫️",
                    2 => "⚫️⚫️🔴",
                    _ => unreachable!("⚫️⚫️⚫️"),
                };
                print!("\r{}", c);
                stdout().flush().unwrap();
            }
            Err(_) => {
                let mut vm = VM::init(CREATE_TRAFFIC_LIGHT).unwrap();
                vm.run().unwrap();
                println!("A Traffic Light task is created.");
            }
        }
        let sec = time::Duration::from_millis(1000);
        thread::sleep(sec);
    }
}
