use crate::errors::{VMError, VMResult};

#[derive(Debug, Default)]
pub struct InstructionsPool {
    instructions: Vec<Vec<u8>>,
}

impl InstructionsPool {
    pub(crate) fn intern_instructions(&mut self, calldata: Vec<u8>) -> u64 {
        let idx = self.instructions.len();
        self.instructions.push(calldata);
        idx as u64
    }
    pub(crate) fn len(&self) -> usize {
        self.instructions.len()
    }
    //err if no instruction in pool
    pub(crate) fn get(&self, index: usize) -> VMResult<&[u8]> {
        self.instructions
            .get(index)
            .map(|val| val.as_slice())
            .ok_or(VMError::InvalidInstructionsIndex(index))
    }
}
