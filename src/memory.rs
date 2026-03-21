use crate::errors::{VMError, VMResult};
use crate::values::{TAG_STRING, TAG_U32, Value, to_string_vec_val, to_u32_vec_val};
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
    // for storage allocating stringsẞ
    //probably rename error
    pub(crate) fn alloc_auto(&mut self, bytes: &[u8]) -> VMResult<(u32, u16)> {
        let offset = self.0.len() as u32;
        let size = u16::try_from(bytes.len()).map_err(|_| VMError::InstructionSizeError {
            context: "size exceeded limit",
            max: u16::MAX as u32,
        })?;
        self.0.extend_from_slice(bytes);
        Ok((offset, size))
    }

    // accepts size in bytes
    pub(crate) fn alloc_manual(
        &mut self,
        offset: u32,
        size: u16,
        tag: u8,
        payload: &[u8],
    ) -> VMResult<Value> {
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
        let payload_bytes = payload.to_be_bytes();
        // shortcut as we use u32 payload -> payload_bytes gives us [0x00,0x00,0x00,0x00]
        // if 4 bytes -> [4 - 4 ..] (all 4 bytes) - for vec u32 vals
        // if 1 byte -> [4 - 1.. ] ([3..] low byte) - for string vals
        self.0[abs_index..abs_index + element_size_bytes]
            .copy_from_slice(&payload_bytes[4 - element_size_bytes..]);
        Ok(())
    }

    pub(crate) fn get_slice_bytes(&self, offset: u32, size: u16) -> &[u8] {
        &self.0[offset as usize..offset as usize + size as usize]
    }

    pub(crate) fn get_slice_as_str(&self, offset: u32, size: u16) -> VMResult<&str> {
        let bytes = &self.0[offset as usize..offset as usize + size as usize];
        std::str::from_utf8(bytes).map_err(|_| VMError::BytesToStringConversionError)
    }

    //check if correct
    // keep unused for now
    /* pub(crate) fn get_mem_slice_typed(&self, offset: u32, size: u16, tag: u64) -> VMResult<&[u8]> {
        let element_size_bytes = element_size_bytes(tag)?;
        let size_in_bytes = size as usize * element_size_bytes;
        Ok(&self.0[offset as usize..offset as usize + size_in_bytes])
    }
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
    */
}
