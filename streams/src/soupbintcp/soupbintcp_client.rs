#[cfg(all(feature = "tokio_transport", not(feature = "mio_transport")))]
use crate::net::tokio_transport::TokioTransport as NetworkTransport;

#[cfg(all(feature = "mio_transport", not(feature = "tokio_transport")))]
use crate::net::mio_transport::MioTransport as NetworkTransport;

// Fallback if both or neither are enabled
#[cfg(all(feature = "tokio_transport", feature = "mio_transport"))]
use crate::net::tokio_transport::TokioTransport as NetworkTransport;

#[cfg(not(any(feature = "tokio_transport", feature = "mio_transport")))]
compile_error!("Either tokio_transport or mio_transport feature must be enabled");

use crate::{
    constants::{
        DEFAULT_BUFFER_CAPACITY, DEFAULT_HEARTBEAT_INTERVAL_SECS, DEFAULT_MAX_RECONNECT_ATTEMPTS,
        DEFAULT_RECONNECT_DELAY_MS, MAX_RECONNECT_DELAY_MS, MIN_SPARE_CAPACITY,
        SOUPBINTCP_LENGTH_SIZE, SOUPBINTCP_MIN_HEADER,
    },
    net::transport::{ReadBuffer, Transport},
    soupbintcp::soupbintcp_packet::{ClientPacket, ServerPacket},
};
use bytes::Bytes;
use crossbeam_channel::Sender;
use data_types::{
    PacketContext, PacketParser, data_feed_type::DataFeedType, time::UnixNanoseconds,
};
use logger::error;
use queue::PacketData;
use std::{fmt, io};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionEvent {
    Connected,
    Reconnecting,
    Reconnected,
    Disconnected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoupBinTcpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub feed_type: DataFeedType,
    pub start_sequence: String,
    pub start_session: String,
}

type ParserFn<T> = Box<dyn PacketParser<T> + Send + Sync>;

pub struct SoupBinTcpClient<T> {
    stream: NetworkTransport,
    parser: ParserFn<T>,
    packet_sender: Sender<PacketData<T>>,
    read_buf: ReadBuffer,
    current_sequence: u64,
    last_server_activity: std::time::Instant,
    last_heartbeat_sent: std::time::Instant,
    last_known_timestamp: UnixNanoseconds,
    current_trace: Option<data_types::tracing::TraceData>,
    feed_type: DataFeedType,
    config: ReconnectConfig,
    reconnect_attempts: u32,
    event_sender: Option<Sender<(DataFeedType, ConnectionEvent)>>,
    just_sent_login: bool,
    heartbeat_interval_secs: u64,
    pending_server_heartbeat: bool,
}

impl<T> fmt::Debug for SoupBinTcpClient<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SoupBinTcpClient")
            .field("packet_sender", &self.packet_sender)
            .field("current_sequence", &self.current_sequence)
            .field("heartbeat_interval_secs", &self.heartbeat_interval_secs)
            .finish()
    }
}

#[derive(Debug, Clone)]
struct ReconnectConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    session: String,
    max_attempts: u32,
    initial_delay_ms: u64,
}

impl<T> SoupBinTcpClient<T> {
    pub async fn connect(
        config: SoupBinTcpConfig,
        sender: Sender<PacketData<T>>,
        parser: ParserFn<T>,
    ) -> io::Result<Self> {
        Self::connect_with_retry_config(
            config,
            sender,
            parser,
            None,
            DEFAULT_MAX_RECONNECT_ATTEMPTS,
            DEFAULT_RECONNECT_DELAY_MS,
        )
        .await
    }

    /// Connect with optional event channel for feed status notifications
    pub async fn connect_with_events(
        config: SoupBinTcpConfig,
        sender: Sender<PacketData<T>>,
        parser: ParserFn<T>,
        event_sender: Sender<(DataFeedType, ConnectionEvent)>,
    ) -> io::Result<Self> {
        Self::connect_with_retry_config(
            config,
            sender,
            parser,
            Some(event_sender),
            DEFAULT_MAX_RECONNECT_ATTEMPTS,
            DEFAULT_RECONNECT_DELAY_MS,
        )
        .await
    }

