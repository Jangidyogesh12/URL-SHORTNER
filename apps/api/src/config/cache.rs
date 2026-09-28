use crate::config::parameter;
use async_trait::async_trait;
use tracing::error;
use redis::aio::ConnectionManager;
use redis::{Client, RedisError};
use std::process::exit;

pub struct Cache {
    manager: ConnectionManager,
}

#[async_trait]
pub trait CacheTrait {
    async fn init() -> Result<Self, RedisError>
    where
        Self: Sized;

    fn get_manager(&self) -> &ConnectionManager;
}

#[async_trait]
impl CacheTrait for Cache {
    async fn init() -> Result<Self, RedisError> {
        let cache_url: String = parameter::get("REDIS_URL").unwrap_or_else(|e| {
            error!("{}", e);
            exit(1);
        });

        let client = Client::open(cache_url)?;
        let manager = ConnectionManager::new(client).await?;

        Ok(Self { manager })
    }

    fn get_manager(&self) -> &ConnectionManager {
        &self.manager
    }
}
