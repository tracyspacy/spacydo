/*
 * dot2bin - assemble text instructions to spacydo binary
 */

use crate::bytecode::opcodes::*;
use crate::errors::{VMError, VMResult};
use crate::inlinevec::InlineVec;

fn next_token<'a>(
    it: &mut std::iter::Enumerate<impl Iterator<Item = &'a str>>,
    cmd: usize,
    ctx: &'static str,
) -> VMResult<(usize, &'a str)> {
    it.next().ok_or(VMError::UnexpectedEOI {
        command: cmd,
        context: ctx,
    })
}

// make configurabe and put in same place with ControlStack and CallStack
const JUMP_STACK_LIMIT: usize = 2;
type JumpStack = InlineVec<u32, JUMP_STACK_LIMIT>;
//replce or import?
const TAG_STRING: u8 = 4;
const TAG_U32: u8 = 6;
//signaling byte
const WO_PAYLOAD: u8 = 0;
const W_PAYLOAD: u8 = 1;

pub fn dot2bin(src: &str) -> VMResult<Vec<u8>> {
    let mut tokens = src.split_whitespace().enumerate();
    let mut bytecode: Vec<u8> = Vec::new();
    // check on assembly nested ifs
    let mut jump_dest_stack: JumpStack = JumpStack::default();
    while let Some((i, token)) = tokens.next() {
        match token {
            "PUSH_U32" => {
                bytecode.push(PUSH_U32);
                let (pos, text) = next_token(&mut tokens, i, "missing u32")?;
                let value = text.parse::<u32>().map_err(|_| VMError::InvalidUINT {
                    command: pos,
                    value: text.into(),
                })?;
                bytecode.extend_from_slice(&value.to_be_bytes());
            }

            // dot is not aware of memory, so it requests alloc, not specifying offset (at least for now)
            // [size:16bits][TAG:8bits][SIGN:8bits][PAYLOAD]
            // it will restrinct size of string to 65535 bytes
            // for "hello"  [00 05] [06] [01] [68 65 6C 6C 6F]
            "PUSH_STRING" => {
                bytecode.push(M_STI);
                let (_pos, text) = next_token(&mut tokens, i, "missing String")?;
                let text_bytes = text.as_bytes();
                //len of bytes bytes
                let size =
                    u16::try_from(text_bytes.len()).map_err(|_| VMError::InstructionSizeError {
                        context: "String size exceeded limit",
                        max: u16::MAX as u32,
                    })?;
                bytecode.extend_from_slice(&size.to_be_bytes());
                bytecode.push(TAG_STRING);
                //signaling byte 1 == with payload
                bytecode.push(W_PAYLOAD);
                bytecode.extend_from_slice(text_bytes);
            }
            // immediate -> size, tag and payload  bytes following opcode
            // [size:16bits][TAG:8bits][SIGN:8bits]
            // to avoid confusion size should be in bytes
            //
            "NEW_VEC_U32_I" => {
                bytecode.push(M_STI);
                let (pos, text) = next_token(&mut tokens, i, "missing size")?;
                //size should be in bytes! so vm work with bytes sizes
                let size = text.parse::<u16>().map_err(|_| VMError::InvalidUINT {
                    command: pos,
                    value: text.into(),
                })?;
                bytecode.extend_from_slice(&size.to_be_bytes());
                bytecode.push(TAG_U32);
                //signaling byte 0 == without payload
                bytecode.push(WO_PAYLOAD);
            }

            "NEW_VEC_U32" => {
                bytecode.push(M_ST);
                bytecode.push(TAG_U32);
            }

            "PUSH_STATE" => {
                bytecode.push(PUSH_STATE);
                let (pos, text) = next_token(&mut tokens, i, "missing Tasks State")?;
                let value = text.parse::<u8>().map_err(|_| VMError::InvalidUINT {
                    command: pos,
                    value: text.into(),
                })?;
                bytecode.push(value);
            }
            "PUSH_MAX_STATES" => {
                bytecode.push(PUSH_MAX_STATES);
                let (pos, text) = next_token(&mut tokens, i, "missing Max States value")?;
                let value = text.parse::<u8>().map_err(|_| VMError::InvalidUINT {
                    command: pos,
                    value: text.into(),
                })?;
                bytecode.push(value);
            }
            "PUSH_TASK_FIELD" => {
                bytecode.push(PUSH_TASK_FIELD);
                let (pos, text) = next_token(&mut tokens, i, "missing Tasks Field")?;
                let value = text.parse::<u8>().map_err(|_| VMError::InvalidUINT {
                    command: pos,
                    value: text.into(),
                })?;
                bytecode.push(value);
            }
            "MUL" => {
                bytecode.push(MUL);
            }
            // u32 value - 4 bytes following opcode
            "MULI" => {
                bytecode.push(MULI);
                let (pos, text) = next_token(&mut tokens, i, "missing u32")?;
                let value = text.parse::<u32>().map_err(|_| VMError::InvalidUINT {
                    command: pos,
                    value: text.into(),
                })?;
                bytecode.extend_from_slice(&value.to_be_bytes());
            }

            "IF" => {
                bytecode.push(JUMP_IF_FALSE);
                jump_dest_stack.push(bytecode.len() as u32)?;
                bytecode.extend_from_slice(&0u32.to_be_bytes());
            }
            "THEN" => {
                let jump_dest = jump_dest_stack.pop()?;
                let upd_dest = bytecode.len() as u32;
                bytecode[jump_dest as usize..jump_dest as usize + 4]
                    .copy_from_slice(&upd_dest.to_be_bytes());
            }

            "PUSH_CALLDATA" => {
                bytecode.push(PUSH_CALLDATA);

                let (_bracket_pos, bracket) = next_token(&mut tokens, i, "empty calldata")?;

                if bracket != "[" {
                    return Err(VMError::MalformedCalldata {
                        command: i,
                        context: "expected [ after PUSH_CALLDATA",
                    });
                }

                let mut inner_instructions = String::new();
                let mut depth = 1;
                loop {
                    let (_pos, text) = next_token(&mut tokens, i, "missing closing ]")?;
                    //closing bracket
                    match text {
                        "[" => depth += 1,
                        "]" => depth -= 1,
                        _ => {}
                    }
                    if depth == 0 {
                        break;
                    }

                    if !inner_instructions.is_empty() {
                        inner_instructions.push(' ');
                    }
                    inner_instructions.push_str(text);
                }

                let calldata_bytecode = if inner_instructions.is_empty() {
                    Vec::new()
                } else {
                    dot2bin(&inner_instructions)?
                };

                // u16 - up to 65_535 . still a lot but better than u32
                let calldata_bytecode_len =
                    u16::try_from(calldata_bytecode.len()).map_err(|_| {
                        VMError::InstructionSizeError {
                            context: "Calldata size exceeded limit",
                            max: u16::MAX as u32,
                        }
                    })?;
                bytecode.extend_from_slice(&calldata_bytecode_len.to_be_bytes());
                bytecode.extend_from_slice(calldata_bytecode.as_slice());
            }

            "T_CREATE" => {
                bytecode.push(T_CREATE);
            }
            "T_GET_FIELD" => {
                bytecode.push(T_GET_FIELD);
            }
            "T_SET_FIELD" => {
                bytecode.push(T_SET_FIELD);
            }
            "T_DELETE" => {
                bytecode.push(T_DELETE);
            }
            "S_SAVE" => {
                bytecode.push(S_SAVE);
            }
            "S_LOAD" => {
                bytecode.push(S_LOAD);
            }
            "S_LEN" => {
                bytecode.push(S_LEN);
            }
            "DO" => {
                bytecode.push(DO);
            }
            "LOOP" => {
                bytecode.push(LOOP);
            }
            "LOOP_INDEX" => {
                bytecode.push(LOOP_INDEX);
            }
            "CALL" => {
                bytecode.push(CALL);
            }
            "END_CALL" => {
                bytecode.push(END_CALL);
            }
            "DROP" => {
                bytecode.push(DROP);
            }
            "DUP" => {
                bytecode.push(DUP);
            }
            "SWAP" => {
                bytecode.push(SWAP);
            }
            "EQ" => {
                bytecode.push(EQ);
            }
            "NEQ" => {
                bytecode.push(NEQ);
            }
            "LT" => {
                bytecode.push(LT);
            }
            "GT" => {
                bytecode.push(GT);
            }
            "M_MUTA" => {
                bytecode.push(M_MUTA);
            }
            "M_STI" => {
                bytecode.push(M_STI);
            }
            "M_ST" => {
                bytecode.push(M_ST);
            }
            _ => {
                return Err(VMError::UnknownOpcode {
                    opcode: token.to_string(),
                });
            }
        }
    }
    // we need to check if there is THEN for each IF : since we pop from jump_dest_stack in THEN case , it should be not emptied in case of missing THEN
    if !jump_dest_stack.is_empty() {
        return Err(VMError::MalformedIfThen {
            context: "Missing THEN",
        });
    }

    Ok(bytecode)
}
