use serde::{Deserialize, Serialize};

/// Elapsed nanoseconds since the last [`super::UnixSeconds`] message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ElapsedNanos(pub u32);

impl From<u32> for ElapsedNanos {
    #[inline]
    fn from(nanos: u32) -> Self {
        ElapsedNanos(nanos)
    }
}

impl From<ElapsedNanos> for u64 {
    #[inline]
    fn from(elapsed: ElapsedNanos) -> Self {
        elapsed.0 as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_u32() {
        let elapsed = ElapsedNanos::from(123456789u32);
        assert_eq!(elapsed.0, 123456789);
    }

    #[test]
    fn test_to_u64() {
        let elapsed = ElapsedNanos(999999999);
        let as_u64: u64 = elapsed.into();
        assert_eq!(as_u64, 999999999u64);
    }

    #[test]
    fn test_ordering() {
        let a = ElapsedNanos(100);
        let b = ElapsedNanos(200);
        assert!(a < b);
        assert_eq!(a, ElapsedNanos(100));
    }
}
