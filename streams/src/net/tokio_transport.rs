use super::transport::{ReadBuffer, Transport};

use data_types::tracing::TraceData;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Result as IoResult},
    net::TcpStream,
};

/// Standard TCP transport using tokio
#[derive(Debug)]
pub struct TokioTransport {
    pub stream: TcpStream,
}

impl TokioTransport {
    pub async fn connect(addr: &str) -> IoResult<Self> {
        let stream = TcpStream::connect(addr).await?;
        stream.set_nodelay(true)?;

        Ok(Self { stream })
    }
}

#[async_trait::async_trait]
impl Transport for TokioTransport {
    #[inline]
    async fn read_bytes(&mut self, buf: &mut ReadBuffer) -> IoResult<(usize, TraceData)> {
        let trace_data = TraceData::with_current_context();

        let n = self.stream.read_buf(buf).await?;
        Ok((n, trace_data))
    }

    #[inline]
    async fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        self.stream.write_all(buf).await
    }

    #[inline]
    async fn flush(&mut self) -> IoResult<()> {
        self.stream.flush().await
    }

    #[inline]
    fn try_write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.stream.try_write(buf)
    }

    #[inline]
    async fn write_all(&mut self, buf: &[u8]) -> IoResult<()> {
        self.stream.write_all(buf).await
    }
}
