// inspiration
// https://craftinginterpreters.com/optimization.html#nan-boxing
//
//

/*
idea
    0        -   11111111111 11   - 00000000000000000000000000000000 - 000000000000000 - 000
 SIGN_BIT(1) -    QNAN BITS(13)   -              PAYLOAD(32)         - UNUSED(15)      - TAG BITS(3)

TAGS:
NONE = 1 (reserved)
FALSE = 2
TRUE = 3
STRING_REF = 4
CALLDATA_REF = 5
U32 = 6


MEM_SLICE tuple (u25,u25)

    1        -   11111111111 11   - 0000000000000000000000000 - 0000000000000000000000000
 SIGN_BIT(1) -    QNAN BITS(13)   -         OFFSET(25)        -         SIZE(25)

*/

use crate::errors::{VMError, VMResult};

pub type Value = u64;

const U25_MAX: u32 = 1 << 25;
// quite NaN - if all those bits set => NaN tagged value of some type
// 0111111111111100000000000000000000000000000000000000000000000000
const QNAN: u64 = 0x7ffc000000000000;
const SIGN_BIT: u64 = 0x8000000000000000;
const TAG_MASK: u64 = 0b111;
//reserve 1 tag for none
const TAG_FALSE: u64 = 2;
const TAG_TRUE: u64 = 3;
const TAG_STRING: u64 = 4;
const TAG_CALLDATA: u64 = 5;
const TAG_U32: u64 = 6;
pub(crate) const FALSE_VAL: Value = QNAN | (TAG_FALSE);
pub(crate) const TRUE_VAL: Value = QNAN | (TAG_TRUE);

// mem_slice tuple type that is (u25,u25)
//
#[inline]
pub(crate) const fn to_mem_slice_val(offset: u32, size: u32) -> VMResult<Value> {
    // u25 max check
    if offset < U25_MAX && size < U25_MAX {
        Ok((SIGN_BIT | QNAN) | ((offset as u64) << 25) | (size as u64))
    } else {
        Err(VMError::InvalidType)
    }
}

//shifting 18 bits (unused(15) + tag (3))
#[inline]
const fn make_tagged(tag: u64, payload: u32) -> Value {
    QNAN | ((payload as u64) << 18) | tag
}

#[inline]
pub const fn to_string_val(idx: u32) -> Value {
    make_tagged(TAG_STRING, idx)
}

#[inline]
pub(crate) const fn to_u32_val(idx: u32) -> Value {
    make_tagged(TAG_U32, idx)
}

#[inline]
pub(crate) const fn to_calldata_val(idx: u32) -> Value {
    make_tagged(TAG_CALLDATA, idx)
}

const fn to_bool_val(b: bool) -> Value {
    if b { TRUE_VAL } else { FALSE_VAL }
}

// check if tagged

#[inline]
const fn is_mem_slice(v: Value) -> bool {
    (v) & (QNAN | SIGN_BIT) == (QNAN | SIGN_BIT)
}

#[inline]
const fn is_qnan(v: Value) -> bool {
    (v & QNAN) == QNAN
}

const fn raw_tag(v: Value) -> u64 {
    v & TAG_MASK
}

#[inline]
const fn tag(v: Value) -> VMResult<u64> {
    if is_qnan(v) {
        Ok(v & TAG_MASK)
    } else {
        Err(VMError::InvalidType)
    }
}

#[inline]
const fn bool_not(v: Value) -> Value {
    v ^ 1
}

// keep for now
// converting last bit to 1 for FALSE_VAL makes it TRUE_VAL , notthing for TRUE_VAL. If resulting val != TRUE_VAL => not a boolean
const fn is_bool_val(v: Value) -> bool {
    is_qnan(v) && (v | 1) == TRUE_VAL
}

// keep for now
#[inline]
const fn is_string_val(v: Value) -> bool {
    is_qnan(v) && raw_tag(v) == TAG_STRING
}
// keep for now
#[inline]
const fn is_calldata_val(v: Value) -> bool {
    is_qnan(v) && raw_tag(v) == TAG_CALLDATA
}

