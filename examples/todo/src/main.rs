use spacydo::{Return, VM, VMResult};
use std::env;

// while it is possible to create tasks with various states, in this example we hardcode 3 possible states
// basic operations
const LS: &str = "S_LEN NEW_VEC_U32_I S_LEN PUSH_U32 0 DO LOOP_INDEX LOOP_INDEX M_MUTA LOOP";
const SHOW: &str = "NEW_VEC_U32 10 S_LEN PUSH_U32 0 DO LOOP_INDEX DUP EQ LOOP_INDEX CALL IF LOOP_INDEX LOOP_INDEX M_MUTA THEN LOOP";
const CREATE_TASK: &str =
    "PUSH_STRING %TITLE% PUSH_MAX_STATES 3 PUSH_CALLDATA [ %INSTRUCTIONS% ] T_CREATE S_SAVE";
const SET_STATUS: &str =
    "PUSH_STATE %STATE_VALUE% PUSH_U32 %ID% PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE";
const DELETE_TASK: &str = "PUSH_U32 %ID% T_DELETE S_SAVE";

#[derive(Clone, Copy)]
enum TaskStatus {
    NotComplete = 0,
    InProgress = 1,
    Complete = 2,
}

enum Command {
    Ls,
    Show,
    Add {
        title: String,
        calldata: Option<(String, Vec<String>)>,
    },
    Status {
        id: String,
        status: TaskStatus,
    },
    Delete {
        id: String,
    },
}

use std::fs;
use std::path::PathBuf;

fn load_calldata(name: &str, args: &[String]) -> Result<String, String> {
    let path = PathBuf::from("src/calldata.toml");

    let text = fs::read_to_string(&path).map_err(|_| "failed to read calldata.toml")?;

    let value: toml::Value = toml::from_str(&text).map_err(|_| "invalid calldata.toml")?;

    let code = value
        .get(name)
        .and_then(|v| v.get("instructions"))
        .and_then(|v| v.as_str())
        .ok_or(format!("calldata '{name}' not found"))?;

    let mut expanded = code.to_string();
    for (i, arg) in args.iter().enumerate() {
        expanded = expanded.replace(&format!("{{{{{i}}}}}"), arg);
    }

    Ok(expanded)
}

fn parse_args() -> Option<Command> {
    let mut args = env::args().skip(1);

    match args.next()?.as_str() {
        "show" => Some(Command::Show),
        "ls" => Some(Command::Ls),
        "add" => {
            let arguments: String = args.collect::<Vec<_>>().join(" ");
            if let Some(pos) = arguments.find("-calldata") {
                let title = arguments[..pos].trim().to_string();
                let calldata_str = arguments[pos + 9..].trim();
                let mut calldata_parts = calldata_str.split_whitespace();
                let instruction_name = calldata_parts.next()?.to_string();
                let args: Vec<String> = calldata_parts.map(|s| s.to_string()).collect();
                Some(Command::Add {
                    title,
                    calldata: Some((instruction_name, args)),
                })
            } else {
                Some(Command::Add {
                    title: arguments,
                    calldata: None,
                })
            }
        }
        "status" => {
            let id = args.next()?.parse().ok()?;
            let status = match args.next()?.as_str() {
                "inprogress" => TaskStatus::InProgress,
                "complete" => TaskStatus::Complete,
                "notcomplete" => TaskStatus::NotComplete,
                _ => return None,
            };

            Some(Command::Status { id, status })
        }
        "delete" => {
            let id = args.next()?.parse().ok()?;
            Some(Command::Delete { id })
        }

        _ => None,
    }
}

fn create_task_vm(title: &str, instructions: &str) -> String {
    CREATE_TASK
        .replace("%TITLE%", title)
        .replace("%INSTRUCTIONS%", instructions)
}
fn set_status_vm(id: &str, status: &TaskStatus) -> String {
    SET_STATUS
        .replace("%ID%", id)
        .replace("%STATE_VALUE%", &(*status as u8).to_string())
}

fn delete_task_vm(id: &str) -> String {
    DELETE_TASK.replace("%ID%", id)
}

fn show(instructions: &str) -> VMResult<()> {
    let bytecode = VM::dot2bin(instructions)?;
    let mut vm = VM::init(bytecode)?;

    let stack = vm.run()?;
    let task_ids = vm.unbox(&stack).next().unwrap()?.as_vec_u32()?.to_vec();

    println!("\n{:<4} {:<30} {:<15}", "ID", "Title", "Status");
    println!("{}", "─".repeat(50));

    for id in task_ids {
        if let Ok(task) = vm.print_task(id) {
            let status_display = match task.state.state {
                0 => "Not complete",
                1 => "In Progress",
                2 => "Complete",
                _ => unreachable!("Unknown state"),
            };
            println!("{:<4} {:<30} {:<15}", task.id, task.title, status_display);
        }
    }
    println!();
    Ok(())
}

fn main() -> VMResult<()> {
    let command = parse_args().expect("Invalid arguments");
    match command {
        Command::Ls => show(LS)?,
        Command::Show => show(SHOW)?,
        Command::Add { title, calldata } => {
            let instructions = match calldata {
                Some((name, params)) => load_calldata(&name, params.as_slice()).unwrap(),
                None => "".to_string(),
            };
            let dot = create_task_vm(&title, &instructions);
            let bytecode = VM::dot2bin(&dot)?;
            let mut vm = VM::init(bytecode)?;
            vm.run()?;
            println!("Task '{}' added", title);
        }

        Command::Status { id, status } => {
            let dot = set_status_vm(&id, &status);
            let bytecode = VM::dot2bin(&dot)?;
            let mut vm = VM::init(bytecode)?;
            vm.run()?;
            println!("Status of task {} updated", id);
        }
        Command::Delete { id } => {
            let dot = delete_task_vm(&id);
            let bytecode = VM::dot2bin(&dot)?;
            let mut vm = VM::init(bytecode)?;
            vm.run()?;
            println!("Task {} is deleted", id);
        }
    }

    Ok(())
}
