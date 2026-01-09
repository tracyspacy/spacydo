use crate::bytecode::{helpers::*, opcodes::*};
use crate::pools::{InstructionsPool, StringPool};
use crate::{VMError, VMResult};

use std::fmt::Write;

const EMPTY: [u8; 0] = [];

pub fn disassemble(
    bytecode: &[u8],
    string_pool: &StringPool,
    instructions_pool: &InstructionsPool,
) -> VMResult<String> {
    let mut result = String::new();
    let mut pc: usize = 0;

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
                let idx = prepare_u32_from_be_checked(bytecode, pc)? as usize;
                let s = string_pool.resolve(idx)?;
                result.push_str(s);
                result.push(' ');
                pc += 4;
            }
            PUSH_STATUS => {
                result.push_str("PUSH_STATUS ");
                let v = prepare_u8(bytecode, pc)?;
                write!(&mut result, "{} ", v).map_err(|_| VMError::WriteError)?;
                //result.push((b'0' + v) as char);
                //result.push(' ');
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
            PUSH_CALLDATA => {
                result.push_str("PUSH_CALLDATA ");
                let idx = prepare_u32_from_be_checked(bytecode, pc)? as usize;
                pc += 4;
                //check logic
                let inner: &[u8] = if idx < instructions_pool.len() {
                    instructions_pool.get(idx)?
                } else {
                    &EMPTY
                };

                if !inner.is_empty() {
                    result.push_str(&disassemble(inner, string_pool, instructions_pool)?);
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
            DROP_IF => result.push_str("DROP_IF "),
            DUP => result.push_str("DUP "),
            SWAP => result.push_str("SWAP "),
            EQ => result.push_str("EQ "),
            NEQ => result.push_str("NEQ "),
            LT => result.push_str("LT "),
            GT => result.push_str("GT "),
            _ => {}
        }
    }

    if !result.is_empty() {
        result.pop();
    }
    Ok(result)
}
