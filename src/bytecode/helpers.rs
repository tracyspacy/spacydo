use crate::errors::{VMError, VMResult};
pub fn prepare_u32_from_be_checked(inst_slice: &[u8], pc: usize) -> VMResult<u32> {
    Ok(u32::from_be_bytes(
        inst_slice[pc..pc + 4]
            .try_into()
            .map_err(|_| VMError::UnexpectedEOB)?,
    ))
}
pub fn prepare_u8(bytecode: &[u8], pc: usize) -> VMResult<&u8> {
    bytecode.get(pc).ok_or(VMError::UnexpectedEOB)
}

#[inline]
pub fn pop_stack<T>(stack: &mut Vec<T>) -> VMResult<T> {
    stack.pop().ok_or(VMError::StackUnderflow)
}
#[inline]
pub fn last_stack<T>(stack: &[T]) -> VMResult<&T> {
    stack.last().ok_or(VMError::StackUnderflow)
}

const MAX_STACK_SIZE: usize = 1_000_000; // move to constants.rs file
#[inline]
pub fn push_stack(stack: &mut Vec<u64>, value: u64) -> VMResult<()> {
    if stack.len() >= MAX_STACK_SIZE {
        return Err(VMError::StackOverflow);
    }
    stack.push(value);
    Ok(())
}