    async fn connect_with_retry_config(
        config: SoupBinTcpConfig,
        sender: Sender<PacketData<T>>,
        parser: ParserFn<T>,
        event_sender: Option<Sender<(DataFeedType, ConnectionEvent)>>,
        max_reconnect_attempts: u32,
        initial_delay_ms: u64,
    ) -> io::Result<Self> {
        let addr = format!("{}:{}", config.host, config.port);
        let stream = NetworkTransport::connect(&addr).await?;

        let reconnect_config = ReconnectConfig {
            host: config.host.to_string(),
            port: config.port,
            username: config.username.to_string(),
            password: config.password.to_string(),
            session: config.start_session.to_string(),
            max_attempts: max_reconnect_attempts,
            initial_delay_ms,
        };

        let feed_type = config.feed_type;

        let read_buf = ReadBuffer::with_capacity(DEFAULT_BUFFER_CAPACITY);

        let now = std::time::Instant::now();

        let mut client = Self {
            stream,
            parser,
            read_buf,
            current_sequence: 0,
            last_server_activity: now,
            last_heartbeat_sent: now,
            last_known_timestamp: UnixNanoseconds(0),
            current_trace: None,
            feed_type,
            config: reconnect_config,
            reconnect_attempts: 0,
            packet_sender: sender,
            event_sender,
            just_sent_login: false,
            heartbeat_interval_secs: DEFAULT_HEARTBEAT_INTERVAL_SECS,
            pending_server_heartbeat: false,
        };

        client
            .send_login(
                &config.username,
                &config.password,
                &config.start_session,
                &config.start_sequence,
            )
            .await?;

        client.send_event(ConnectionEvent::Connected).await;

        Ok(client)
    }

    pub fn current_sequence(&self) -> u64 {
        self.current_sequence
    }

    pub fn feed_type(&self) -> &DataFeedType {
        &self.feed_type
    }

