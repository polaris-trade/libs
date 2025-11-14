// use async_compression::tokio::bufread::GzipDecoder;
// use data_types::result::{ErrorKind, ParseError};
// use futures::{ready, Stream};
// use itch_message::{
//     enums::{
//         message_type::{DataFeedType, MessageType},
//         message_type_mdf::MessageTypeMdf,
//     },
//     messages::{header::MessageHeader, itch_message::ItchMessage, market_by_price::MarketByPrice},
// };
// use pin_project_lite::pin_project;
// use std::{
//     io,
//     path::Path,
//     pin::Pin,
//     task::{Context, Poll},
// };
// use tokio::{
//     fs::File,
//     io::{AsyncRead, BufReader, ReadBuf},
// };

// const BUFSIZE: usize = 8 * 1024;
// const MAX_LEVEL_OFFSET: usize = 9;
// const MIN_HEADER_WITH_LEVEL: usize = 10;

// pub type Result<T> = std::result::Result<T, ParseError>;

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum ReadMode {
//     Sequential, // the default
//     Framed,
// }

// pin_project! {
//     /// An asynchronous stream of ITCH protocol messages.
//     ///
//     /// This stream reads from an underlying async streams and parses ITCH messages
//     /// on-demand. It maintains an internal buffer to handle partial messages and
//     /// implements the `Stream` trait for async iteration.
//     ///
//     /// # Examples
//     ///
//     /// ```no_run
//     /// use streams::MessageStream;
//     /// use itch_message::enums::message_type::DataFeedType;
//     /// use futures::StreamExt;
//     ///
//     /// # async fn example() -> std::io::Result<()> {
//     /// let mut stream = MessageStream::from_file("data.bin", DataFeedType::Itch).await?;
//     ///
//     /// while let Some(result) = stream.next().await {
//     ///     match result {
//     ///         Ok(message) => println!("Parsed message: {:?}", message),
//     ///         Err(e) => eprintln!("Parse error: {:?}", e),
//     ///     }
//     /// }
//     /// # Ok(())
//     /// # }
//     /// ```
//     pub struct MessageStream<R> {
//         #[pin]
//         reader: R,
//         buffer: Box<[u8; BUFSIZE]>,
//         bufstart: usize,
//         bufend: usize,
//         bytes_read: usize,
//         read_calls: u32,
//         last_seq: u64,
//         is_fused: bool,
//         feed: DataFeedType,
//         read_mode: ReadMode
//     }
// }

// impl MessageStream<BufReader<File>> {
//     /// Creates a new `MessageStream` from a file path.
//     ///
//     /// # Arguments
//     ///
//     /// * `path` - Path to the ITCH data file
//     /// * `feed` - Type of data feed (ITCH or MDF)
//     ///
//     /// # Errors
//     ///
//     /// Returns an error if the file cannot be opened.
//     pub async fn from_file<P: AsRef<Path>>(path: P, feed: DataFeedType) -> io::Result<Self> {
//         let file = File::open(path).await?;
//         let reader = BufReader::new(file);
//         Ok(Self::from_reader_with_mode(
//             reader,
//             feed,
//             ReadMode::Sequential,
//         ))
//     }

//     pub async fn from_file_with_mode<P: AsRef<Path>>(
//         path: P,
//         feed: DataFeedType,
//         mode: ReadMode,
//     ) -> io::Result<Self> {
//         let file = File::open(path).await?;
//         let reader = BufReader::new(file);
//         Ok(Self::from_reader_with_mode(reader, feed, mode))
//     }
// }

// impl MessageStream<GzipDecoder<BufReader<File>>> {
//     /// Creates a new `MessageStream` from a gzip-compressed file path.
//     ///
//     /// # Arguments
//     ///
//     /// * `path` - Path to the gzipped ITCH data file
//     /// * `feed` - Type of data feed (ITCH or MDF)
//     ///
//     /// # Errors
//     ///
//     /// Returns an error if the file cannot be opened.
//     pub async fn from_gzip<P: AsRef<Path>>(path: P, feed: DataFeedType) -> io::Result<Self> {
//         let file = File::open(path).await?;
//         let reader = BufReader::new(file);
//         let gzip_decoder = GzipDecoder::new(reader);
//         Ok(Self::from_reader(gzip_decoder, feed))
//     }
// }

// impl<R: AsyncRead> MessageStream<R> {
//     /// Creates a new `MessageStream` from any async streams.
//     ///
//     /// # Arguments
//     ///
//     /// * `streams` - Any type implementing `AsyncRead`
//     /// * `feed` - Type of data feed (ITCH or MDF)
//     pub fn from_reader(reader: R, feed: DataFeedType) -> Self {
//         Self::from_reader_with_mode(reader, feed, ReadMode::Sequential)
//     }

//     /// new general constructor that accepts a read mode
//     pub fn from_reader_with_mode(reader: R, feed: DataFeedType, read_mode: ReadMode) -> Self {
//         Self {
//             reader,
//             buffer: Box::new([0; BUFSIZE]),
//             bufstart: 0,
//             bufend: 0,
//             bytes_read: 0,
//             read_calls: 0,
//             last_seq: 0,
//             is_fused: false,
//             feed,
//             read_mode,
//         }
//     }

//     /// Returns the total number of bytes read from the underlying streams.
//     pub fn bytes_read(&self) -> usize {
//         self.bytes_read
//     }

//     /// Returns the total number of u64 successfully parsed.
//     pub fn message_count(&self) -> u64 {
//         self.last_seq
//     }

//     /// Returns the number of read calls made to the underlying streams.
//     pub fn read_calls(&self) -> u32 {
//         self.read_calls
//     }

//     /// Returns whether the stream has been fused (terminated due to EOF or error).
//     pub fn is_fused(&self) -> bool {
//         self.is_fused
//     }

//     /// Returns a reference to the underlying streams.
//     ///
//     /// This allows access to streams-specific methods (e.g., sequence numbers from SoupBinTcpClient).
//     pub fn reader(&self) -> &R {
//         &self.reader
//     }

//     /// Returns a mutable reference to the underlying streams.
//     pub fn reader_mut(&mut self) -> &mut R {
//         &mut self.reader
//     }
// }

// impl<R: AsyncRead + Unpin> Stream for MessageStream<R> {
//     type Item = Result<ItchMessage>;

//     fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
//         let mut this = self.project();

//         if *this.is_fused {
//             return Poll::Ready(None);
//         }

//         loop {
//             // --- Parsing Scope ---
//             {
//                 let available_data = &this.buffer[*this.bufstart..*this.bufend];

//                 if available_data.is_empty() {
//                     // Break scope to fetch more bytes.
//                 } else {
//                     match this.read_mode {
//                         ReadMode::Sequential => {
//                             if let Ok(header) =
//                                 MessageHeader::parse(available_data, this.feed.clone())
//                             {
//                                 let message_len = match header.message_type {
//                                     MessageType::Mdf(MessageTypeMdf::MarketByPrice) => {
//                                         if available_data.len() < MIN_HEADER_WITH_LEVEL {
//                                             // Not enough data to read the dynamic length field, need more bytes.
//                                             0
//                                         } else {
//                                             let max_level = available_data[MAX_LEVEL_OFFSET];
//                                             MarketByPrice::total_len(max_level as i8)
//                                         }
//                                     }
//                                     _ => ItchMessage::get_static_len(
//                                         header.message_type,
//                                         this.feed.clone(),
//                                     )
//                                         .unwrap_or(0),
//                                 };

//                                 if message_len > 0 && available_data.len() >= message_len {
//                                     let message_slice = &available_data[..message_len];
//                                     // Pass the full message including the type byte to the parser
//                                     // The parsers expect to read from b[1..] with b[0] being the message type
//                                     // For stream parsing, we don't have timestamp context, so pass 0
//                                     let parse_result = ItchMessage::parse(
//                                         header,
//                                         message_slice,
//                                         this.feed.clone(),
//                                         0,
//                                     );

//                                     *this.bufstart += message_len;
//                                     *this.last_seq += 1;

//                                     return Poll::Ready(Some(parse_result.map_err(|e| {
//                                         *this.is_fused = true;
//                                         e
//                                     })));
//                                 }
//                                 // Not enough data for a full message, break scope to fetch more.
//                             } else {
//                                 // invalid header -> fatal
//                                 *this.is_fused = true;
//                                 return Poll::Ready(Some(Err(ParseError::new(
//                                     ErrorKind::InvalidMessageType,
//                                 ))));
//                             }
//                             // Not enough data for a full message, break scope to fetch more.
//                         }
//                         ReadMode::Framed => {
//                             // Expect frame layout: [0..8) seq (u64 LE), [8..12) len (u32 LE), [12..) data
//                             // Need at least 12 bytes to read length
//                             if available_data.len() < 12 {
//                                 // need more bytes
//                             } else {
//                                 // SAFETY: slice has at least 12 bytes
//                                 let len_bytes: [u8; 4] =
//                                     available_data[8..12].try_into().expect("slice len 4");
//                                 let data_len = u32::from_le_bytes(len_bytes) as usize;
//                                 let total_frame_len =
//                                     12usize.checked_add(data_len).unwrap_or(usize::MAX);

//                                 if data_len == 0 {
//                                     // treat empty payload as error
//                                     *this.is_fused = true;
//                                     return Poll::Ready(Some(Err(ParseError::new(
//                                         ErrorKind::InvalidMessageType,
//                                     ))));
//                                 }

//                                 if available_data.len() >= total_frame_len {
//                                     let message_slice = &available_data[12..12 + data_len];

//                                     // Parse header from the message payload (not from the frame header)
//                                     if let Ok(header) =
//                                         MessageHeader::parse(message_slice, this.feed.clone())
//                                     {
//                                         // Determine expected message length from header (dynamic/static)
//                                         let expected_len = match header.message_type {
//                                             MessageType::Mdf(MessageTypeMdf::MarketByPrice) => {
//                                                 if message_slice.len() < MIN_HEADER_WITH_LEVEL {
//                                                     0
//                                                 } else {
//                                                     let max_level = message_slice[MAX_LEVEL_OFFSET];
//                                                     MarketByPrice::total_len(max_level as i8)
//                                                 }
//                                             }
//                                             _ => ItchMessage::get_static_len(
//                                                 header.message_type,
//                                                 this.feed.clone(),
//                                             )
//                                                 .unwrap_or(0),
//                                         };

//                                         // Validate that the frame length matches the expected message length
//                                         if expected_len == 0 || expected_len != message_slice.len()
//                                         {
//                                             *this.is_fused = true;
//                                             println!(
//                                                 "Expected len: {}, actual len: {}, current seq: {}",
//                                                 expected_len,
//                                                 message_slice.len(),
//                                                 *this.last_seq + 1
//                                             );
//                                             println!("Header: {:?}", header);
//                                             return Poll::Ready(Some(Err(ParseError::new(
//                                                 ErrorKind::Incomplete {
//                                                     needed: Some(expected_len),
//                                                 },
//                                             ))));
//                                         }

//                                         let parse_result = ItchMessage::parse(
//                                             header,
//                                             message_slice,
//                                             this.feed.clone(),
//                                             0,
//                                         );

//                                         *this.bufstart += total_frame_len;
//                                         // *this.last_seq = current_seq;
//                                         *this.last_seq += 1;

//                                         return Poll::Ready(Some(parse_result.map_err(|e| {
//                                             *this.is_fused = true;
//                                             e
//                                         })));
//                                     } else {
//                                         // header parsing failed inside the payload -> fatal
//                                         *this.is_fused = true;
//                                         return Poll::Ready(Some(Err(ParseError::new(
//                                             ErrorKind::InvalidMessageType,
//                                         ))));
//                                     }
//                                 }
//                                 // otherwise not enough bytes for whole frame -> read more
//                             }
//                         }
//                     }
//                 }

//                 // else {
//                 //     // Header parsing failed, this is a fatal error.
//                 //     *this.is_fused = true;
//                 //     return Poll::Ready(Some(Err(ParseError::new(ErrorKind::InvalidMessageType))));
//                 // }
//             }
//             // --- End of Parsing Scope ---

//             // If we get here, we need more data.
//             // First, compact the buffer by moving the remaining data to the start.
//             if *this.bufstart > 0 {
//                 this.buffer.copy_within(*this.bufstart..*this.bufend, 0);
//                 *this.bufend -= *this.bufstart;
//                 *this.bufstart = 0;
//             }

//             // If the buffer is still full, it means the message is larger than the buffer.
//             if *this.bufend == BUFSIZE {
//                 *this.is_fused = true;
//                 return Poll::Ready(Some(Err(ParseError::new(ErrorKind::Incomplete {
//                     needed: None,
//                 }))));
//             }

//             // Create a ReadBuf that wraps the unfilled part of our buffer.
//             let mut read_buf = ReadBuf::new(&mut this.buffer[*this.bufend..]);

//             // Try to read more data into the rest of the buffer.
//             let poll_result = this.reader.as_mut().poll_read(cx, &mut read_buf);

//             match ready!(poll_result) {
//                 Ok(()) => {
//                     // The read was successful.
//                     let bytes_filled = read_buf.filled().len();
//                     if bytes_filled == 0 {
//                         // EOF reached
//                         *this.is_fused = true;
//                         if *this.bufend > 0 {
//                             // Data ends mid-message - incomplete data
//                             return Poll::Ready(Some(Err(ParseError::new(
//                                 ErrorKind::Incomplete { needed: None },
//                             ))));
//                         } else {
//                             // Clean EOF - no more messages
//                             return Poll::Ready(None);
//                         }
//                     } else {
//                         *this.bufend += bytes_filled;
//                         *this.read_calls += 1;
//                         *this.bytes_read += bytes_filled;
//                         continue; // Loop to try parsing again with the new data.
//                     }
//                 }
//                 Err(_e) => {
//                     *this.is_fused = true;
//                     // IO error occurred while reading
//                     return Poll::Ready(Some(Err(ParseError::new(ErrorKind::Io))));
//                 }
//             }
//         }
//     }
// }
