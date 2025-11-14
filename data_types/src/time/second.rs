use crate::{
    result::ParseResult,
    time::{DateTimeUtc, JAKARTA_OFFSET, NANO_PER_SEC, nanosecond::UnixNanoseconds},
    utils::parser_uint,
};
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// Unix Timestamp in seconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UnixSeconds(pub u64);

/// Convert Nanoseconds back to Seconds (truncates the fractional part)
impl From<UnixNanoseconds> for UnixSeconds {
    #[inline]
    fn from(ns: UnixNanoseconds) -> Self {
        UnixSeconds(ns.0 / NANO_PER_SEC)
    }
}

impl From<u32> for UnixSeconds {
    #[inline]
    fn from(secs: u32) -> Self {
        UnixSeconds(secs as u64)
    }
}

impl From<u64> for UnixSeconds {
    #[inline]
    fn from(secs: u64) -> Self {
        UnixSeconds(secs)
    }
}

impl UnixSeconds {
    /// Parse seconds from 8 bytes (safe version)
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> ParseResult<Self> {
        parser_uint::parse_u64(bytes).map(UnixSeconds)
    }

    /// Parse seconds from 4 bytes (safe version) - extends to u64
    #[inline]
    pub fn from_bytes_u32(bytes: &[u8]) -> ParseResult<Self> {
        parser_uint::parse_u32(bytes).map(|val| UnixSeconds(val as u64))
    }

    /// Convert into `DateTime<Utc>`
    #[inline]
    pub fn to_utc(&self) -> DateTimeUtc {
        let secs = self.0 as i64;
        Utc.timestamp_opt(secs, 0)
            .single()
            .expect("valid timestamp")
    }

    /// Convert to Jakarta fixed offset time
    #[inline]
    pub fn to_local(&self) -> DateTime<FixedOffset> {
        self.to_utc().with_timezone(&JAKARTA_OFFSET)
    }

    /// ISO8601 string in local timezone
    #[inline]
    pub fn to_iso8601(&self) -> String {
        self.to_local().to_rfc3339()
    }

    /// Convert into Nanoseconds (checked)
    #[inline]
    pub fn to_nanoseconds(&self) -> Result<UnixNanoseconds, &'static str> {
        (*self).try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::nanosecond::UnixNanoseconds;
    use chrono::{Datelike, Timelike};

    #[test]
    fn test_from_u32() {
        let secs = UnixSeconds::from(1_u32);
        assert_eq!(secs.0, 1_u64);
    }

    #[test]
    fn test_from_u64() {
        let secs = UnixSeconds::from(42_u64);
        assert_eq!(secs.0, 42_u64);
    }

    #[test]
    fn test_from_nanoseconds() {
        let ns = UnixNanoseconds(1_500_000_000);
        let secs: UnixSeconds = ns.into();
        assert_eq!(secs.0, 1); // truncates fractional part
    }

    #[test]
    fn test_to_utc() {
        let secs = UnixSeconds::from(1_000_000_000_u64);
        let dt_utc = secs.to_utc();
        assert_eq!(dt_utc.timestamp(), 1_000_000_000);
    }

    #[test]
    fn test_to_local() {
        let secs = UnixSeconds::from(1_000_000_000_u64);
        let dt_local = secs.to_local();
        assert_eq!(dt_local.offset().local_minus_utc(), 7 * 3600);
    }

    #[test]
    fn test_to_iso8601() {
        let secs = UnixSeconds::from(1_000_000_000_u64);
        let iso = secs.to_iso8601();
        assert!(iso.contains("+07:00"));
    }

    #[test]
    fn test_to_nanoseconds() {
        let secs = UnixSeconds::from(2_u64);
        let ns = secs.to_nanoseconds().unwrap();
        assert_eq!(ns.0, 2 * NANO_PER_SEC);
    }

    #[test]
    fn test_from_bytes_u64() {
        let bytes = [0x00, 0x00, 0x00, 0x00, 0x3B, 0x9A, 0xCA, 0x00];
        let secs = UnixSeconds::from_bytes(&bytes).unwrap();
        assert_eq!(secs.0, 1_000_000_000);
    }

    #[test]
    fn test_from_bytes_u32() {
        let bytes = [0x3B, 0x9A, 0xCA, 0x00];
        let secs = UnixSeconds::from_bytes_u32(&bytes).unwrap();
        assert_eq!(secs.0, 1_000_000_000);
    }

    #[test]
    fn test_from_bytes_insufficient_data() {
        let bytes = [0x3B, 0x9A, 0xCA];
        assert!(UnixSeconds::from_bytes(&bytes).is_err());
        assert!(UnixSeconds::from_bytes_u32(&bytes).is_err());
    }

    #[test]
    fn test_from_bytes_edge_cases() {
        let zero_bytes = [0u8; 8];
        let secs = UnixSeconds::from_bytes(&zero_bytes).unwrap();
        assert_eq!(secs.0, 0);

        let max_u32_bytes = [0xFF, 0xFF, 0xFF, 0xFF];
        let secs = UnixSeconds::from_bytes_u32(&max_u32_bytes).unwrap();
        assert_eq!(secs.0, u32::MAX as u64);

        let max_u64_bytes = [0xFF; 8];
        let secs = UnixSeconds::from_bytes(&max_u64_bytes).unwrap();
        assert_eq!(secs.0, u64::MAX);
    }

    #[test]
    fn test_from_bytes_datetime_conversion() {
        let timestamp_secs = 946684800u64; // Jan 1, 2000 00:00:00 UTC
        let bytes = timestamp_secs.to_be_bytes();

        let secs = UnixSeconds::from_bytes(&bytes).unwrap();
        let dt_utc = secs.to_utc();
        let dt_local = secs.to_local();

        assert_eq!(secs.0, timestamp_secs);
        assert_eq!(dt_utc.year(), 2000);
        assert_eq!(dt_local.year(), 2000);
        assert_eq!(dt_local.hour(), 7);
    }
}