    pub async fn pump_packets(&mut self) -> io::Result<()> {
        loop {
            // non-blocking heartbeat sending
            self.try_send_heartbeats();

            // batch process all buffered packets
            while let Some((packet_type, packet_bytes)) = self.try_parse_packet() {
                self.process_packet(packet_type, packet_bytes).await?;
            }

            // buffer management: shrink if too large and mostly empty
            {
                use crate::constants::MAX_BUFFER_CAPACITY;

                // if buffer > MAX_BUFFER_CAPACITY and is mostly empty, shrink it
                if self.read_buf.capacity() > MAX_BUFFER_CAPACITY
                    && self.read_buf.len() < MIN_SPARE_CAPACITY
                {
                    // For BytesMut, create a new buffer with appropriate capacity
                    let new_capacity = std::cmp::max(
                        DEFAULT_BUFFER_CAPACITY,
                        self.read_buf.len() + MIN_SPARE_CAPACITY,
                    );
                    let mut new_buf = ReadBuffer::with_capacity(new_capacity);
                    new_buf.extend_from_slice(&self.read_buf[..]);
                    self.read_buf = new_buf;
                }
            }

            // reserve space if needed
            if self.read_buf.capacity() - self.read_buf.len() < MIN_SPARE_CAPACITY {
                self.read_buf.reserve(MIN_SPARE_CAPACITY);
            }

            // Create a span for this TCP read operation
            let read_span = tracing::trace_span!(
                "tcp_read",
                feed_type = ?self.feed_type,
                seq = self.current_sequence + 1
            );
            let _guard = read_span.enter();

            match self.stream.read_bytes(&mut self.read_buf).await {
                Ok((0, _)) => {
                    // no more data available right now, continue loop
                    return Ok(());
                }
                Ok((_n, trace_data)) => {
                    self.current_trace = Some(trace_data);
                    // process multiple complete packets in the next loop iteration
                }
                Err(e) if self.is_reconnectable_error(&e) => {
                    self.try_reconnect().await?;
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    async fn send_login(
        &mut self,
        username: &str,
        password: &str,
        session_id: &str,
        sequence_number: &str,
    ) -> io::Result<()> {
        let packet = ClientPacket::LoginRequest {
            username,
            password,
            session_id,
            sequence_number,
        };

        let result = self.send_packet(packet).await;

        // immediate auth failure detection
        self.just_sent_login = true;

        result
    }

    /// Non-blocking feed connection event notification
    async fn send_event(&self, event: ConnectionEvent) {
        if let Some(ref tx) = self.event_sender {
            let _ = tx.send((self.feed_type, event));
        }
    }

    /// Non-blocking attempt to send pending heartbeats using try_write
    /// to avoid blocking data processing if socket buffer is full
    #[inline]
    fn try_send_heartbeats(&mut self) {
        // Length=1, Type='R'
        let packet = b"\x00\x01R";

        // check if need to send heartbeat
        let need_periodic =
            self.last_heartbeat_sent.elapsed().as_secs() >= self.heartbeat_interval_secs;
        let need_response = self.pending_server_heartbeat;

        if need_periodic || need_response {
            match self.stream.try_write(packet) {
                Ok(n) if n == packet.len() => {
                    self.last_heartbeat_sent = std::time::Instant::now();
                    self.pending_server_heartbeat = false;
                    println!("Sent heartbeat (non-blocking)");
                }
                Ok(_) => {
                    // partial write - will retry next iteration
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // socket full, try next iteration
                    // data processing continues without blocking
                }
                Err(_) => {
                    // other errors - ignore for now, will be caught on next read
                }
            }
        }
    }

    #[inline]
    async fn send_packet(&mut self, packet: ClientPacket<'_>) -> io::Result<()> {
        let bytes = packet.to_bytes();
        self.stream.write_all(&bytes).await?;
        self.stream.flush().await?;
        self.last_heartbeat_sent = std::time::Instant::now();
        Ok(())
    }

    /// Parse a packet from the read buffer.
    ///
    /// Returns the packet type and the complete packet bytes (including header).
    #[inline]
    fn try_parse_packet(&mut self) -> Option<(u8, Bytes)> {
        if self.read_buf.len() < SOUPBINTCP_MIN_HEADER {
            return None;
        }

        let packet_len = u16::from_be_bytes([self.read_buf[0], self.read_buf[1]]) as usize;
        let total_len = SOUPBINTCP_LENGTH_SIZE + packet_len;

        if self.read_buf.len() < total_len {
            return None;
        }

        let packet_type = self.read_buf[SOUPBINTCP_LENGTH_SIZE];

        let packet_bytes = Bytes::copy_from_slice(&self.read_buf[..total_len]);

        // remove parsed data from read buffer
        let _ = self.read_buf.split_to(total_len);

        Some((packet_type, packet_bytes))
    }

    #[inline]
    #[tracing::instrument(
        skip(self, packet_bytes),
        fields(
            seq = self.current_sequence + 1,
            feed_type = ?self.feed_type,
            packet_type = %format!("{}", packet_type as char)
        ),
        name = "parse_packet"
    )]
    async fn process_packet(&mut self, packet_type: u8, packet_bytes: Bytes) -> io::Result<()> {
        self.last_server_activity = std::time::Instant::now();
        self.just_sent_login = false;

        if packet_type == b'S' {
            self.current_sequence += 1;

            let payload = &packet_bytes[SOUPBINTCP_MIN_HEADER..];

            let context = PacketContext {
                feed_type: Some(&self.feed_type),
                last_timestamp: Some(self.last_known_timestamp),
            };

            let parsed = self
                .parser
                .parse(payload, context)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

            // // update last known timestamp
            // if parsed.msg_type == MessageType::Seconds {
            //     if let itch_message::messages::body::MessageBody::Seconds(ref seconds_msg) =
            //         parsed.body
            //     {
            //         self.last_known_timestamp = seconds_msg.timestamp;
            //     }
            // }

            // Use the trace data captured during TCP read, or create new if missing
            let trace_data = self
                .current_trace
                .clone()
                .unwrap_or_else(data_types::tracing::TraceData::with_current_context);

            match self.packet_sender.try_send((
                self.current_sequence,
                packet_bytes,
                parsed,
                Some(trace_data),
            )) {
                Ok(_) => {
                    return Ok(());
                }
                Err(crossbeam_channel::TrySendError::Full(packet)) => {
                    // apply backpressure by blocking
                    self.packet_sender
                        .send(packet)
                        .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Disconnected"))?;
                }
                Err(crossbeam_channel::TrySendError::Disconnected(_)) => {
                    return Err(io::Error::new(io::ErrorKind::BrokenPipe, "Disconnected"));
                }
            }

            return Ok(());
        }

        let payload = &packet_bytes[SOUPBINTCP_MIN_HEADER..];
        let packet = ServerPacket::parse(packet_type, payload);

        match packet {
            ServerPacket::LoginAccepted {
                session,
                sequence_number,
            } => {
                if let Ok(seq) = sequence_number.trim().parse::<u64>() {
                    println!(
                        "Login accepted: session='{}', server will start from sequence {}",
                        session, seq
                    );
                    self.current_sequence = seq;
                }
                self.reconnect_attempts = 0;
            }
            ServerPacket::LoginRejected { reason } => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!("Login rejected: reason code {}", reason),
                ));
            }
            ServerPacket::ServerHeartbeat => {
                println!("Received server heartbeat");
                self.pending_server_heartbeat = true;
            }
            ServerPacket::EndOfSession => {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Server ended session",
                ));
            }
            ServerPacket::Debug(_) | ServerPacket::Unknown { .. } => {
                // ignored
            }
            ServerPacket::SequencedData(_) => unreachable!(),
        }

        Ok(())
    }

    async fn try_reconnect(&mut self) -> io::Result<()> {
        self.send_event(ConnectionEvent::Reconnecting).await;

        if self.reconnect_attempts >= self.config.max_attempts {
            self.send_event(ConnectionEvent::Disconnected).await;
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                format!(
                    "Max reconnection attempts ({}) exceeded",
                    self.config.max_attempts
                ),
            ));
        }

        self.reconnect_attempts += 1;

        let delay_ms = self.config.initial_delay_ms * (2_u64.pow(self.reconnect_attempts - 1));
        let delay = std::cmp::min(delay_ms, MAX_RECONNECT_DELAY_MS);

        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

        let addr = format!("{}:{}", self.config.host, self.config.port);
        match NetworkTransport::connect(&addr).await {
            Ok(new_stream) => {
                self.stream = new_stream;
                self.read_buf.clear();
                self.pending_server_heartbeat = false;

                let sequence_str = format!("{}", self.current_sequence + 1);
                println!(
                    "Reconnecting: requesting session '{}' starting from sequence {}",
                    self.config.session, sequence_str
                );
                let username = self.config.username.clone();
                let password = self.config.password.clone();
                let session = self.config.session.clone();
                self.send_login(&username, &password, &session, &sequence_str)
                    .await?;

                self.last_server_activity = std::time::Instant::now();

                self.send_event(ConnectionEvent::Reconnected).await;

                Ok(())
            }
            Err(e) => {
                error!(
                    "Reconnection attempt {} failed for {:?} feed: {:?}",
                    self.reconnect_attempts, self.feed_type, e
                );
                Err(e)
            }
        }
    }

    fn is_reconnectable_error(&self, e: &io::Error) -> bool {
        matches!(
            e.kind(),
            io::ErrorKind::ConnectionReset
                | io::ErrorKind::ConnectionAborted
                | io::ErrorKind::BrokenPipe
                | io::ErrorKind::NotConnected
        )
    }
}