#[inline]
const fn is_u32_val(v: Value) -> bool {
    is_qnan(v) && raw_tag(v) == TAG_U32
}

// u32_val & string_val & calldata_val all u32
// potentially risky, need to add validation
// shouldn't be accesible - wrong conversions are possible - ie nan boxed TRUE_VAL and FALSE_VAL both returns 0
#[inline]
pub(crate) const fn to_u32(v: Value) -> u32 {
    ((v >> 18) & 0xffff_ffff) as u32
    // unused 15 bits  + tag bits 3
}

#[inline]
pub(crate) const fn to_mem_slice(v: Value) -> VMResult<(u32, u32)> {
    if is_mem_slice(v) {
        let offset = ((v >> 25) & 0x1FFFFFF) as u32;
        let size = (v & 0x1FFFFFF) as u32;
        Ok((offset, size))
    } else {
        Err(VMError::InvalidType)
    }
}

pub(crate) fn value_eq(left: Value, right: Value) -> VMResult<Value> {
    let tag_left = tag(left)?;
    let tag_right = tag(right)?;
    if tag_left != tag_right {
        Err(VMError::TypeMismatch)
    } else {
        match tag_left {
            TAG_U32 | TAG_STRING | TAG_CALLDATA => Ok(to_bool_val(to_u32(left) == to_u32(right))),
            TAG_TRUE | TAG_FALSE => Ok(to_bool_val(left == right)),
            _ => Err(VMError::InvalidType),
        }
    }
}

pub(crate) fn value_neq(left: Value, right: Value) -> VMResult<Value> {
    Ok(bool_not(value_eq(left, right)?))
}

//comparing u32_val only for now
pub(crate) fn value_cmp(left: Value, right: Value, is_lt: bool) -> VMResult<Value> {
    if !is_u32_val(left) || !is_u32_val(right) {
        return Err(VMError::InvalidType);
    }
    match is_lt {
        true => Ok(to_bool_val(to_u32(left) < to_u32(right))),
        false => Ok(to_bool_val(to_u32(left) > to_u32(right))),
    }
}

// unboxing Values

pub(crate) enum ValueType {
    U32,
    String,
    CallData,
    Bool,
    MemSlice,
}

pub(crate) fn get_value_type(nan_boxed_val: Value) -> VMResult<ValueType> {
    if is_mem_slice(nan_boxed_val) {
        return Ok(ValueType::MemSlice);
    }
    match tag(nan_boxed_val)? {
        TAG_U32 => Ok(ValueType::U32),
        TAG_STRING => Ok(ValueType::String),
        TAG_CALLDATA => Ok(ValueType::CallData),
        TAG_FALSE | TAG_TRUE => Ok(ValueType::Bool),
        _ => Err(VMError::InvalidType),
    }
}

#[derive(Debug)]
pub enum Return<'a> {
    U32(u32),
    String(&'a str),
    CallData(String),
    Bool(bool),
    MemSlice(u32, u32),
}

impl<'a> Return<'a> {
    pub fn as_u32(&self) -> VMResult<u32> {
        match self {
            Return::U32(v) => Ok(*v),
            _ => Err(VMError::TypeMismatch),
        }
    }
    pub fn as_str(&self) -> VMResult<&'a str> {
        match self {
            Return::String(s) => Ok(*s),
            _ => Err(VMError::TypeMismatch),
        }
    }

    pub fn as_calldata(&self) -> VMResult<&str> {
        match self {
            Return::CallData(c) => Ok(c),
            _ => Err(VMError::TypeMismatch),
        }
    }

    pub fn as_bool(&self) -> VMResult<bool> {
        match self {
            Return::Bool(b) => Ok(*b),
            _ => Err(VMError::TypeMismatch),
        }
    }

    pub fn as_mem_slice(&self) -> VMResult<(u32, u32)> {
        match self {
            Return::MemSlice(o, s) => Ok((*o, *s)),
            _ => Err(VMError::TypeMismatch),
        }
    }
}
