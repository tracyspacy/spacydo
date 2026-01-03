use crate::errors::{VMError, VMResult};
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct StringPool {
    map: HashMap<Box<str>, usize>,
    vec: Vec<Box<str>>,
}

impl StringPool {
    pub(crate) fn intern_string(&mut self, s: String) -> usize {
        // Check for dup
        if let Some(&idx) = self.map.get(s.as_str()) {
            return idx;
        }

        // probably use box:: leak and 'static
        let idx = self.vec.len();
        let boxed: Box<str> = s.into_boxed_str();
        self.map.insert(boxed.clone(), idx);
        self.vec.push(boxed);
        idx
    }

    pub(crate) fn resolve(&self, idx: usize) -> VMResult<&str> {
        self.vec
            .get(idx)
            .map(|opt| &**opt)
            .ok_or(VMError::InvalidStringIndex(idx))
    }
}
