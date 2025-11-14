use bytes::BytesMut;
use data_types::tracing::TraceData;
use tokio::io;

/// Read buffer type for network I/O accumulation.
pub type ReadBuffer = BytesMut;

/// Transport abstraction for different I/O implementations (MIO, Tokio, io_uring, DPDK, etc.)
///
/// Uses BytesMut for efficient network accumulation. The transport does NOT perform any
/// framing - it provides raw byte I/O. Application protocols (like SoupBinTCP) must handle
/// their own message boundaries and use buffer pools for packet data.
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    /// Read raw bytes from the socket into the provided buffer.
    /// Returns number of bytes read (0 = EOF).
    async fn read_bytes(&mut self, buf: &mut ReadBuffer) -> io::Result<(usize, TraceData)>;

    /// Write data to the transport.
    async fn write(&mut self, buf: &[u8]) -> io::Result<()>;

    /// Flush any buffered data.
    async fn flush(&mut self) -> io::Result<()>;

    /// Try to write without blocking (for non-async contexts).
    fn try_write(&mut self, buf: &[u8]) -> io::Result<usize>;

    /// Write all data (blocking until complete).
    async fn write_all(&mut self, buf: &[u8]) -> io::Result<()>;
}
