use crate::{
    constants::{
        BATCH_READ_MAX_BYTES, DEFAULT_BUFFER_CAPACITY, MAX_BUFFER_CAPACITY, MIO_BATCH_SIZE,
        MIO_POLL_TIMEOUT_MS,
    },
    net::transport::{ReadBuffer, Transport},
};
use tracing::error;

const MIO_TEMP_BUFFER_SIZE: usize = BATCH_READ_MAX_BYTES;
use async_trait::async_trait;
use bytes::BytesMut;
use mio::{Events, Interest, Poll, Token};
use std::{
    io::{self, Read, Write},
    net::ToSocketAddrs,
    sync::{
        Arc, Mutex as StdMutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};
use tokio::sync::mpsc;

/// TCP transport using MIO event loop
#[derive(Debug)]
pub struct MioTransport {
    /// Channel to receive batched messages
    msg_rx: mpsc::UnboundedReceiver<Vec<ReadBuffer>>,
    /// Shutdown flag
    shutdown: Arc<AtomicBool>,
    /// MIO stream for writes (wrapped for Send)
    write_stream: Arc<StdMutex<mio::net::TcpStream>>,
}

impl MioTransport {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let std_stream = std::net::TcpStream::connect(addr)?;
        std_stream.set_nodelay(true)?;
        std_stream.set_nonblocking(true)?;

        let std_stream_write = std_stream.try_clone()?;

        let read_stream = Arc::new(StdMutex::new(mio::net::TcpStream::from_std(std_stream)));
        let write_stream = Arc::new(StdMutex::new(mio::net::TcpStream::from_std(
            std_stream_write,
        )));
        let shutdown = Arc::new(AtomicBool::new(false));

        let (msg_tx, msg_rx) = mpsc::unbounded_channel();

        let shutdown_clone = Arc::clone(&shutdown);

        thread::Builder::new()
            .name("mio-transport-loop".to_string())
            .spawn(move || {
                if let Err(e) = Self::mio_tight_loop(read_stream, msg_tx, shutdown_clone) {
                    eprintln!("MIO tight loop error: {}", e);
                }
            })?;

        Ok(Self {
            msg_rx,
            shutdown,
            write_stream,
        })
    }

    /// MIO tight read loop running in dedicated thread.
    fn mio_tight_loop(
        stream: Arc<StdMutex<mio::net::TcpStream>>,
        msg_tx: mpsc::UnboundedSender<Vec<ReadBuffer>>,
        shutdown: Arc<AtomicBool>,
    ) -> io::Result<()> {
        const STREAM: Token = Token(0);

        let mut poll = Poll::new()?;
        let mut events = Events::with_capacity(128);
        let mut temp_buf = vec![0u8; MIO_TEMP_BUFFER_SIZE];

        // Accumulation buffer local to MIO thread
        let mut read_buf = BytesMut::with_capacity(DEFAULT_BUFFER_CAPACITY);

        // Register stream with MIO
        {
            let mut stream_lock = stream.lock().unwrap();
            poll.registry()
                .register(&mut *stream_lock, STREAM, Interest::READABLE)?;
        }

        loop {
            if shutdown.load(Ordering::Relaxed) {
                break;
            }

            // Poll for events with short timeout to allow shutdown checks
            poll.poll(
                &mut events,
                Some(Duration::from_millis(MIO_POLL_TIMEOUT_MS)),
            )?;

            for event in events.iter() {
                if event.token() == STREAM && event.is_readable() {
                    loop {
                        let mut stream_lock = stream.lock().unwrap();

                        match stream_lock.read(&mut temp_buf) {
                            Ok(0) => {
                                // EOF — clean exit
                                drop(stream_lock);
                                // notify receiver by closing sender (dropping msg_tx) happens automatically when exiting thread
                                return Ok(());
                            }
                            Ok(n) => {
                                drop(stream_lock);

                                read_buf.extend_from_slice(&temp_buf[..n]);

                                // Extract and send raw byte chunks
                                match Self::extract_chunks(
                                    &mut read_buf,
                                    MIO_BATCH_SIZE,
                                    BATCH_READ_MAX_BYTES,
                                ) {
                                    Ok(chunks) => {
                                        if !chunks.is_empty() {
                                            // if receiver closed, stop trying to send and exit gracefully
                                            if msg_tx.send(chunks).is_err() {
                                                return Ok(());
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("MIO extract error (recovering): {}", e);
                                        // clear buffer to regain sync and continue reading.
                                        read_buf.clear();
                                        // continue reading instead of returning Err
                                    }
                                }
                            }
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                                // socket drained for now
                                break;
                            }
                            Err(e) => {
                                // Log non-recoverable read error and exit cleanly
                                error!("MIO transport read error: {}", e);
                                return Err(e);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Extract up to `max_chunks` raw byte chunks from the buffer.
    /// No framing logic - returns raw data chunks for application protocols to parse.
    #[inline]
    fn extract_chunks(
        buf: &mut ReadBuffer,
        max_chunks: usize,
        max_bytes: usize,
    ) -> io::Result<Vec<ReadBuffer>> {
        if buf.is_empty() {
            return Ok(Vec::new());
        }

        let mut chunks = Vec::with_capacity(std::cmp::min(32, max_chunks));
        let mut total_bytes = 0;

        // Split buffer into chunks (raw bytes, no framing)
        #[cfg(any(
            feature = "transport_bytes",
            all(not(feature = "transport_bytes"), not(feature = "transport_slice"))
        ))]
        {
            while !buf.is_empty() && chunks.len() < max_chunks {
                // Calculate chunk size - take up to DEFAULT_BUFFER_CAPACITY or remaining bytes
                let chunk_size = buf.len().min(DEFAULT_BUFFER_CAPACITY);

                // Check if adding this chunk would exceed max_bytes
                if total_bytes + chunk_size > max_bytes {
                    // Take only what fits
                    let allowed = max_bytes - total_bytes;
                    if allowed > 0 {
                        let chunk = buf.split_to(allowed);
                        chunks.push(chunk);
                    }
                    break;
                }

                let chunk = buf.split_to(chunk_size);
                total_bytes += chunk_size;
                chunks.push(chunk);
            }

            // Recycle / shrink policy
            if buf.is_empty() {
                buf.clear();
            } else if buf.capacity() > MAX_BUFFER_CAPACITY {
                *buf = BytesMut::with_capacity(DEFAULT_BUFFER_CAPACITY);
            }
        }

        Ok(chunks)
    }

    #[inline]
    pub fn recycle_buffer(&mut self) {
        // Buffer is managed by MIO thread
    }
}

impl Drop for MioTransport {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

#[async_trait]
impl Transport for MioTransport {
    /// Read raw bytes into the provided buffer.
    /// Returns (bytes_read, trace_data) tuple.
    ///
    /// The MIO thread continuously reads from the socket and batches data.
    /// This method receives those batches and appends them to the caller's buffer.
    #[inline]
    async fn read_bytes(
        &mut self,
        buf: &mut ReadBuffer,
    ) -> io::Result<(usize, data_types::tracing::TraceData)> {
        // Wait for data from MIO thread
        match self.msg_rx.recv().await {
            Some(chunks) if !chunks.is_empty() => {
                // Append all batched data to caller's buffer
                let mut total = 0;
                for chunk in chunks {
                    #[cfg(any(
                        feature = "transport_bytes",
                        all(not(feature = "transport_bytes"), not(feature = "transport_slice"))
                    ))]
                    {
                        buf.extend_from_slice(&chunk);
                        total += chunk.len();
                    }

                    #[cfg(feature = "transport_slice")]
                    {
                        buf.extend_from_slice(&chunk);
                        total += chunk.len();
                    }
                }
                Ok((total, data_types::tracing::TraceData::default()))
            }
            Some(_) | None => Ok((0, data_types::tracing::TraceData::default())), // Empty batch or EOF
        }
    }

    #[inline]
    async fn write(&mut self, buf: &[u8]) -> io::Result<()> {
        self.write_all(buf).await
    }

    #[inline]
    async fn flush(&mut self) -> io::Result<()> {
        let mut stream = self.write_stream.lock().unwrap();
        stream.flush()
    }

    #[inline]
    fn try_write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut stream = self.write_stream.lock().unwrap();
        stream.write(buf)
    }

    #[inline]
    async fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        let mut remaining = buf;

        while !remaining.is_empty() {
            // Scope the lock to avoid Send issues
            let result = {
                let mut stream = self.write_stream.lock().unwrap();
                stream.write(remaining)
            }; // Lock is dropped here

            match result {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "failed to write whole buffer",
                    ));
                }
                Ok(n) => {
                    remaining = &remaining[n..];
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    tokio::task::yield_now().await;
                }
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}
