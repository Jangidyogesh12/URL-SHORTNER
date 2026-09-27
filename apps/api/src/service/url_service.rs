use crate::{
    config::database::Database,
    dto::url_dto::UrlReadDto,
    entity::{url::Url, user::User},
    error::{api_error::ApiError, url_error::UrlError},
    repository::url_repository::{UrlRepository, UrlRepositoryTrait},
};
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime};
use std::sync::Arc;
use uuid::Uuid;

const SHORT_CODE_LENGTH: usize = 8;
const SHORT_CODE_MAX_ATTEMPTS: usize = 5;
const URL_EXPIRATION_DAYS: i64 = 30;
const BASE62_ALPHABET: &[u8; 62] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

#[derive(Clone)]
pub struct UrlService {
    url_repo: UrlRepository,
}

impl UrlService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            url_repo: UrlRepository::new(db_conn),
        }
    }

    pub async fn create_url(
        &self,
        user: &User,
        long_url: String,
    ) -> Result<(UrlReadDto, bool), ApiError> {
        if let Some(url) = self
            .url_repo
            .find_by_user_and_long_url(user.id, long_url.clone())
            .await
        {
            return Ok((url.into(), false));
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

        Ok((url.into(), true))
    }

    pub async fn get_url(&self, user: &User, long_url: String) -> Result<UrlReadDto, ApiError> {
        let url = self
            .url_repo
            .find_by_user_and_long_url(user.id, long_url)
            .await
            .ok_or(UrlError::UrlNotFound)?;

        Ok(url.into())
    }

    pub async fn get_urls(&self, user: &User) -> Result<Vec<UrlReadDto>, ApiError> {
        let urls = self.url_repo.find_all_by_user(user.id).await?;
        Ok(urls.into_iter().map(UrlReadDto::from).collect())
    }

    pub async fn delete_url(&self, user: &User, long_url: String) -> Result<(), ApiError> {
        let rows_affected = self
            .url_repo
            .delete_by_user_and_long_url(user.id, long_url)
            .await?;

        match rows_affected {
            0 => Err(UrlError::UrlNotFound.into()),
            _ => Ok(()),
        }
    }

    pub async fn edit_url(
        &self,
        user: &User,
        short_code: String,
        new_url: String,
    ) -> Result<UrlReadDto, ApiError> {
        self.url_repo
            .find_by_user_and_short_code(user.id, short_code.clone())
            .await
            .ok_or(UrlError::ShortCodeNotFound)?;

        if self
            .url_repo
            .find_by_user_and_long_url(user.id, new_url.clone())
            .await
            .is_some()
        {
            return Err(UrlError::UrlAlreadyExists.into());
        }

        let url = self
            .url_repo
            .update_long_url(user.id, short_code, new_url)
            .await?
            .ok_or(UrlError::ShortCodeNotFound)?;

        Ok(url.into())
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
