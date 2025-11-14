use crate::{
    result::ParseResult,
    time::{DateTimeUtc, ElapsedNanos, JAKARTA_OFFSET, NANO_PER_SEC, second::UnixSeconds},
    utils::parser_uint,
};
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::ops::Add;

/// Unix Timestamp in nanoseconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UnixNanoseconds(pub u64);

impl TryFrom<UnixSeconds> for UnixNanoseconds {
    type Error = &'static str;

    /// Try to convert [`UnixSeconds`] to [`UnixNanoseconds`], checking for overflow
    #[inline]
    fn try_from(s: UnixSeconds) -> Result<Self, Self::Error> {
        s.0.checked_mul(NANO_PER_SEC)
            .map(UnixNanoseconds)
            .ok_or("seconds * 1_000_000_000 overflowed u64")
    }
}

impl From<u64> for UnixNanoseconds {
    #[inline]
    fn from(ns: u64) -> Self {
        UnixNanoseconds(ns)
    }
}

impl From<u32> for UnixNanoseconds {
    #[inline]
    fn from(ns: u32) -> Self {
        UnixNanoseconds(ns as u64)
    }
}

impl Add<ElapsedNanos> for UnixNanoseconds {
    type Output = UnixNanoseconds;

    #[inline]
    fn add(self, nanos: ElapsedNanos) -> Self::Output {
        UnixNanoseconds(self.0 + nanos.0 as u64)
    }
}

impl Add<u32> for UnixNanoseconds {
    type Output = UnixNanoseconds;

    #[inline]
    fn add(self, rhs: u32) -> Self::Output {
        UnixNanoseconds(self.0 + rhs as u64)
    }
}

/// Add u64 nanoseconds directly (for backward compatibility)
impl Add<u64> for UnixNanoseconds {
    type Output = UnixNanoseconds;

    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        UnixNanoseconds(self.0 + rhs)
    }
}

impl UnixNanoseconds {
    #[inline]
    pub fn from_seconds_checked(seconds: u64) -> Result<Self, &'static str> {
        UnixSeconds(seconds).try_into()
    }

    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> ParseResult<Self> {
        parser_uint::parse_u64(bytes).map(UnixNanoseconds)
    }

    #[inline]
    pub fn from_bytes_u32(bytes: &[u8]) -> ParseResult<Self> {
        parser_uint::parse_u32(bytes).map(|val| UnixNanoseconds(val as u64))
    }

    /// Convert into `DateTime<Utc>`
    #[inline]
    pub fn to_utc(&self) -> DateTimeUtc {
        let secs = (self.0 / NANO_PER_SEC) as i64;
        let nsec = (self.0 % NANO_PER_SEC) as u32;
        Utc.timestamp_opt(secs, nsec)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn try_from_seconds_ok() {
        let s = UnixSeconds(1);
        let ns = UnixNanoseconds::try_from(s).unwrap();
        assert_eq!(ns.0, 1_000_000_000);
    }

    #[test]
    fn try_from_seconds_overflow() {
        let seconds = 18_446_744_074u64; // exceeds u64::MAX / 1_000_000_000
        assert!(UnixNanoseconds::from_seconds_checked(seconds).is_err());
    }

    #[test]
    fn to_utc_and_local() {
        let ns = UnixNanoseconds::try_from(UnixSeconds(0)).unwrap();
        let dt_utc = ns.to_utc();
        assert_eq!(dt_utc, Utc.timestamp_opt(0, 0).unwrap());

        let dt_local = ns.to_local();
        let expected = Utc
            .timestamp_opt(0, 0)
            .unwrap()
            .with_timezone(&JAKARTA_OFFSET);
        assert_eq!(dt_local, expected);
    }

    #[test]
    fn to_iso8601() {
        let ns = UnixNanoseconds::try_from(UnixSeconds(0)).unwrap();
        let iso_str = ns.to_iso8601();
        assert_eq!(iso_str, "1970-01-01T07:00:00+07:00");
    }

    #[test]
    fn from_u32() {
        let ns = UnixNanoseconds::from(1234567890u32);
        assert_eq!(ns.0, 1234567890u64);
    }

    #[test]
    fn from_u64() {
        let ns = UnixNanoseconds::from(1234567890123456789u64);
        assert_eq!(ns.0, 1234567890123456789u64);
    }

    #[test]
    fn from_bytes_u64() {
        let bytes = [0x00, 0x00, 0x00, 0x00, 0x3B, 0x9A, 0xCA, 0x00]; // 1_000_000_000
        let ns = UnixNanoseconds::from_bytes(&bytes).unwrap();
        assert_eq!(ns.0, 1_000_000_000);
    }

    #[test]
    fn from_bytes_u32() {
        let bytes = [0x3B, 0x9A, 0xCA, 0x00]; // 1_000_000_000
        let ns = UnixNanoseconds::from_bytes_u32(&bytes).unwrap();
        assert_eq!(ns.0, 1_000_000_000);
    }

    #[test]
    fn from_bytes_insufficient_data() {
        let bytes = [0x3B, 0x9A, 0xCA];
        assert!(UnixNanoseconds::from_bytes(&bytes).is_err());
        assert!(UnixNanoseconds::from_bytes_u32(&bytes).is_err());
    }

    #[test]
    fn from_bytes_edge_cases() {
        let zero_bytes = [0u8; 8];
        let ns = UnixNanoseconds::from_bytes(&zero_bytes).unwrap();
        assert_eq!(ns.0, 0);

        let max_u32_bytes = [0xFF, 0xFF, 0xFF, 0xFF];
        let ns = UnixNanoseconds::from_bytes_u32(&max_u32_bytes).unwrap();
        assert_eq!(ns.0, u32::MAX as u64);

        let max_u64_bytes = [0xFF; 8];
        let ns = UnixNanoseconds::from_bytes(&max_u64_bytes).unwrap();
        assert_eq!(ns.0, u64::MAX);
    }

    #[test]
    fn test_add_elapsed_nanos() {
        use crate::time::ElapsedNanos;

        let base = UnixNanoseconds(1_000_000_000);
        let elapsed = ElapsedNanos(123456789);
        let result = base + elapsed;
        assert_eq!(result.0, 1_123_456_789);
    }

    #[test]
    fn test_add_u32() {
        let base = UnixNanoseconds(1_000_000_000);
        let result = base + 500_000_000u32;
        assert_eq!(result.0, 1_500_000_000);
    }

    #[test]
    fn test_add_u64() {
        let base = UnixNanoseconds(1_000_000_000);
        let result = base + 2_000_000_000u64;
        assert_eq!(result.0, 3_000_000_000);
    }

    #[test]
    fn test_seconds_to_nanos_with_elapsed() {
        use crate::time::{ElapsedNanos, UnixSeconds};

        // Simulating ITCH message timestamp calculation
        let last_seconds = UnixSeconds(1700000000);
        let base_nanos = UnixNanoseconds::try_from(last_seconds).unwrap();
        let elapsed = ElapsedNanos(999999999);
        let final_timestamp = base_nanos + elapsed;

        assert_eq!(final_timestamp.0, 1700000000 * NANO_PER_SEC + 999999999);
    }
}
