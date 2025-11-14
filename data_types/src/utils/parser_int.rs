use crate::utils::{ParseResult, check_len};
use std::ptr;

#[inline(always)]
pub fn parse_i8(b: &[u8]) -> ParseResult<i8> {
    check_len(b, 1)?;
    Ok(b[0] as i8)
}

#[inline(always)]
pub fn parse_i16(b: &[u8]) -> ParseResult<i16> {
    check_len(b, 2)?;
    Ok(i16::from_be_bytes(
        b[0..2].try_into().expect("checked by check_len"),
    ))
}

#[inline(always)]
pub fn parse_i32(b: &[u8]) -> ParseResult<i32> {
    check_len(b, 4)?;
    Ok(i32::from_be_bytes(
        b[0..4].try_into().expect("checked by check_len"),
    ))
}

#[inline(always)]
pub fn parse_i64(b: &[u8]) -> ParseResult<i64> {
    check_len(b, 8)?;
    Ok(i64::from_be_bytes(
        b[0..8].try_into().expect("checked by check_len"),
    ))
}

//
// ====================
// Unsafe fast variants
// ====================
//

/// # Safety
/// The caller must ensure that `b` has at least 2 bytes.
#[inline(always)]
pub unsafe fn parse_i16_unsafe(b: &[u8]) -> i16 {
    unsafe { i16::from_be(ptr::read_unaligned(b.as_ptr() as *const i16)) }
}

/// # Safety
/// The caller must ensure that `b` has at least 4 bytes.
#[inline(always)]
pub unsafe fn parse_i32_unsafe(b: &[u8]) -> i32 {
    unsafe { i32::from_be(ptr::read_unaligned(b.as_ptr() as *const i32)) }
}

/// # Safety
/// The caller must ensure that `b` has at least 8 bytes.
#[inline(always)]
pub unsafe fn parse_i64_unsafe(b: &[u8]) -> i64 {
    unsafe { i64::from_be(ptr::read_unaligned(b.as_ptr() as *const i64)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_i8() {
        assert_eq!(parse_i8(&[0x7F]).unwrap(), 127);
        assert_eq!(parse_i8(&[0xFF]).unwrap(), -1);
    }

    #[test]
    fn test_parse_i16() {
        assert_eq!(parse_i16(&[0x12, 0x34]).unwrap(), 0x1234);
        assert_eq!(parse_i16(&[0xFF, 0xFE]).unwrap(), -2);
    }

    #[test]
    fn test_parse_i32() {
        assert_eq!(parse_i32(&[0x00, 0x00, 0x01, 0x00]).unwrap(), 256);
        assert_eq!(parse_i32(&[0xFF, 0xFF, 0xFF, 0xFF]).unwrap(), -1);
    }

    #[test]
    fn test_parse_i64() {
        assert_eq!(parse_i64(&[0, 0, 0, 0, 0, 0, 0, 1]).unwrap(), 1);
        assert_eq!(
            parse_i64(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]).unwrap(),
            -1
        );
    }

    #[test]
    fn test_parse_i16_unsafe() {
        let bytes = [0x12, 0x34];
        let val = unsafe { parse_i16_unsafe(&bytes) };
        assert_eq!(val, 0x1234);
    }

    #[test]
    fn test_parse_i32_unsafe() {
        let bytes = [0x00, 0x00, 0x01, 0x00];
        let val = unsafe { parse_i32_unsafe(&bytes) };
        assert_eq!(val, 256);
    }

    #[test]
    fn test_parse_i64_unsafe() {
        let bytes = [0, 0, 0, 0, 0, 0, 0, 1];
        let val = unsafe { parse_i64_unsafe(&bytes) };
        assert_eq!(val, 1);
    }
}
