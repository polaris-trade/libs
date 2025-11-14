/// Default initial buffer capacity (8 KiB)
pub const DEFAULT_BUFFER_CAPACITY: usize = 8 * 1024;

/// Minimum spare capacity before reserving more (1 KiB)
pub const MIN_SPARE_CAPACITY: usize = 1024;

/// Maximum buffer capacity before shrinking (8 MB)
pub const MAX_BUFFER_CAPACITY: usize = 8 * 1024 * 1024;

/// SoupBinTCP packet length field size (u16)
pub const SOUPBINTCP_LENGTH_SIZE: usize = 2;

/// SoupBinTCP minimum packet header size (length + type)
pub const SOUPBINTCP_MIN_HEADER: usize = 3;

/// Default inactivity timeout for SoupBinTCP connections in seconds
pub const SOUPBINTCP_INACTIVITY_TIMEOUT_SECS: u64 = 15;

/// Default batch size for message extraction in MIO loop
pub const MIO_BATCH_SIZE: usize = 100;

/// MIO poll timeout in milliseconds
pub const MIO_POLL_TIMEOUT_MS: u64 = 10;

/// Maximum chunks per batch read
pub const BATCH_READ_MAX_CHUNKS: usize = 32;

/// Maximum bytes per batch read (64 KiB)
pub const BATCH_READ_MAX_BYTES: usize = 64 * 1024;

/// Default buffer pool capacity per buffer (2 KiB)
pub const BUFFER_POOL_CAPACITY: usize = 2048;

/// Maximum buffer pool size
pub const BUFFER_POOL_MAX_SIZE: usize = 1000;

/// Default max reconnection attempts
pub const DEFAULT_MAX_RECONNECT_ATTEMPTS: u32 = 5;

/// Default initial reconnection delay in milliseconds
pub const DEFAULT_RECONNECT_DELAY_MS: u64 = 1000;

/// Maximum reconnection delay in milliseconds (30 seconds)
pub const MAX_RECONNECT_DELAY_MS: u64 = 30000;

/// Default heartbeat interval in seconds
pub const DEFAULT_HEARTBEAT_INTERVAL_SECS: u64 = 5;
