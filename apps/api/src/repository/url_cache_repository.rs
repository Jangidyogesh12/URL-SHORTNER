use crate::config::cache::{Cache, CacheTrait};
use crate::dto::url_dto::UrlReadDto;
use async_trait::async_trait;
use tracing::warn;
use redis::{AsyncCommands, RedisResult};
use std::sync::Arc;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

const URL_USER_PREFIX: &str = "url:user";
const URL_CODE_PREFIX: &str = "url:code";

#[derive(Clone)]
pub struct UrlCacheRepository {
    cache_conn: Arc<Cache>,
}

#[async_trait]
pub trait UrlCacheRepositoryTrait {
    fn new(cache_conn: &Arc<Cache>) -> Self;
    async fn get_by_user_and_long_url(&self, user_id: Uuid, long_url: &str) -> Option<UrlReadDto>;
    async fn get_by_short_code(&self, short_code: &str) -> Option<UrlReadDto>;
    async fn set_url(&self, user_id: Option<Uuid>, url: &UrlReadDto);
    async fn delete_by_user_and_long_url(&self, user_id: Uuid, long_url: &str);
    async fn delete_by_short_code(&self, short_code: &str);
}

#[async_trait]
impl UrlCacheRepositoryTrait for UrlCacheRepository {
    fn new(cache_conn: &Arc<Cache>) -> Self {
        Self {
            cache_conn: Arc::clone(cache_conn),
        }
    }

    async fn get_by_user_and_long_url(&self, user_id: Uuid, long_url: &str) -> Option<UrlReadDto> {
        let mut conn = self.cache_conn.get_manager().clone();
        let key = Self::user_key(user_id, long_url);

        let value: Option<String> = conn.get(key).await.unwrap_or_else(|e| {
            warn!("cache get failed: {}", e);
            None
        });

        value.and_then(|json| serde_json::from_str(&json).ok())
    }

    async fn get_by_short_code(&self, short_code: &str) -> Option<UrlReadDto> {
        let mut conn = self.cache_conn.get_manager().clone();
        let key = Self::code_key(short_code);

        let value: Option<String> = conn.get(key).await.unwrap_or_else(|e| {
            warn!("cache get failed: {}", e);
            None
        });

        value.and_then(|json| serde_json::from_str(&json).ok())
    }

    async fn set_url(&self, user_id: Option<Uuid>, url: &UrlReadDto) {
        let ttl = match Self::cache_ttl(url) {
            Some(ttl) => ttl,
            None => {
                warn!("skipping cache set: invalid expires_at");
                return;
            }
        };

        let json = match serde_json::to_string(url) {
            Ok(json) => json,
            Err(e) => {
                warn!("cache serialize failed: {}", e);
                return;
            }
        };

        let mut conn = self.cache_conn.get_manager().clone();

        if let Some(user_id) = user_id {
            let result: RedisResult<()> = match ttl {
                Some(ttl) => {
                    conn.set_ex(Self::user_key(user_id, &url.long_url), json.as_str(), ttl)
                        .await
                }
                None => conn.set(Self::user_key(user_id, &url.long_url), json.as_str()).await,
            };
            if let Err(e) = result {
                warn!("cache set failed: {}", e);
            }
        }

        let result: RedisResult<()> = match ttl {
            Some(ttl) => {
                conn.set_ex(Self::code_key(&url.short_code), json.as_str(), ttl)
                    .await
            }
            None => conn.set(Self::code_key(&url.short_code), json.as_str()).await,
        };
        if let Err(e) = result {
            warn!("cache set failed: {}", e);
        }
    }

    async fn delete_by_user_and_long_url(&self, user_id: Uuid, long_url: &str) {
        let mut conn = self.cache_conn.get_manager().clone();
        let result: RedisResult<()> = conn.del(Self::user_key(user_id, long_url)).await;
        if let Err(e) = result {
            warn!("cache delete failed: {}", e);
        }
    }

    async fn delete_by_short_code(&self, short_code: &str) {
        let mut conn = self.cache_conn.get_manager().clone();
        let result: RedisResult<()> = conn.del(Self::code_key(short_code)).await;
        if let Err(e) = result {
            warn!("cache delete failed: {}", e);
        }
    }
}

impl UrlCacheRepository {
    fn user_key(user_id: Uuid, long_url: &str) -> String {
        format!("{}:{}:{}", URL_USER_PREFIX, user_id, long_url)
    }

    fn code_key(short_code: &str) -> String {
        format!("{}:{}", URL_CODE_PREFIX, short_code)
    }

    fn cache_ttl(url: &UrlReadDto) -> Option<Option<u64>> {
        let expires_at = url.expires_at.as_deref()?;

        OffsetDateTime::parse(expires_at, &Rfc3339)
            .ok()
            .map(|expires_at| {
                Some(
                    (expires_at - OffsetDateTime::now_utc())
                        .whole_seconds()
                        .max(1) as u64,
                )
            })
    }
}
