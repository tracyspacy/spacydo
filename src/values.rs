// inspiration
// https://craftinginterpreters.com/optimization.html#nan-boxing
//
//

/*
idea
    0        -   11111111111 11   - 00000000000000000000000000000000 - 000000000000000 - 000
 SIGN_BIT(1) -    QNAN BITS(13)   -              PAYLOAD(32)         - UNUSED(15)      - TAG BITS(3)

TAGS:
NULL = 1
FALSE = 2
TRUE = 3
STRING = 4
CALLDATA_REF = 5
U32 = 6


//vector and tag represents what exactly is stored there like Vec<u8> | Vec<u32> |

    1        -   11111111111 11   - 0000000000000000000000000 - 0000000000000000 -  000000   - 100
 SIGN_BIT(1) -    QNAN BITS(13)   -         OFFSET(25)        -     SIZE(16)     - UNUSED(6) - TAG BITS(3)

    1        -   11111111111 11   - 0000000000000000000000000 - 0000000000000000 -  000000   - 110
 SIGN_BIT(1) -    QNAN BITS(13)   -         OFFSET(25)        -     SIZE(16)     - UNUSED(6) - TAG BITS(3)


*/

use crate::errors::{VMError, VMResult};

pub type Value = u64;

const U25_MAX: u32 = 1 << 25;
// quite NaN - if all those bits set => NaN tagged value of some type
// 0111111111111100000000000000000000000000000000000000000000000000
const QNAN: u64 = 0x7ffc000000000000;
const SIGN_BIT: u64 = 0x8000000000000000;
const TAG_MASK: u64 = 0b111;
const TAG_NULL: u64 = 1;
const TAG_FALSE: u64 = 2;
const TAG_TRUE: u64 = 3;
pub(crate) const TAG_STRING: u64 = 4;
const TAG_CALLDATA: u64 = 5;
pub(crate) const TAG_U32: u64 = 6;
pub(crate) const FALSE_VAL: Value = QNAN | (TAG_FALSE);
pub(crate) const TRUE_VAL: Value = QNAN | (TAG_TRUE);
pub(crate) const NULL_VAL: Value = QNAN | (TAG_NULL);

// fat poiner to a vec (offset , size)
#[inline]
const fn to_vec_val(offset: u32, size: u16, type_tag: u64) -> VMResult<Value> {
    if offset < U25_MAX {
        Ok((SIGN_BIT | QNAN) | ((offset as u64) << 25) | ((size as u64) << 9) | (type_tag))
    } else {
        Err(VMError::MSliceParamOverflow)
    }
}

#[inline]
pub(crate) const fn to_string_vec_val(offset: u32, size: u16) -> VMResult<Value> {
    to_vec_val(offset, size, TAG_STRING)
}

#[inline]
pub(crate) const fn to_u32_vec_val(offset: u32, size: u16) -> VMResult<Value> {
    to_vec_val(offset, size, TAG_U32)
}

//shifting 18 bits (unused(15) + tag (3))
#[inline]
const fn make_scalar_tagged(tag: u64, payload: u32) -> Value {
    QNAN | ((payload as u64) << 18) | tag
}

#[inline]
pub(crate) const fn to_u32_val(idx: u32) -> Value {
    make_scalar_tagged(TAG_U32, idx)
}

#[inline]
pub(crate) const fn to_calldata_val(idx: u32) -> Value {
    make_scalar_tagged(TAG_CALLDATA, idx)
}

pub(crate) const fn to_bool_val(b: bool) -> Value {
    if b { TRUE_VAL } else { FALSE_VAL }
}

#[inline]
const fn is_qnan(v: Value) -> bool {
    (v & QNAN) == QNAN
}

#[inline]
const fn is_sign_bit(v: Value) -> bool {
    (v & SIGN_BIT) == SIGN_BIT
}

#[inline]
const fn is_scalar(v: Value) -> bool {
    !is_sign_bit(v) && is_qnan(v)
}

#[inline]
pub(crate) const fn is_vec(v: Value) -> bool {
    is_sign_bit(v) && is_qnan(v)
}

/* hiding for now
 #[inline]
pub(crate) const fn is_string_vec(v: Value) -> bool {
    is_vec(v) && raw_tag(v) == TAG_STRING
}

#[inline]
pub(crate) const fn is_u32_vec(v: Value) -> bool {
    is_vec(v) && raw_tag(v) == TAG_U32
} */

#[inline]
const fn raw_tag(v: Value) -> u64 {
    v & TAG_MASK
}

#[inline]
pub(crate) const fn tag(v: Value) -> VMResult<u64> {
    if is_qnan(v) {
        Ok(v & TAG_MASK)
    } else {
        Err(VMError::InvalidType)
    }
}

#[inline]
pub(crate) const fn bool_not(v: Value) -> Value {
    v ^ 1
}

#[inline]
const fn is_u32_val(v: Value) -> bool {
    is_scalar(v) && raw_tag(v) == TAG_U32
}

// u32_val & string_val & calldata_val all u32
// potentially risky, need to add validation
// shouldn't be accesible - wrong conversions are possible - ie nan boxed TRUE_VAL and FALSE_VAL both returns 0
#[inline]
pub(crate) const fn to_u32(v: Value) -> u32 {
    ((v >> 18) & 0xffff_ffff) as u32
    // unused 15 bits  + tag bits 3
}

//returns tuple u32 - offset u16- size
#[inline]
pub const fn to_fat_pointer(v: Value) -> VMResult<(u32, u16)> {
    if !is_vec(v) {
        return Err(VMError::InvalidType);
    }
    let offset = ((v >> 25) & 0x1FFFFFF) as u32;
    let size = ((v >> 9) & 0xFFFF) as u16;
    Ok((offset, size))
}

pub(crate) fn value_eq(left: Value, right: Value) -> VMResult<Value> {
    let tag_left = tag(left)?;
    let tag_right = tag(right)?;
    if tag_left != tag_right {
        Err(VMError::TypeMismatch)
    } else {
        match tag_left {
            TAG_U32 | TAG_CALLDATA => Ok(to_bool_val(to_u32(left) == to_u32(right))),
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
    String, // basicaly vec, probably rename
    CallData,
    Bool,
    VecU32,
    Null,
}

pub(crate) fn get_value_type(nan_boxed_val: Value) -> VMResult<ValueType> {
    if is_vec(nan_boxed_val) {
        return match raw_tag(nan_boxed_val) {
            TAG_U32 => Ok(ValueType::VecU32),
            TAG_STRING => Ok(ValueType::String),
            _ => Err(VMError::InvalidType),
        };
    }
    if !is_scalar(nan_boxed_val) {
        return Err(VMError::InvalidType);
    }

    match raw_tag(nan_boxed_val) {
        TAG_U32 => Ok(ValueType::U32),
        TAG_STRING => Ok(ValueType::String),
        TAG_CALLDATA => Ok(ValueType::CallData),
        TAG_FALSE | TAG_TRUE => Ok(ValueType::Bool),
        TAG_NULL => Ok(ValueType::Null),
        _ => Err(VMError::InvalidType),
    }
}

#[derive(Debug, PartialEq)]
pub enum Return<'a> {
    U32(u32),
    String(&'a str),
    CallData(&'a [u8]),
    Bool(bool),
    VecU32(&'a [u32]),
    Null,
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

    pub fn as_calldata(&self) -> VMResult<&[u8]> {
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

    pub fn as_vec_u32(&self) -> VMResult<&[u32]> {
        match self {
            Return::VecU32(v) => Ok(*v),
            _ => Err(VMError::TypeMismatch),
        }
    }
}
