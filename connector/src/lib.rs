#[cfg(feature = "mssql")]
pub mod mssql;

pub mod error;
pub use error::ConnectionError;
pub type ConnectionResult<T> = Result<T, ConnectionError>;
