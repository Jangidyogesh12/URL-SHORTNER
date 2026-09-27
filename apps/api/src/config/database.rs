use crate::config::parameter;
use async_trait::async_trait;
use log::error;
use sqlx::{Error, PgPool, Pool, Postgres};
use std::process::exit;

pub struct Database {
    pool: PgPool,
}

#[async_trait]
pub trait DatabaseTrait {
    async fn init() -> Result<Self, Error>
    where
        Self: Sized;

    fn get_pool(&self) -> &PgPool;
}

#[async_trait]
impl DatabaseTrait for Database {
    async fn init() -> Result<Self, Error> {
        let database_url: String = parameter::get("DATABASE_URL").unwrap_or_else(|e| {
            error!("{}", e);
            exit(1);
        });

        let pool: Pool<Postgres> = PgPool::connect(&database_url).await?;
        Ok(Self { pool })
    }

    fn get_pool(&self) -> &PgPool {
        &self.pool
    }
}
