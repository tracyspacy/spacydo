// custom binary encoding/decoding implementation
// Why: bincode was great, but unfortunately is not maintained anymore
// A minimal own implementation, despite lacking the polish of an established solutions,
// is the right long-term move, would allow vm be dependency free, edge cases would be covered.
//
// some comments:
// to little endian for now similar to bincode default
// String use u16 ie 2 byte for len. Should be safer on decoding (65535 bytes allocation in worst case)
// TaskStatus uses u8 ie 1 byte not u32 like bincode
//
// ? we can use separate types for title and instructions , so we can encode them with different len - u8 for title and u16 for instructions

use crate::errors::{VMError, VMResult};
use crate::storage::task_types::{StorageData, Task, TaskStatus};
use std::io::{Read, Write};

const U8_BYTES: u8 = 1;
const U16_BYTES: u8 = 2;
const U32_BYTES: u8 = 4;

pub trait Encode {
    fn encode<W: Write>(&self, w: &mut W) -> VMResult<()>;
}
/*
we should encode u32,String, TaskStatus enum

pub struct Task {
    pub id: u32,
    pub title: String,
    pub status: TaskStatus,
    pub instructions: String,
}
*/

impl Encode for u8 {
    fn encode<W: Write>(&self, w: &mut W) -> VMResult<()> {
        w.write_all(&[*self])
            .map_err(|_| VMError::StorageWriteError)
    }
}

impl Encode for u16 {
    fn encode<W: Write>(&self, w: &mut W) -> VMResult<()> {
        let max_bits = 1 + self.checked_ilog2().unwrap_or(0);
        match max_bits {
            0..=8 => {
                let size_bytes: u8 = 1;
                w.write_all(&[size_bytes])
                    .map_err(|_| VMError::StorageWriteError)?;
                w.write_all(&[*self as u8])
                    .map_err(|_| VMError::StorageWriteError)?;
            }
            _ => {
                let size_bytes: u8 = 2;
                w.write_all(&[size_bytes])
                    .map_err(|_| VMError::StorageWriteError)?;
                w.write_all(&self.to_le_bytes())
                    .map_err(|_| VMError::StorageWriteError)?;
            }
        }
        Ok(())
    }
}

// need to write compact - if u32 is <= u8 -> write as single byte
// to determine amount of bits needed we can use (log2(value)+1)
// idea is to provide byte size as u8 len val [LEN:u8][VAL]
impl Encode for u32 {
    fn encode<W: Write>(&self, w: &mut W) -> VMResult<()> {
        let max_bits = 1 + self.checked_ilog2().unwrap_or(0);
        match max_bits {
            0..=8 => {
                w.write_all(&[U8_BYTES])
                    .map_err(|_| VMError::StorageWriteError)?;
                w.write_all(&[*self as u8])
                    .map_err(|_| VMError::StorageWriteError)?;
            }
            9..=16 => {
                w.write_all(&[U16_BYTES])
                    .map_err(|_| VMError::StorageWriteError)?;
                w.write_all(&(*self as u16).to_le_bytes())
                    .map_err(|_| VMError::StorageWriteError)?;
            }
            _ => {
                w.write_all(&[U32_BYTES])
                    .map_err(|_| VMError::StorageWriteError)?;
                w.write_all(&self.to_le_bytes())
                    .map_err(|_| VMError::StorageWriteError)?;
            }
        }
        Ok(())
    }
}

// format similar to default bincode : le [len+ascii bytes]
impl Encode for String {
    fn encode<W: Write>(&self, w: &mut W) -> VMResult<()> {
        let bytes = self.as_bytes();
        let len = bytes.len();
        if len > u16::MAX as usize {
            return Err(VMError::StorageSizeTooBigError);
        }
        (len as u16).encode(w)?;
        w.write_all(bytes).map_err(|_| VMError::StorageWriteError)?;
        Ok(())
    }
}
// format similar to default bincode : le [len+values_bytes]
impl Encode for Vec<Task> {
    fn encode<W: Write>(&self, w: &mut W) -> VMResult<()> {
        let len = self.len() as u32;
        //need limit
        len.encode(w)?;
        for item in self {
            item.encode(w)?;
        }
        Ok(())
    }
}

impl Encode for TaskStatus {
    fn encode<W: Write>(&self, w: &mut W) -> VMResult<()> {
        (*self as u8).encode(w)?;
        Ok(())
    }
}

