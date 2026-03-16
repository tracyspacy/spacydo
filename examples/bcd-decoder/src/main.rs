use spacydo::{Return, Task, VM};
use std::env;

const BITS: &str = "PUSH_STRING %BIT% PUSH_MAX_STATES 2 PUSH_CALLDATA [ ] T_CREATE S_SAVE";

// Inverter/NOT gate automatically invert state based on state of bit
const INVERTER: &str = "PUSH_STRING %INV_N% PUSH_MAX_STATES 2 PUSH_CALLDATA [ \
PUSH_U32 %ID% PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 0 EQ \
IF PUSH_STATE 1 PUSH_U32 %OWN_ID% PUSH_TASK_FIELD 1 T_SET_FIELD END_CALL THEN \
PUSH_U32 %ID% PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 1 EQ \
IF PUSH_STATE 0 PUSH_U32 %OWN_ID% PUSH_TASK_FIELD 1 T_SET_FIELD END_CALL THEN  \
] T_CREATE S_SAVE";

const ANDGATE: &str = "PUSH_STRING %AND_GATE% PUSH_MAX_STATES 2 PUSH_CALLDATA [ \
PUSH_U32 %ID_0% PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 1 NEQ \
IF \
    PUSH_STATE 0 PUSH_U32 %OWN_ID% PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE END_CALL THEN \
PUSH_U32 %ID_1% PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 1 NEQ \
IF \
    PUSH_STATE 0 PUSH_U32 %OWN_ID% PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE END_CALL THEN \
PUSH_U32 %ID_2% PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 1 NEQ \
IF \
    PUSH_STATE 0 PUSH_U32 %OWN_ID% PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE END_CALL THEN \
PUSH_U32 %ID_3% PUSH_TASK_FIELD 1 T_GET_FIELD PUSH_STATE 1 NEQ \
IF \
    PUSH_STATE 0 PUSH_U32 %OWN_ID% PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE END_CALL THEN \
PUSH_STATE 1 PUSH_U32 %OWN_ID% PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE END_CALL \
] T_CREATE S_SAVE";

const SET_STATE: &str =
    "PUSH_STATE %STATE_VALUE% PUSH_U32 %ID% PUSH_TASK_FIELD 1 T_SET_FIELD S_SAVE";

const SHOW: &str = "PUSH_U32 0 S_LEN M_SLICE S_LEN PUSH_U32 0 DO LOOP_INDEX DUP EQ LOOP_INDEX CALL IF LOOP_INDEX LOOP_INDEX M_STORE THEN LOOP";

fn init() {
    let bytecode = VM::dot2bin("PUSH_U32 17 PUSH_TASK_FIELD 1 T_GET_FIELD").unwrap();
    let mut check_vm = VM::init(bytecode).unwrap();
    if check_vm.run().is_err() {
        // BITS
        create_bit("BIT_0");
        create_bit("BIT_1");
        create_bit("BIT_2");
        create_bit("BIT_3");

        // Inverters / NOT GATES -> INVERTED BITS
        create_inverter("INV_BIT_0", "0", "4");
        create_inverter("INV_BIT_1", "1", "5");
        create_inverter("INV_BIT_2", "2", "6");
        create_inverter("INV_BIT_3", "3", "7");

        // AND GATE for 0
        create_and_gate("AND_0", "4", "5", "6", "7", "8");

        // AND GATE for 1
        create_and_gate("AND_1", "0", "5", "6", "7", "9");

        // AND GATE for 2
        create_and_gate("AND_2", "4", "1", "6", "7", "10");

        // AND GATE for 3
        create_and_gate("AND_3", "0", "1", "6", "7", "11");

        // AND GATE for 4
        create_and_gate("AND_4", "4", "5", "2", "7", "12");

        // AND GATE for 5
        create_and_gate("AND_5", "0", "5", "2", "7", "13");

        // AND GATE for 6
        create_and_gate("AND_6", "4", "1", "2", "7", "14");

        // AND GATE for 7
        create_and_gate("AND_7", "0", "1", "2", "7", "15");

        // AND GATE for 8
        create_and_gate("AND_8", "4", "5", "6", "3", "16");

        // AND GATE for 9
        create_and_gate("AND_9", "0", "5", "6", "3", "17");
    }
}

fn create_bit(title: &str) {
    let bit = BITS.replace("%BIT%", title);
    let bytecode = VM::dot2bin(&bit).unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    vm.run().unwrap();
}

