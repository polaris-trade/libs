use crate::{ConnectionError, ConnectionResult};
use bb8::{Pool, PooledConnection};
use bb8_tiberius::ConnectionManager;
use config_loader::database::MssqlConfig;
use std::time::Duration;
use tiberius::Config;

pub type MssqlPool = Pool<ConnectionManager>;
pub type MssqlClient<'a> = PooledConnection<'a, ConnectionManager>;
pub use tiberius::Query;

pub async fn create_mssql_client(config: MssqlConfig) -> ConnectionResult<MssqlPool> {
    let mut mssql_config = Config::new();
    mssql_config.host(config.host.as_str());
    mssql_config.port(config.port);
    mssql_config.authentication(tiberius::AuthMethod::sql_server(
        &config.username,
        &config.password,
    ));
    mssql_config.database(&config.database);

    let manager = ConnectionManager::new(mssql_config);

    let pool = Pool::builder()
        .max_size(config.pool_size.unwrap_or(10))
        .min_idle(config.min_idle)
        .connection_timeout(Duration::from_secs(
            config.connection_timeout.unwrap_or(30) as u64
        ))
        .build(manager)
        .await
        .map_err(|e| ConnectionError::Io {
            source: std::io::Error::new(std::io::ErrorKind::Other, e),
        })?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let conn_err = ConnectionError::Io { source: io_err };

        let display = conn_err.to_string();
        assert!(display.contains("I/O error"));
    }
}
