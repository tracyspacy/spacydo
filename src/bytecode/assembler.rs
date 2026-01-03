use crate::bytecode::opcodes::*;
use crate::errors::{VMError, VMResult};
use crate::pools::{InstructionsPool, StringPool};

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

pub fn assemble(
    src: &str,
    string_pool: &mut StringPool,
    instructions_pool: &mut InstructionsPool,
) -> VMResult<Vec<u8>> {
    let mut tokens = src.split_whitespace().enumerate();
    let mut bytecode: Vec<u8> = Vec::new();

    while let Some((i, token)) = tokens.next() {
        match token {
            "PUSH_U8" => {
                bytecode.push(PUSH_U8);
                let (pos, text) = next_token(&mut tokens, i, "missing u8")?;
                let value = text.parse::<u8>().map_err(|_| VMError::InvalidUINT {
                    command: pos,
                    value: text.into(),
                })?;

                bytecode.push(value);
            }
            "PUSH_U64" => {
                bytecode.push(PUSH_U64);
                let (pos, text) = next_token(&mut tokens, i, "missing u64")?;
                let value = text.parse::<u64>().map_err(|_| VMError::InvalidUINT {
                    command: pos,
                    value: text.into(),
                })?;
                bytecode.extend_from_slice(&value.to_be_bytes());
            }
            "PUSH_STRING" => {
                bytecode.push(PUSH_STRING);
                let (_pos, text) = next_token(&mut tokens, i, "missing String")?;
                let idx = string_pool.intern_string(text.to_string());
                bytecode.extend_from_slice(&idx.to_be_bytes());
            }
            "PUSH_STATUS" => {
                bytecode.push(PUSH_STATUS);
                let (pos, text) = next_token(&mut tokens, i, "missing Tasks Status")?;
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
                    assemble(&inner_instructions, string_pool, instructions_pool)?
                };

                let idx = instructions_pool.intern_instructions(calldata_bytecode);
                bytecode.extend_from_slice(&idx.to_be_bytes());
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
            "DROP_IF" => {
                bytecode.push(DROP_IF);
            }
            "DUP" => {
                bytecode.push(DUP);
            }
            "EQ" => {
                bytecode.push(EQ);
            }
            "NEQ" => {
                bytecode.push(NEQ);
            }
            _ => {
                return Err(VMError::UnknownOpcode {
                    opcode: token.to_string(),
                });
            }
        }
    }
    Ok(bytecode)
}