impl Encode for Task {
    fn encode<W: Write>(&self, w: &mut W) -> VMResult<()> {
        self.id.encode(w)?;
        self.title.encode(w)?;
        self.status.encode(w)?;
        self.instructions.encode(w)?;
        Ok(())
    }
}

/*
pub(crate) struct StorageData {
    tasks: Vec<Task>,
    next_id: u32,
}
*/

impl Encode for StorageData {
    fn encode<W: Write>(&self, w: &mut W) -> VMResult<()> {
        self.tasks.encode(w)?;
        self.next_id.encode(w)?;
        Ok(())
    }
}

//
// DECODING
//

pub trait Decode: Sized {
    fn decode<R: Read>(r: &mut R) -> VMResult<Self>;
}

impl Decode for u8 {
    fn decode<R: Read>(r: &mut R) -> VMResult<Self> {
        let mut buf = [0u8; 1];
        r.read_exact(&mut buf)
            .map_err(|_| VMError::StorageReadError)?;
        Ok(buf[0])
    }
}

impl Decode for u16 {
    fn decode<R: Read>(r: &mut R) -> VMResult<Self> {
        let mut bytes = [0u8; 1];
        r.read_exact(&mut bytes)
            .map_err(|_| VMError::StorageReadError)?;
        match bytes[0] {
            U8_BYTES => {
                let mut bytes = [0u8; U8_BYTES as usize];
                r.read_exact(&mut bytes)
                    .map_err(|_| VMError::StorageReadError)?;
                Ok(bytes[0] as u16)
            }
            U16_BYTES => {
                let mut bytes = [0u8; U16_BYTES as usize];
                r.read_exact(&mut bytes)
                    .map_err(|_| VMError::StorageReadError)?;
                Ok(u16::from_le_bytes(bytes))
            }
            _ => Err(VMError::StorageReadError),
        }
    }
}

impl Decode for u32 {
    fn decode<R: Read>(r: &mut R) -> VMResult<Self> {
        let mut bytes = [0u8; 1];
        r.read_exact(&mut bytes)
            .map_err(|_| VMError::StorageReadError)?;
        match bytes[0] {
            U8_BYTES => {
                let mut bytes = [0u8; U8_BYTES as usize];
                r.read_exact(&mut bytes)
                    .map_err(|_| VMError::StorageReadError)?;
                Ok(bytes[0] as u32)
            }
            U16_BYTES => {
                let mut bytes = [0u8; U16_BYTES as usize];
                r.read_exact(&mut bytes)
                    .map_err(|_| VMError::StorageReadError)?;
                Ok(u16::from_le_bytes(bytes) as u32)
            }
            U32_BYTES => {
                let mut bytes = [0u8; U32_BYTES as usize];
                r.read_exact(&mut bytes)
                    .map_err(|_| VMError::StorageReadError)?;
                Ok(u32::from_le_bytes(bytes))
            }

            _ => Err(VMError::StorageReadError),
        }
    }
}

impl Decode for String {
    fn decode<R: Read>(r: &mut R) -> VMResult<Self> {
        let len = u16::decode(r)?;
        let mut buf = vec![0u8; len as usize];
        r.read_exact(&mut buf)
            .map_err(|_| VMError::StorageReadError)?;
        // add new error type utf8 conversion error
        String::from_utf8(buf).map_err(|_| VMError::StorageReadError)
    }
}

impl Decode for Vec<Task> {
    fn decode<R: Read>(r: &mut R) -> VMResult<Self> {
        let len = u32::decode(r)?;
        //need limit and check
        let mut buf = Vec::with_capacity(len as usize);
        for _ in 0..len {
            buf.push(Task::decode(r)?);
        }
        Ok(buf)
    }
}

impl Decode for TaskStatus {
    fn decode<R: Read>(r: &mut R) -> VMResult<Self> {
        TaskStatus::try_from(u8::decode(r)? as u32)
    }
}

impl Decode for Task {
    fn decode<R: Read>(r: &mut R) -> VMResult<Self> {
        Ok(Task {
            id: u32::decode(r)?,
            title: String::decode(r)?,
            status: TaskStatus::decode(r)?,
            instructions: String::decode(r)?,
        })
    }
}

impl Decode for StorageData {
    fn decode<R: Read>(r: &mut R) -> VMResult<Self> {
        Ok(StorageData {
            tasks: Vec::decode(r)?,
            next_id: u32::decode(r)?,
        })
    }
}
