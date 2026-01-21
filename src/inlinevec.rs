// The `InlineVec` is a vector-like backed by fixed size array data structure
// Parameters of `InlineVec<T, C>` : `T` stands for the element type and `C` for the maximum capacity, ie array size.
// InlineVec is fixed size - ie not growable.

#[derive(Debug)]
pub struct InlineVec<T, const C: usize> {
    len: u32,
    array: [T; C],
}

use crate::errors::{VMError, VMResult};

// TODO:
//  - add Display impl for pretty printing > InlineVec[1,2] instead of InlineVec { len: 2, array: [1, 2, 0, 0, 0] }
//  - add macro ?
//  - add `as_slice`
//  - add `as_mut_slice`
//  - add to_vec?

impl<T: Default + Copy, const C: usize> InlineVec<T, C> {
    pub(crate) fn new() -> InlineVec<T, C> {
        InlineVec {
            len: 0,
            array: [(); C].map(|_| Default::default()),
        }
    }

    pub(crate) const fn capacity(&self) -> usize {
        C
    }
    pub(crate) const fn len(&self) -> usize {
        self.len as usize
    }
    pub(crate) const fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub(crate) const fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }
    pub(crate) const fn remaining_capacity(&self) -> usize {
        self.capacity() - self.len()
    }
    pub(crate) fn push(&mut self, val: T) -> VMResult<()> {
        if self.is_full() {
            return Err(VMError::StackOverflow);
        }
        self.array[self.len()] = val;
        self.len += 1;
        Ok(())
    }
    pub(crate) fn pop(&mut self) -> VMResult<T> {
        if self.is_empty() {
            return Err(VMError::StackUnderflow);
        }
        self.len -= 1;

        let value = std::mem::take(&mut self.array[self.len()]);

        Ok(value)
    }
    pub(crate) fn last(&self) -> VMResult<T> {
        if self.is_empty() {
            return Err(VMError::StackUnderflow);
        }
        let value = self.array[self.len() - 1];

        Ok(value)
    }

    pub fn as_slice(&self) -> &[T] {
        &self.array[..self.len()]
    }
}

impl<T: Default + Copy, const C: usize> Default for InlineVec<T, C> {
    /// Return an empty array
    fn default() -> InlineVec<T, C> {
        InlineVec::new()
    }
}

#[cfg(test)]
mod test {
    use crate::errors::VMError;
    use crate::inlinevec::InlineVec;
    #[test]
    fn create_inline_vec() {
        let ivec: InlineVec<u64, 255> = InlineVec::new();
        assert_eq!(ivec.array, [0u64; u8::MAX as usize]);
    }

    #[test]
    fn push_success() {
        let mut ivec: InlineVec<u64, 5> = InlineVec::new();
        ivec.push(1).unwrap();
        ivec.push(2).unwrap();
        assert_eq!(ivec.array, [1, 2, 0, 0, 0]);
    }

    #[test]
    fn push_overflow() {
        let mut ivec: InlineVec<u64, 1> = InlineVec::new();
        ivec.push(1).unwrap();
        assert!(matches!(ivec.push(2), Err(VMError::StackOverflow)));
    }
    #[test]
    fn pop_succes() {
        let mut ivec: InlineVec<u64, 5> = InlineVec::new();
        ivec.push(1).unwrap();
        ivec.push(2).unwrap();
        ivec.pop().unwrap();
        assert_eq!(ivec.array, [1, 0, 0, 0, 0]);
    }
    #[test]
    fn pop_underflow() {
        let mut ivec: InlineVec<u64, 5> = InlineVec::new();
        assert!(matches!(ivec.pop(), Err(VMError::StackUnderflow)));
    }
}
