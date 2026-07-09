use redis::{aio::MultiplexedConnection, Client};
use tokio::time::timeout;

use crate::{config::RedisConfig, error::AppError, metrics};

pub async fn connect(config: &RedisConfig) -> Result<(Client, MultiplexedConnection), AppError> {
    let client = Client::open(config.url.as_str()).map_err(|error| {
        metrics::record_redis_error();
        AppError::dependency(format!("redis configuration failed: {error}"))
    })?;
    let mut connection = multiplexed_connection(&client, config.connect_timeout).await?;
    ping_connection(&mut connection, config.connect_timeout).await?;

    Ok((client, connection))
}

pub async fn multiplexed_connection(
    client: &Client,
    timeout_duration: std::time::Duration,
) -> Result<MultiplexedConnection, AppError> {
    timeout(timeout_duration, client.get_multiplexed_async_connection())
        .await
        .map_err(|_| {
            metrics::record_redis_error();
            AppError::dependency("redis connection timed out")
        })?
        .map_err(|error| {
            metrics::record_redis_error();
            AppError::dependency(format!("redis connection failed: {error}"))
        })
}

pub async fn ping(client: &Client, timeout_duration: std::time::Duration) -> Result<(), AppError> {
    let mut connection = multiplexed_connection(client, timeout_duration).await?;
    ping_connection(&mut connection, timeout_duration).await
}

async fn ping_connection(
    connection: &mut MultiplexedConnection,
    timeout_duration: std::time::Duration,
) -> Result<(), AppError> {
    timeout(
        timeout_duration,
        redis::cmd("PING").query_async::<String>(connection),
    )
    .await
    .map_err(|_| {
        metrics::record_redis_error();
        AppError::dependency("redis ping timed out")
    })?
    .map(|_| ())
    .map_err(|error| {
        metrics::record_redis_error();
        AppError::dependency(format!("redis ping failed: {error}"))
    })
}
