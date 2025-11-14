use crate::{ParseError, ParseResult, utils::parser_int::parse_i32};
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// A Date represented as a 4-byte unsigned integer in YYYYMMDD format
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Date(pub u32);

impl Date {
    /// Convert to [chrono::NaiveDate]
    #[inline(always)]
    pub fn to_naive_date(&self) -> Option<NaiveDate> {
        let year = (self.0 / 10_000) as i32;
        let month = (self.0 / 100) % 100;
        let day = self.0 % 100;
        NaiveDate::from_ymd_opt(year, month, day)
    }

    /// Encode back to bytes (big-endian)
    #[inline(always)]
    pub fn to_bytes(&self) -> [u8; 4] {
        self.0.to_be_bytes()
    }

    /// # Safety
    /// The caller must ensure that `bytes` has at least 4 bytes.
    /// No bounds checks are performed. Use only in hot paths with pre-validated slices.
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> Self {
        unsafe {
            let arr: [u8; 4] = *(bytes.as_ptr() as *const [u8; 4]);
            Self(u32::from_be_bytes(arr))
        }
    }
}

impl TryFrom<&[u8]> for Date {
    type Error = ParseError;

    #[inline(always)]
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let raw = parse_i32(bytes)?;
        Ok(Self(raw as u32))
    }
}

impl From<u32> for Date {
    #[inline(always)]
    fn from(raw: u32) -> Self {
        Self(raw)
    }
}

impl From<Date> for u32 {
    #[inline(always)]
    fn from(date: Date) -> Self {
        date.0
    }
}

impl From<Date> for [u8; 4] {
    #[inline(always)]
    fn from(date: Date) -> Self {
        date.to_bytes()
    }
}

impl From<NaiveDate> for Date {
    #[inline(always)]
    fn from(date: NaiveDate) -> Self {
        let raw = (date.year() as u32) * 10_000 + date.month() * 100 + date.day();
        Self(raw)
    }
}

impl TryFrom<Date> for NaiveDate {
    type Error = ParseError;
    #[inline(always)]
    fn try_from(date: Date) -> ParseResult<NaiveDate> {
        date.to_naive_date().ok_or(ParseError::InvalidDate)
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.to_naive_date() {
            Some(d) => write!(f, "{}", d.format("%Y-%m-%d")),
            None => write!(f, "Invalid({})", self.0),
        }
    }
}

#[cfg(test)]
mod extra_tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_display() {
        let valid_date = Date(20251024);
        assert_eq!(valid_date.to_string(), "2025-10-24");

        let invalid_date = Date(20251340);
        assert_eq!(invalid_date.to_string(), "Invalid(20251340)");
    }

    #[test]
    fn test_from_u32_and_into_u32() {
        let raw: u32 = 20251024;
        let date: Date = raw.into();
        assert_eq!(date.0, raw);

        let back: u32 = date.into();
        assert_eq!(back, raw);
    }

    #[test]
    fn test_from_bytes_and_to_bytes_roundtrip() {
        let raw: u32 = 20251024;
        let date: Date = raw.into();
        let bytes: [u8; 4] = date.into();
        assert_eq!(bytes, raw.to_be_bytes());

        let reconstructed = unsafe { Date::from_bytes_unchecked(&bytes) };
        assert_eq!(reconstructed.0, raw);
    }

    #[test]
    fn test_from_naive_date() {
        let nd = NaiveDate::from_ymd_opt(2025, 10, 24).unwrap();
        let date: Date = nd.into();
        assert_eq!(date.0, 20251024);
    }

    #[test]
    fn test_try_from_date_to_naive_date() {
        let date = Date(20251024);
        let nd: NaiveDate = date.try_into().unwrap();
        assert_eq!(nd, NaiveDate::from_ymd_opt(2025, 10, 24).unwrap());

        let invalid_date = Date(20251340);
        let res: Result<NaiveDate, _> = invalid_date.try_into();
        assert!(res.is_err());
    }

    #[test]
    fn test_try_from_valid_bytes() {
        let bytes = 20251024u32.to_be_bytes();
        let date = Date::try_from(&bytes[..]).unwrap();
        assert_eq!(date.0, 20251024);
    }

    #[test]
    fn test_try_from_short_bytes() {
        let bytes = [0x01, 0x02];
        let result = Date::try_from(&bytes[..]);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(
            err,
            ParseError::Incomplete { .. } | ParseError::InvalidValue
        ));
    }

    #[test]
    fn test_try_from_arbitrary_4bytes() {
        let bytes = b"abcd";
        let date = Date::try_from(&bytes[..]).unwrap();
        let expected = i32::from_be_bytes(*bytes) as u32;
        assert_eq!(date.0, expected);
    }

    #[test]
    fn test_try_from_edge_cases() {
        let bytes = i32::MAX.to_be_bytes();
        let date = Date::try_from(&bytes[..]).unwrap();
        assert_eq!(date.0, i32::MAX as u32);

        let bytes = i32::MIN.to_be_bytes();
        let date = Date::try_from(&bytes[..]).unwrap();
        assert_eq!(date.0, i32::MIN as u32);
    }

    #[test]
    fn test_try_from_4bytes_any_value() {
        let bytes = b"abcd";
        let result = Date::try_from(&bytes[..]);
        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date.0, i32::from_be_bytes(*bytes) as u32);
    }
}
