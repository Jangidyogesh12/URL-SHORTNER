use crate::{
    config::{cache::Cache, database::Database},
    dto::url_dto::UrlReadDto,
    entity::{url::Url, user::User},
    error::{api_error::ApiError, url_error::UrlError},
    repository::{
        url_cache_repository::{UrlCacheRepository, UrlCacheRepositoryTrait},
        url_repository::{UrlRepository, UrlRepositoryTrait},
    },
};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};
use uuid::Uuid;

const SHORT_CODE_LENGTH: usize = 8;
const SHORT_CODE_MAX_ATTEMPTS: usize = 5;
const URL_EXPIRATION_DAYS: i64 = 30;
const BASE62_ALPHABET: &[u8; 62] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

#[derive(Clone)]
pub struct UrlService {
    url_repo: UrlRepository,
    url_cache: UrlCacheRepository,
}

impl UrlService {
    pub fn new(db_conn: &Arc<Database>, cache_conn: &Arc<Cache>) -> Self {
        Self {
            url_repo: UrlRepository::new(db_conn),
            url_cache: UrlCacheRepository::new(cache_conn),
        }
    }

    pub async fn create_url(
        &self,
        user: &User,
        long_url: String,
    ) -> Result<(UrlReadDto, bool), ApiError> {
        if let Some(url) = self
            .url_cache
            .get_by_user_and_long_url(user.id, &long_url)
            .await
        {
            if is_alive(&url) {
                return Ok((url, false));
            }
        }

        if let Some(url) = self
            .url_repo
            .find_by_user_and_long_url(user.id, long_url.clone())
            .await
        {
            if url.expires_at > OffsetDateTime::now_utc() {
                let dto = UrlReadDto::from(url);
                self.url_cache.set_url(Some(user.id), &dto).await;
                return Ok((dto, false));
            }
        }

        let short_code = self
            .generate_unique_short_code(&user.id, &long_url)
            .await?;
        let now = OffsetDateTime::now_utc();

        let url = self
            .url_repo
            .create(Url {
                id: Uuid::new_v4(),
                user_id: Some(user.id),
                original_url: long_url,
                short_code,
                created_at: now,
                updated_at: now,
                expires_at: now + Duration::days(URL_EXPIRATION_DAYS),
            })
            .await?;

        let dto = UrlReadDto::from(url);
        self.url_cache.set_url(Some(user.id), &dto).await;

        Ok((dto, true))
    }

    pub async fn get_url(&self, user: &User, long_url: String) -> Result<UrlReadDto, ApiError> {
        if let Some(url) = self
            .url_cache
            .get_by_user_and_long_url(user.id, &long_url)
            .await
        {
            if is_alive(&url) {
                return Ok(url);
            }
        }

        let url = self
            .url_repo
            .find_by_user_and_long_url(user.id, long_url)
            .await
            .ok_or(UrlError::UrlNotFound)?;

        if url.expires_at <= OffsetDateTime::now_utc() {
            return Err(UrlError::UrlExpired.into());
        }

        let dto = UrlReadDto::from(url);
        self.url_cache.set_url(Some(user.id), &dto).await;

        Ok(dto)
    }

    pub async fn get_urls(&self, user: &User) -> Result<Vec<UrlReadDto>, ApiError> {
        let urls = self.url_repo.find_all_by_user(user.id).await?;
        Ok(urls.into_iter().map(UrlReadDto::from).collect())
    }

    pub async fn delete_url(&self, user: &User, long_url: String) -> Result<(), ApiError> {
        let url = self
            .url_repo
            .find_by_user_and_long_url(user.id, long_url.clone())
            .await
            .ok_or(UrlError::UrlNotFound)?;

        let rows_affected = self
            .url_repo
            .delete_by_user_and_long_url(user.id, long_url)
            .await?;

        if rows_affected == 0 {
            return Err(UrlError::UrlNotFound.into());
        }

        self.url_cache
            .delete_by_user_and_long_url(user.id, &url.original_url)
            .await;
        self.url_cache.delete_by_short_code(&url.short_code).await;

        Ok(())
    }

    pub async fn edit_url(
        &self,
        user: &User,
        short_code: String,
        new_url: String,
    ) -> Result<UrlReadDto, ApiError> {
        let url = self
            .url_repo
            .find_by_user_and_short_code(user.id, short_code.clone())
            .await
            .ok_or(UrlError::ShortCodeNotFound)?;

        if let Some(existing) = self
            .url_repo
            .find_by_user_and_long_url(user.id, new_url.clone())
            .await
        {
            if existing.expires_at > OffsetDateTime::now_utc() {
                return Err(UrlError::UrlAlreadyExists.into());
            }
        }

        let updated = self
            .url_repo
            .update_long_url(user.id, short_code, new_url)
            .await?
            .ok_or(UrlError::ShortCodeNotFound)?;

        let dto = UrlReadDto::from(updated);

        self.url_cache
            .delete_by_user_and_long_url(user.id, &url.original_url)
            .await;
        self.url_cache.set_url(Some(user.id), &dto).await;

        Ok(dto)
    }

    pub async fn redirect(&self, short_code: String) -> Result<String, ApiError> {
        if short_code.len() != SHORT_CODE_LENGTH {
            return Err(UrlError::ShortCodeNotFound.into());
        }

        if let Some(url) = self.url_cache.get_by_short_code(&short_code).await {
            if is_alive(&url) {
                return Ok(url.long_url);
            }
        }

        let url = self
            .url_repo
            .find_by_short_code(short_code)
            .await
            .ok_or(UrlError::ShortCodeNotFound)?;

        if url.expires_at <= OffsetDateTime::now_utc() {
            return Err(UrlError::UrlExpired.into());
        }

        let user_id = url.user_id;
        let original_url = url.original_url.clone();
        let dto = UrlReadDto::from(url);

        self.url_cache.set_url(user_id, &dto).await;

        Ok(original_url)
    }

    async fn generate_unique_short_code(
        &self,
        user_id: &Uuid,
        long_url: &str,
    ) -> Result<String, ApiError> {
        let mut salt: Option<String> = None;

        for _ in 0..SHORT_CODE_MAX_ATTEMPTS {
            let short_code = generate_short_code(user_id, long_url, salt.as_deref());

            if self
                .url_repo
                .find_by_short_code(short_code.clone())
                .await
                .is_none()
            {
                return Ok(short_code);
            }

            salt = Some(Uuid::new_v4().to_string());
        }

        Err(UrlError::ShortCodeGenerationFailed.into())
    }
}

fn is_alive(url: &UrlReadDto) -> bool {
    OffsetDateTime::parse(&url.expires_at, &Rfc3339)
        .map(|expires_at| expires_at > OffsetDateTime::now_utc())
        .unwrap_or(false)
}

fn generate_short_code(user_id: &Uuid, long_url: &str, salt: Option<&str>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(user_id.to_string());
    hasher.update(b"|");
    hasher.update(long_url);
    if let Some(salt) = salt {
        hasher.update(b"|");
        hasher.update(salt);
    }

    let digest = hasher.finalize();

    digest
        .iter()
        .map(|byte| BASE62_ALPHABET[(*byte as usize) % BASE62_ALPHABET.len()] as char)
        .take(SHORT_CODE_LENGTH)
        .collect()
}
