use bytes::Bytes;
use data_types::tracing::TraceData;

/// Packet data: sequence number, raw bytes (for backup), parsed message, optional trace data
pub type PacketDataWithTrace<T> = (u64, Bytes, T, TraceData);
pub type PacketData<T> = (u64, Bytes, T, Option<TraceData>);
