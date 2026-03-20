/*
 * bin2dot - disassemble spacydo binary to text instructions
 */
use crate::bytecode::{helpers::*, opcodes::*};
use crate::inlinevec::InlineVec;
use crate::{VMError, VMResult};
use std::fmt::Write;

// make configurabe and put in same place with ControlStack and CallStack
const JUMP_STACK_LIMIT: usize = 2;
type JumpStack = InlineVec<u32, JUMP_STACK_LIMIT>;

pub fn bin2dot(bytecode: &[u8]) -> VMResult<String> {
    let mut result = String::new();
    let mut pc: usize = 0;
    let mut jump_dest_stack: JumpStack = JumpStack::default();
    while pc < bytecode.len() {
        let op = bytecode[pc];
        pc += 1;

        match op {
            PUSH_U32 => {
                result.push_str("PUSH_U32 ");
                let v = prepare_u32_from_be_checked(bytecode, pc)?; // change?
                write!(&mut result, "{} ", v).map_err(|_| VMError::WriteError)?;
                pc += 4;
            }
            PUSH_STRING => {
                result.push_str("PUSH_STRING ");
                let size = bytecode[pc] as usize;
                pc += 1;
                let str = std::str::from_utf8(&bytecode[pc..pc + size]).unwrap();
                result.push_str(str);
                result.push(' ');
                pc += size;
            }
            PUSH_STATE => {
                result.push_str("PUSH_STATE ");
                let v = prepare_u8(bytecode, pc)?;
                write!(&mut result, "{} ", v).map_err(|_| VMError::WriteError)?;
                //result.push((b'0' + v) as char);
                //result.push(' ');
                pc += 1;
            }
            PUSH_MAX_STATES => {
                result.push_str("PUSH_MAX_STATES ");
                let v = prepare_u8(bytecode, pc)?;
                write!(&mut result, "{} ", v).map_err(|_| VMError::WriteError)?;
                pc += 1;
            }

            PUSH_TASK_FIELD => {
                result.push_str("PUSH_TASK_FIELD ");
                let v = prepare_u8(bytecode, pc)?;
                write!(&mut result, "{} ", v).map_err(|_| VMError::WriteError)?;
                //result.push((b'0' + v) as char);
                // result.push(' ');
                pc += 1;
            }

            JUMP_IF_FALSE => {
                result.push_str("IF ");
                jump_dest_stack.push(prepare_u32_from_be_checked(bytecode, pc)?)?;
                pc += 4;
            }

            PUSH_CALLDATA => {
                result.push_str("PUSH_CALLDATA [ ");
                let size = prepare_u16_from_be_checked(bytecode, pc)? as usize;
                pc += 2;
                //check logic
                let inner = &bytecode[pc..pc + size];

                if !inner.is_empty() {
                    result.push_str(&bin2dot(inner)?);
                }
                result.push_str(" ] ");
            }

            T_CREATE => result.push_str("T_CREATE "),
            T_GET_FIELD => result.push_str("T_GET_FIELD "),
            T_SET_FIELD => result.push_str("T_SET_FIELD "),
            T_DELETE => result.push_str("T_DELETE "),
            S_SAVE => result.push_str("S_SAVE "),
            S_LOAD => result.push_str("S_LOAD "),
            S_LEN => result.push_str("S_LEN "),
            DO => result.push_str("DO "),
            LOOP => result.push_str("LOOP "),
            LOOP_INDEX => result.push_str("LOOP_INDEX "),
            CALL => result.push_str("CALL "),
            END_CALL => result.push_str("END_CALL "),
            DROP => result.push_str("DROP "),
            DUP => result.push_str("DUP "),
            SWAP => result.push_str("SWAP "),
            EQ => result.push_str("EQ "),
            NEQ => result.push_str("NEQ "),
            LT => result.push_str("LT "),
            GT => result.push_str("GT "),
            _ => {}
        }

        //checking if pc is eq to position of then word
        while let Some(dest) = jump_dest_stack.last() {
            if dest as usize == pc {
                result.push_str("THEN ");
                jump_dest_stack.pop()?;
            } else {
                break;
            }
        }
    }

    if !result.is_empty() {
        result.pop();
    }
    Ok(result)
}
