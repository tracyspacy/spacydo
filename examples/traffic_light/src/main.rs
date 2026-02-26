use spacydo::{VM, VMResult};
use std::{
    io::{Write, stdout},
    thread, time,
};

// this task is a simple state machine which has own state transition rules in calldata that executes on each task's calldata call
// calldata logic: check task's state -> transit to next
// state transition rule: 0->1->2->0

const CREATE_TRAFFIC_LIGHT: &str = "PUSH_STRING TRAFFIC_LIGHT PUSH_MAX_STATES 3 PUSH_CALLDATA \
   [ PUSH_U32 0 PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 0 EQ \
   IF PUSH_STATE 1 PUSH_U32 0 PUSH_TASK_FIELD 1 T_SET_FIELD END_CALL THEN  \
   PUSH_U32 0 PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 1 EQ \
   IF PUSH_STATE 2 PUSH_U32 0 PUSH_TASK_FIELD 1 T_SET_FIELD END_CALL THEN  \
   PUSH_U32 0 PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 2 EQ \
   IF PUSH_STATE 0 PUSH_U32 0 PUSH_TASK_FIELD 1 T_SET_FIELD END_CALL THEN  \
   END_CALL ] T_CREATE S_SAVE";

// if persistent state on each state change is crucial , ie in case of abrupt need to continue from previous
/*
const CREATE_TRAFFIC_LIGHT: &str = "PUSH_STRING TRAFFIC_LIGHT PUSH_MAX_STATES 3 PUSH_CALLDATA \
   [ PUSH_U32 0 PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 0 EQ \
   IF PUSH_STATE 1 PUSH_U32 0 PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE END_CALL THEN  \
   PUSH_U32 0 PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 1 EQ \
   IF PUSH_STATE 2 PUSH_U32 0 PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE END_CALL THEN  \
   PUSH_U32 0 PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 2 EQ \
   IF PUSH_STATE 0 PUSH_U32 0 PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE END_CALL THEN  \
   END_CALL ] T_CREATE S_SAVE";
*/

fn call(vm: &mut VM) -> VMResult<u8> {
    vm.run()?;
    let task = vm.print_task(0)?;
    let state = task.state.state;
    Ok(state)
}

fn save() -> VMResult<()> {
    let mut saver = VM::init("S_SAVE")?;
    saver.run()?;
    Ok(())
}
fn display_color(state: u8) {
    let c = match state {
        0 => "🟢⚫️⚫️",
        1 => "⚫️🟡⚫️",
        2 => "⚫️⚫️🔴",
        _ => unreachable!("⚫️⚫️⚫️"),
    };
    print!("\r{}", c);
    stdout().flush().unwrap();
}
// It is an updated traffic_light example with reduced expensive i/o operations (save on each vm.run())
// ! if it is critical to preserve persistent state on each state change - use commented version of bytecode above
// now we initialize vm once with simple instructions to call calldata of 0th task
// then in loop we reuse it multiple times (vm.run())
// task is saved to persistent storage only once at the end
fn main() {
    let mut vm = VM::init("PUSH_U32 0 CALL").unwrap();
    let first_state = match call(&mut vm) {
        Ok(s) => s,
        Err(_) => {
            VM::init(CREATE_TRAFFIC_LIGHT).unwrap().run().unwrap();
            println!("Traffic Light task is created");
            vm = VM::init("PUSH_U32 0 CALL").unwrap();
            call(&mut vm).unwrap()
        }
    };
    display_color(first_state);

    for _ in 0..100 {
        match call(&mut vm) {
            Ok(s) => {
                display_color(s);
            }
            Err(_) => {
                save().unwrap();
            }
        }
        let sec = time::Duration::from_millis(1000);
        thread::sleep(sec);
    }
    save().unwrap();
}
