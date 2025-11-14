/// Server to client SoupBinTCP packet types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerPacket<'a> {
    Debug(&'a [u8]),

    /// Sent in response to [`ClientPacket::LoginRequest`].
    LoginAccepted {
        /// The session ID assigned to the client.
        /// 10 bytes, space-padded.
        session: &'a str,
        /// The start sequence number for the session.
        /// 20 bytes, space-padded.
        sequence_number: &'a str,
    },

    LoginRejected {
        reason: u8,
    },

    /// Actual market data payload to parse
    SequencedData(&'a [u8]),

    /// If client receive this, need to send [`ClientPacket::ClientHeartbeat`]
    ServerHeartbeat,

    EndOfSession,

    /// Unknown packet type.
    Unknown {
        packet_type: u8,
        payload: &'a [u8],
    },
}

/// Client to server SoupBinTCP packet types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientPacket<'a> {
    LoginRequest {
        username: &'a str,
        password: &'a str,
        /// The session ID requested by the client.
        session_id: &'a str,
        /// The start sequence number for the session. Min is 1.
        sequence_number: &'a str,
    },

    LogoutRequest,
    /// Sent in response to [`ServerPacket::ServerHeartbeat`] or periodically by the client
    ClientHeartbeat,
    UnsequencedData(&'a [u8]),
}

impl<'a> ServerPacket<'a> {
    pub fn parse(packet_type: u8, payload: &'a [u8]) -> Self {
        match packet_type {
            b'+' => ServerPacket::Debug(payload),
            b'A' => {
                if payload.len() >= 30 {
                    match (
                        std::str::from_utf8(&payload[0..10]),
                        std::str::from_utf8(&payload[10..30]),
                    ) {
                        (Ok(session), Ok(sequence_number)) => ServerPacket::LoginAccepted {
                            session: session.trim(),
                            sequence_number: sequence_number.trim(),
                        },
                        _ => ServerPacket::Unknown {
                            packet_type,
                            payload,
                        },
                    }
                } else {
                    ServerPacket::Unknown {
                        packet_type,
                        payload,
                    }
                }
            }
            b'J' => {
                if !payload.is_empty() {
                    ServerPacket::LoginRejected { reason: payload[0] }
                } else {
                    ServerPacket::Unknown {
                        packet_type,
                        payload,
                    }
                }
            }
            b'S' => ServerPacket::SequencedData(payload),
            b'H' => ServerPacket::ServerHeartbeat,
            b'Z' => ServerPacket::EndOfSession,
            _ => ServerPacket::Unknown {
                packet_type,
                payload,
            },
        }
    }
}

impl<'a> ClientPacket<'a> {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            ClientPacket::LoginRequest {
                username,
                password,
                session_id,
                sequence_number,
            } => {
                // 2 (len) + 1 (type) + 46 (payload)
                let mut buf = Vec::with_capacity(49);
                buf.extend_from_slice(&47u16.to_be_bytes());
                buf.push(b'L');

                Self::write_padded_left(&mut buf, username.as_bytes(), 6);
                Self::write_padded_left(&mut buf, password.as_bytes(), 10);
                Self::write_padded_left(&mut buf, session_id.as_bytes(), 10);
                Self::write_padded_right(&mut buf, sequence_number.as_bytes(), 20);

                buf
            }
            ClientPacket::LogoutRequest => Self::wrap_packet(b'O', &[]),
            ClientPacket::ClientHeartbeat => Self::wrap_packet(b'R', &[]),
            ClientPacket::UnsequencedData(data) => Self::wrap_packet(b'U', data),
        }
    }

    fn wrap_packet(packet_type: u8, payload: &[u8]) -> Vec<u8> {
        // type byte + payload length
        let packet_len = 1 + payload.len();
        let mut packet = Vec::with_capacity(2 + packet_len);

        // length field (big-endian u16)
        packet.extend_from_slice(&(packet_len as u16).to_be_bytes());

        packet.push(packet_type);

        packet.extend_from_slice(payload);

        packet
    }

    #[inline]
    fn write_padded_left(buf: &mut Vec<u8>, data: &[u8], width: usize) {
        let len = data.len().min(width);
        buf.extend_from_slice(&data[..len]);
        buf.resize(buf.len() + (width - len), b' ');
    }

    #[inline]
    fn write_padded_right(buf: &mut Vec<u8>, data: &[u8], width: usize) {
        let len = data.len().min(width);
        buf.resize(buf.len() + (width - len), b' ');
        buf.extend_from_slice(&data[..len]);
    }
}
