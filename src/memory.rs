use crate::errors::{VMError, VMResult};
use crate::values::{TAG_STRING, TAG_U32, Value, to_string_vec_val, to_u32_vec_val};
use crate::{FALSE_VAL, TRUE_VAL, to_bool_val};
#[derive(Debug)]
pub struct LinearMemory(Vec<u8>);

#[inline]
pub(crate) const fn element_size_bytes(tag: u64) -> VMResult<usize> {
    match tag {
        TAG_STRING => Ok(1),
        TAG_U32 => Ok(4),
        _ => Err(VMError::InvalidType),
    }
}

impl LinearMemory {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    //the purpose is to align string and especially vec u32 acorrectly.
    // it may be string with len 2 before, that will lead to incorrect allignment of vec
    // works: 1-4 => 4 , 5-8 => 8 etc
    // also important to check proposed offset by assembler and not allow to overlay
    fn offset_aligned(&self, tag: u8) -> u32 {
        match tag as u64 {
            TAG_U32 => (self.len() + 3) & !3,
            _ => self.len(),
        }
    }

    // accepts size in bytes
    // gets aligned offset
    // since we have 2 sourses -> load and bytecode (dot) without coordination, auto alloc is safer option
    pub(crate) fn alloc(&mut self, size: u16, tag: u8, payload: &[u8]) -> VMResult<Value> {
        let offset = self.offset_aligned(tag);
        let end = offset as usize + size as usize;
        if end > self.0.len() {
            self.0.resize(end, 0u8);
        }

        if !payload.is_empty() {
            self.0[offset as usize..end].copy_from_slice(payload);
        }
        let val = match tag as u64 {
            TAG_STRING => to_string_vec_val(offset, size)?,
            TAG_U32 => to_u32_vec_val(offset, size)?,
            _ => return Err(VMError::InvalidType),
        };
        Ok(val)
    }
    //todo
    //check endianness for the whole vm!
    pub(crate) fn mut_vec(
        &mut self,
        offset: u32,
        size: u16,
        index: u32,
        payload: u32,
        tag: u64,
    ) -> VMResult<()> {
        let element_size_bytes = element_size_bytes(tag)?;
        if index >= size as u32 / element_size_bytes as u32 {
            return Err(VMError::MSliceOutOfBounds {
                index: index * element_size_bytes as u32,
                size: size as u32,
            });
        }
        let abs_index = offset as usize + index as usize * element_size_bytes;
        let payload_bytes = payload.to_ne_bytes();
        // shortcut as we use u32 payload -> payload_bytes gives us [0x00,0x00,0x00,0x00]
        // if 4 bytes -> [.. element_size_bytes] (all 4 bytes) - for vec u32 vals
        // if 1 byte -> [.. element_size_bytes ] ([..1] low byte) - for string vals
        self.0[abs_index..abs_index + element_size_bytes]
            .copy_from_slice(&payload_bytes[..element_size_bytes]);
        Ok(())
    }

    pub(crate) fn get_slice_bytes(&self, offset: u32, size: u16) -> &[u8] {
        &self.0[offset as usize..offset as usize + size as usize]
    }
    pub(crate) fn get_slice_as_str(&self, offset: u32, size: u16) -> VMResult<&str> {
        let bytes = &self.0[offset as usize..offset as usize + size as usize];
        std::str::from_utf8(bytes).map_err(|_| VMError::BytesToStringConversionError)
    }

    //reinterpreting &[u8] as &[u32]
    // unlike from_raw_parts not cause ub, just populates suffix and prefix even if size is wrong?
    // https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html
    // https://doc.rust-lang.org/std/primitive.slice.html#method.align_to

    pub(crate) fn get_slice_as_u32(
        &self,
        offset: u32,
        size: u16,
        //  tag: u8,
    ) -> VMResult<&[u32]> {
        let bytes = self.get_slice_bytes(offset, size);
        if !core::mem::size_of_val::<[u8]>(bytes).is_multiple_of(size_of::<u32>()) {
            return Err(VMError::SliceSizeMismatch);
        }
        let (prefix, aligned_u32, suffix) = unsafe { bytes.align_to::<u32>() };
        if !prefix.is_empty() || !suffix.is_empty() {
            return Err(VMError::AlignmentMismatch);
        }
        Ok(aligned_u32)
    }

    pub(crate) fn is_m_slice_eq(&self, left: (u32, u16), right: (u32, u16)) -> Value {
        let (lo, ls) = left;
        let (ro, rs) = right;
        if ls != rs {
            return FALSE_VAL;
        }
        if lo == ro {
            return TRUE_VAL;
        }
        to_bool_val(self.get_slice_bytes(lo, ls) == self.get_slice_bytes(ro, rs))
    }

    pub(crate) fn len(&self) -> u32 {
        self.0.len() as u32
    }
}
