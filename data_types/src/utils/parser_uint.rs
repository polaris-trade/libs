use crate::utils::{ParseResult, check_len};
use std::ptr;

#[inline(always)]
pub fn parse_u8(b: &[u8]) -> ParseResult<u8> {
    check_len(b, 1)?;
    Ok(b[0])
}

#[inline(always)]
pub fn parse_u16(b: &[u8]) -> ParseResult<u16> {
    check_len(b, 2)?;
    Ok(u16::from_be_bytes(
        b[0..2].try_into().expect("checked by check_len"),
    ))
}

#[inline(always)]
pub fn parse_u32(b: &[u8]) -> ParseResult<u32> {
    check_len(b, 4)?;
    Ok(u32::from_be_bytes(
        b[0..4].try_into().expect("checked by check_len"),
    ))
}

#[inline(always)]
pub fn parse_u64(b: &[u8]) -> ParseResult<u64> {
    check_len(b, 8)?;
    Ok(u64::from_be_bytes(
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
pub unsafe fn parse_u16_unsafe(b: &[u8]) -> u16 {
    unsafe { u16::from_be(ptr::read_unaligned(b.as_ptr() as *const u16)) }
}

/// # Safety
/// The caller must ensure that `b` has at least 4 bytes.
#[inline(always)]
pub unsafe fn parse_u32_unsafe(b: &[u8]) -> u32 {
    unsafe { u32::from_be(ptr::read_unaligned(b.as_ptr() as *const u32)) }
}

/// # Safety
/// The caller must ensure that `b` has at least 8 bytes.
#[inline(always)]
pub unsafe fn parse_u64_unsafe(b: &[u8]) -> u64 {
    unsafe { u64::from_be(ptr::read_unaligned(b.as_ptr() as *const u64)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_u8() {
        assert_eq!(parse_u8(&[0x7F]).unwrap(), 127);
        assert_eq!(parse_u8(&[0xFF]).unwrap(), 255);
    }

    #[test]
    fn test_parse_u16() {
        assert_eq!(parse_u16(&[0x12, 0x34]).unwrap(), 0x1234);
        assert_eq!(parse_u16(&[0xFF, 0xFE]).unwrap(), 0xFFFE);
    }

    #[test]
    fn test_parse_u32() {
        assert_eq!(parse_u32(&[0x00, 0x00, 0x01, 0x00]).unwrap(), 256);
        assert_eq!(parse_u32(&[0xFF, 0xFF, 0xFF, 0xFF]).unwrap(), 0xFFFFFFFF);
    }

    #[test]
    fn test_parse_u64() {
        assert_eq!(parse_u64(&[0, 0, 0, 0, 0, 0, 0, 1]).unwrap(), 1);
        assert_eq!(
            parse_u64(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]).unwrap(),
            0xFFFFFFFFFFFFFFFF
        );
    }

    #[test]
    fn test_parse_u16_unsafe() {
        let bytes = [0x12, 0x34];
        let val = unsafe { parse_u16_unsafe(&bytes) };
        assert_eq!(val, 0x1234);
    }

    #[test]
    fn test_parse_u32_unsafe() {
        let bytes = [0x00, 0x00, 0x01, 0x00];
        let val = unsafe { parse_u32_unsafe(&bytes) };
        assert_eq!(val, 256);
    }

    #[test]
    fn test_parse_u64_unsafe() {
        let bytes = [0, 0, 0, 0, 0, 0, 0, 1];
        let val = unsafe { parse_u64_unsafe(&bytes) };
        assert_eq!(val, 1);
    }
}
