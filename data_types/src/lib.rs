pub mod error;
pub mod price;
pub mod result;
pub mod string;
pub mod time;
pub mod utils;
use std::io;

pub use error::ParseError;
pub use result::ParseResult;

use crate::{data_feed_type::DataFeedType, time::UnixNanoseconds};
pub mod data_feed_type;
pub mod tracing;

pub trait Parsable: Sized {
    /// Number of bytes this type requires to parse
    const BYTE_LEN: usize;

    /// Parse from a byte slice
    fn parse(b: &[u8]) -> ParseResult<Self>;
}

pub struct PacketContext<'a> {
    pub feed_type: Option<&'a DataFeedType>,
    pub last_timestamp: Option<UnixNanoseconds>,
}

pub trait PacketParser<T> {
    /// Parse bytes into T using the optional context
    fn parse(&self, bytes: &[u8], context: PacketContext) -> io::Result<T>;
}