fn create_inverter(title: &str, bit_id: &str, own_id: &str) {
    let inverter = INVERTER
        .replace("%INV_N%", title)
        .replace("%ID%", bit_id)
        .replace("%OWN_ID%", own_id);
    let bytecode = VM::dot2bin(&inverter).unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    vm.run().unwrap();
}

fn create_and_gate(
    title: &str,
    input_0: &str,
    input_1: &str,
    input_2: &str,
    input_3: &str,
    own_id: &str,
) {
    let and_gate = ANDGATE
        .replace("%AND_GATE%", title)
        .replace("%ID_0%", input_0)
        .replace("%ID_1%", input_1)
        .replace("%ID_2%", input_2)
        .replace("%ID_3%", input_3)
        .replace("%OWN_ID%", own_id);
    let bytecode = VM::dot2bin(&and_gate).unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    vm.run().unwrap();
}

fn set_state(state: &str, id: &str) {
    let set_state = SET_STATE
        .replace("%STATE_VALUE%", state)
        .replace("%ID%", id);
    let bytecode = VM::dot2bin(&set_state).unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    vm.run().unwrap();
}

fn show() {
    let bytecode = VM::dot2bin(SHOW).unwrap();
    let mut vm = VM::init(bytecode).unwrap();
    let stack = vm.run().unwrap();
    let (offset, size) = vm
        .unbox(&stack)
        .next()
        .unwrap()
        .unwrap()
        .as_mem_slice()
        .unwrap();

    let task_ids: Vec<u32> = vm
        .return_memory(offset, size)
        .filter_map(|r| match r.unwrap() {
            Return::U32(val) => Some(val),
            _ => None,
        })
        .collect();

    let tasks: [Task; 18] = std::array::from_fn(|i| {
        let id = task_ids[i];
        vm.print_task(id).unwrap()
    });

    show_and_gate(&tasks);
}
fn show_and_gate(tasks: &[Task]) {
    let bits = &tasks[..=3];
    let inverted_bits = &tasks[4..=7];
    let mut and_gate_input: [&str; 4] = [""; 4];

    for i in 0..4 {
        if bits[i].state.state != 0 {
            and_gate_input[i] = bits[i].title.as_str();
        } else {
            and_gate_input[i] = inverted_bits[i].title.as_str();
        }
    }
    let and_gates = &tasks[8..];
    // should be either 1 or none
    let and_gate_index = and_gates
        .iter()
        .position(|g| g.state.state != 0)
        .expect("No AND Gate with Output 1 - decimal number is > 9");
    println!("\n ***** BINARY CODED DECIMAL (BCD) DECODER ***** ");
    println!(
        "\nBits           : {:?} {:?} {:?} {:?}",
        bits[3].state.state, bits[2].state.state, bits[1].state.state, bits[0].state.state
    );
    println!(
        "Inverted Bits  : {:?} {:?} {:?} {:?}",
        inverted_bits[3].state.state,
        inverted_bits[2].state.state,
        inverted_bits[1].state.state,
        inverted_bits[0].state.state
    );

    println!();
    println!("{:9} ───────────┐", and_gate_input[3]);
    println!("{:9} ───────────┼──┐", and_gate_input[2]);
    println!("{:9} ───────────┼──┼──┐", and_gate_input[1]);
    println!("{:9} ───────────┼──┼──┼──┐", and_gate_input[0]);
    println!("                     │  │  │  │");
    println!("                   ┌─▼──▼──▼──▼─┐");
    println!("                   │ AND GATE {} │ ", and_gate_index,);
    println!("                   ╰╮          ╭╯   ");
    println!("                    ╰──────────╯");
    println!("                          │      ");

    let output_line: String = (0..10)
        .map(|i| {
            if i == and_gate_index {
                "▼".to_string()
            } else {
                " ".to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    println!("                {}", output_line);
    println!("Decimal number: 0 1 2 3 4 5 6 7 8 9 \n");
}

fn main() {
    init();
    let mut args = env::args().skip(1);
    let bits = args.next().expect("No bits");
    if bits.len() == 4 && bits.chars().all(|c| matches!(c, '0' | '1')) {
        for i in 0..4 {
            let c = bits.chars().rev().nth(i).unwrap().to_string();
            set_state(&c, &i.to_string());
        }
    } else {
        eprintln!("Error: input must be exactly 4 bits containing only 0 or 1 -for ex 1001");
        std::process::exit(1);
    }
    show();
}
