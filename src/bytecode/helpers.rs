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
