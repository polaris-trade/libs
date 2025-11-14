use std::borrow::Cow;

/// Connection error without application context.
///
/// This error type focuses on **what went wrong** at the connection layer,
/// not **where** or **why** from the application's perspective.
///
/// Application code should add context using `anyhow::Context`:
/// ```rust,ignore
/// use anyhow::Context;
///
/// create_mssql_client(config)
///     .await
///     .context("Failed to connect to production database")?;
/// ```
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConnectionError {
    /// Invalid configuration provided.
    #[error("invalid configuration: {message}")]
    InvalidConfig { message: Cow<'static, str> },

    /// Connection timed out.
    #[error("connection timed out")]
    Timeout,

    /// Connection refused by the server.
    #[error("connection refused")]
    Refused,

    /// Authentication failed.
    #[error("authentication failed: {message}")]
    AuthenticationFailed { message: Cow<'static, str> },

    /// DNS resolution failed.
    #[error("DNS resolution failed for '{hostname}'")]
    DnsResolutionFailed { hostname: String },

    /// TLS/SSL error.
    #[error("TLS error: {message}")]
    Tls { message: Cow<'static, str> },

    /// I/O error occurred.
    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    /// Protocol-level error.
    #[error("protocol error: {message}")]
    Protocol { message: Cow<'static, str> },

    /// Connection closed unexpectedly.
    #[error("connection closed unexpectedly")]
    ConnectionClosed,

    /// Maximum retry attempts exceeded.
    #[error("maximum retries exceeded ({attempts} attempts)")]
    MaxRetriesExceeded { attempts: usize },

    /// Database-specific error.
    #[error("database error: {message}")]
    DatabaseSpecific { message: Cow<'static, str> },

    /// Catch-all for other errors.
    #[error("{message}")]
    Other { message: Cow<'static, str> },
}

impl ConnectionError {
    /// Create an invalid config error.
    pub fn invalid_config(message: impl Into<Cow<'static, str>>) -> Self {
        Self::InvalidConfig {
            message: message.into(),
        }
    }

    /// Create a timeout error.
    pub fn timeout() -> Self {
        Self::Timeout
    }

    /// Create a connection refused error.
    pub fn refused() -> Self {
        Self::Refused
    }

    /// Create an authentication failed error.
    pub fn auth_failed(message: impl Into<Cow<'static, str>>) -> Self {
        Self::AuthenticationFailed {
            message: message.into(),
        }
    }

    /// Create a DNS resolution failed error.
    pub fn dns_failed(hostname: impl Into<String>) -> Self {
        Self::DnsResolutionFailed {
            hostname: hostname.into(),
        }
    }

    /// Create a protocol error.
    pub fn protocol(message: impl Into<Cow<'static, str>>) -> Self {
        Self::Protocol {
            message: message.into(),
        }
    }

    /// Create a database-specific error.
    pub fn database(message: impl Into<Cow<'static, str>>) -> Self {
        Self::DatabaseSpecific {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_error_display() {
        let err = ConnectionError::invalid_config("missing host");
        assert!(err.to_string().contains("invalid configuration"));

        let err = ConnectionError::timeout();
        assert!(err.to_string().contains("timed out"));
    }

    #[test]
    fn test_error_kinds() {
        let err = ConnectionError::auth_failed("invalid password");
        assert!(matches!(err, ConnectionError::AuthenticationFailed { .. }));

        let err = ConnectionError::MaxRetriesExceeded { attempts: 3 };
        if let ConnectionError::MaxRetriesExceeded { attempts } = err {
            assert_eq!(attempts, 3);
        } else {
            panic!("Expected MaxRetriesExceeded variant");
        }
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
        let conn_err: ConnectionError = io_err.into();
        assert!(matches!(conn_err, ConnectionError::Io { .. }));
    }

    #[test]
    fn test_error_source_chain() {
        use std::error::Error;

        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "test");
        let conn_err = ConnectionError::from(io_err);
        assert!(conn_err.source().is_some());
    }
}
