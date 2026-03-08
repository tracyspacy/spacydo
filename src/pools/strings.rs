use crate::errors::{VMError, VMResult};
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct StringPool {
    map: HashMap<Box<[u8]>, u32>,
    vec: Vec<Box<[u8]>>,
}

impl StringPool {
    pub(crate) fn intern_string(&mut self, s: &[u8]) -> u32 {
        // Check for dup
        if let Some(&idx) = self.map.get(s) {
            return idx;
        }

        // probably use box:: leak and 'static
        let idx = self.vec.len() as u32;
        //let boxed: Box<[u8]> = s.into_boxed_str();
        self.map.insert(s.into(), idx);
        self.vec.push(s.into());
        idx
    }

    pub(crate) fn resolve(&self, idx: usize) -> VMResult<&str> {
        let bytes = self.vec.get(idx).ok_or(VMError::InvalidStringIndex(idx))?;
        let str = std::str::from_utf8(bytes).unwrap();
        Ok(str)
    }
}
