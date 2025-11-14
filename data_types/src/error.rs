use std::borrow::Cow;

use thiserror::Error;

/// A parsing error with optional contextual information.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParseError {
    /// A character was invalid during parsing (e.g., unexpected byte).
    #[error("invalid character: 0x{value:02x}")]
    InvalidChar { value: u8 },

    /// A character was invalid at a specific position.
    #[error("invalid character: 0x{value:02x} at position {position}")]
    InvalidCharAt { value: u8, position: usize },

    /// A date field was invalid or malformed.
    #[error("invalid date")]
    InvalidDate,

    /// A date field was invalid at a specific position.
    #[error("invalid date at position {position}")]
    InvalidDateAt { position: usize },

    /// Timestamp had an unexpected format or unknown types.
    #[error("invalid timestamp: {timestamp}")]
    InvalidTimestamp { timestamp: &'static str },

    /// Timestamp error at a specific position.
    #[error("invalid timestamp: {timestamp} at position {position}")]
    InvalidTimestampAt {
        timestamp: &'static str,
        position: usize,
    },

    /// Message type field had an unexpected value.
    #[error("invalid message type: 0x{value:02x} ('{}')", *.value as char)]
    InvalidMessageType { value: u8 },

    /// Message type error at a specific position.
    #[error("invalid message type: 0x{value:02x} ('{}') at position {position}", *.value as char)]
    InvalidMessageTypeAt { value: u8, position: usize },

    /// A numeric or value field is outside of allowed bounds.
    #[error("invalid value")]
    InvalidValue,

    /// Invalid value at a specific position.
    #[error("invalid value at position {position}")]
    InvalidValueAt { position: usize },

    /// Enumeration encoding had an unexpected numeric value.
    #[error("invalid enum value: 0x{value:04x}")]
    InvalidEnumValue { value: u16 },

    /// Enumeration encoding had an unexpected numeric value at a position.
    #[error("invalid enum value: 0x{value:04x} at position {position}")]
    InvalidEnumValueAt { value: u16, position: usize },

    /// Enumeration encoding had an unexpected string value.
    #[error("invalid enum string: {invalid:?}")]
    InvalidEnumString { invalid: Vec<u8> },

    /// Enumeration encoding had an unexpected string value at a position.
    #[error("invalid enum string: {invalid:?} at position {position}")]
    InvalidEnumStringAt { invalid: Vec<u8>, position: usize },

    /// I/O error occurred while reading/parsing.
    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    /// UTF-8 decoding error.
    #[error("UTF-8 error: {source}")]
    Utf8 {
        #[from]
        source: std::str::Utf8Error,
    },

    /// Data ended prematurely: not enough bytes to parse a complete record.
    #[error("incomplete data (needed: {} bytes)", needed.map_or_else(|| "unknown".to_string(), |n| n.to_string()))]
    Incomplete { needed: Option<usize> },

    /// Incomplete data at a specific position.
    #[error("incomplete data at position {position} (needed: {} bytes)", needed.map_or_else(|| "unknown".to_string(), |n| n.to_string()))]
    IncompleteAt {
        needed: Option<usize>,
        position: usize,
    },

    /// Catch-all variant for ad-hoc messages.
    #[error("{message}")]
    Custom { message: Cow<'static, str> },
}

impl ParseError {
    /// Create a custom error with a message.
    pub fn custom<M>(message: M) -> Self
    where
        M: Into<Cow<'static, str>>,
    {
        Self::Custom {
            message: message.into(),
        }
    }

    /// Add position context to this error if it doesn't already have it.
    pub fn with_position(self, position: usize) -> Self {
        match self {
            Self::InvalidChar { value } => Self::InvalidCharAt { value, position },
            Self::InvalidDate => Self::InvalidDateAt { position },
            Self::InvalidTimestamp { timestamp } => Self::InvalidTimestampAt {
                timestamp,
                position,
            },
            Self::InvalidMessageType { value } => Self::InvalidMessageTypeAt { value, position },
            Self::InvalidValue => Self::InvalidValueAt { position },
            Self::InvalidEnumValue { value } => Self::InvalidEnumValueAt { value, position },
            Self::InvalidEnumString { invalid } => Self::InvalidEnumStringAt { invalid, position },
            Self::Incomplete { needed } => Self::IncompleteAt { needed, position },
            // Already have position or not applicable
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::result::ParseResult;

    use super::*;

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::InvalidChar { value: 0x41 };
        assert!(err.to_string().contains("invalid character: 0x41"));

        let err = ParseError::InvalidTimestamp { timestamp: "TS" };
        assert!(err.to_string().contains("invalid timestamp: TS"));

        let err = ParseError::Incomplete { needed: Some(5) };
        assert!(err.to_string().contains("incomplete data"));
    }

    #[test]
    fn test_parse_error_with_position() {
        let err = ParseError::InvalidChar { value: 0xFF }.with_position(42);
        assert!(matches!(
            err,
            ParseError::InvalidCharAt {
                value: 0xFF,
                position: 42
            }
        ));
        assert!(err.to_string().contains("position 42"));

        let err = ParseError::InvalidDate.with_position(100);
        assert!(matches!(err, ParseError::InvalidDateAt { position: 100 }));
        assert!(err.to_string().contains("position 100"));
    }

    #[test]
    fn test_parse_result_ok() {
        // Test that ParseResult type alias works correctly with Ok variant
        fn returns_ok() -> ParseResult<i32> {
            Ok(42)
        }
        let result = returns_ok();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_parse_result_err() {
        // Test that ParseResult type alias works correctly with Err variant
        fn returns_err() -> ParseResult<i32> {
            Err(ParseError::InvalidValue)
        }
        let result = returns_err();
        assert!(result.is_err());
        assert!(matches!(result, Err(ParseError::InvalidValue)));
    }

    #[test]
    fn test_error_trait() {
        // Ensure ParseError implements std::error::Error
        fn assert_error<E: std::error::Error>(_: E) {}
        assert_error(ParseError::InvalidChar { value: 0x00 });
    }

    #[test]
    fn test_error_source_chain() {
        use std::error::Error;

        // Test that IO errors chain properly
        let io_err = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "test");
        let parse_err = ParseError::from(io_err);
        assert!(parse_err.source().is_some());

        // Test that UTF-8 errors chain properly
        let bytes = b"\xFF\xFF";
        #[allow(invalid_from_utf8)]
        let Err(utf8_err) = std::str::from_utf8(bytes) else {
            panic!("Expected UTF-8 error");
        };
        let parse_err = ParseError::from(utf8_err);
        assert!(parse_err.source().is_some());
    }

    #[test]
    fn test_from_conversions() {
        // Test From<io::Error>
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let err: ParseError = io_err.into();
        assert!(matches!(err, ParseError::Io { .. }));
    }

    #[test]
    fn test_custom_error() {
        let err = ParseError::custom("something went wrong");
        assert!(err.to_string().contains("something went wrong"));
    }
}
