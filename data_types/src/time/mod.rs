use chrono::{FixedOffset, Utc};

pub const NANO_PER_SEC: u64 = 1_000_000_000;
pub const JAKARTA_OFFSET: FixedOffset = FixedOffset::east_opt(7 * 3600).expect("Invalid offset.");

pub type DateTimeUtc = chrono::DateTime<Utc>;
pub mod date;
pub use date::Date;
pub mod nanosecond;
pub use nanosecond::UnixNanoseconds;
pub mod second;
pub use second::UnixSeconds;
pub mod elapsed_nanos;
pub use elapsed_nanos::ElapsedNanos;

/// Re-export commonly used time types
pub mod prelude {
    pub use super::{
        date::Date, elapsed_nanos::ElapsedNanos, nanosecond::UnixNanoseconds, second::UnixSeconds,
    };
}
