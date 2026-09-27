use crate::config::database::{Database, DatabaseTrait};
use crate::entity::url::Url;
use crate::error::db_error::DbError;
use async_trait::async_trait;
use sqlx::Error as SqlxError;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct UrlRepository {
    pub(crate) db_conn: Arc<Database>,
}

#[async_trait]
pub trait UrlRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn create(&self, url: Url) -> Result<Url, DbError>;
    async fn find_by_user_and_long_url(&self, user_id: Uuid, long_url: String) -> Option<Url>;
    async fn find_all_by_user(&self, user_id: Uuid) -> Result<Vec<Url>, DbError>;
    async fn find_by_short_code(&self, short_code: String) -> Option<Url>;
    async fn find_by_user_and_short_code(&self, user_id: Uuid, short_code: String) -> Option<Url>;
    async fn delete_by_user_and_long_url(&self, user_id: Uuid, long_url: String)
        -> Result<u64, DbError>;
    async fn update_long_url(
        &self,
        user_id: Uuid,
        short_code: String,
        new_url: String,
    ) -> Result<Option<Url>, DbError>;
}

#[async_trait]
impl UrlRepositoryTrait for UrlRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }

    async fn create(&self, url: Url) -> Result<Url, DbError> {
        let url = sqlx::query_as::<_, Url>(
            r#"
        INSERT INTO urls (
            id,
            user_id,
            original_url,
            short_code,
            created_at,
            updated_at,
            expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING
            id,
            user_id,
            original_url,
            short_code,
            created_at,
            updated_at,
            expires_at
        "#,
        )
        .bind(url.id)
        .bind(url.user_id)
        .bind(url.original_url)
        .bind(url.short_code)
        .bind(url.created_at)
        .bind(url.updated_at)
        .bind(url.expires_at)
        .fetch_one(self.db_conn.get_pool())
        .await
        .map_err(|e| match e {
            SqlxError::Database(e) => match e.code() {
                Some(code) if code == "23505" => DbError::UniqueConstraintViolation(e.to_string()),
                _ => DbError::SomethingWentWrong(e.to_string()),
            },
            _ => DbError::SomethingWentWrong(e.to_string()),
        })?;

        Ok(url)
    }

    async fn find_by_user_and_long_url(&self, user_id: Uuid, long_url: String) -> Option<Url> {
        let url = sqlx::query_as::<_, Url>(
            "SELECT * FROM urls WHERE user_id = $1 AND original_url = $2",
        )
        .bind(user_id)
        .bind(long_url)
        .fetch_optional(self.db_conn.get_pool())
        .await
        .unwrap_or(None);

        url
    }

    async fn find_all_by_user(&self, user_id: Uuid) -> Result<Vec<Url>, DbError> {
        let urls = sqlx::query_as::<_, Url>(
            "SELECT * FROM urls WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(self.db_conn.get_pool())
        .await
        .map_err(|e| DbError::SomethingWentWrong(e.to_string()))?;

        Ok(urls)
    }

    async fn find_by_short_code(&self, short_code: String) -> Option<Url> {
        let url = sqlx::query_as::<_, Url>("SELECT * FROM urls WHERE short_code = $1")
            .bind(short_code)
            .fetch_optional(self.db_conn.get_pool())
            .await
            .unwrap_or(None);

        url
    }

    async fn find_by_user_and_short_code(&self, user_id: Uuid, short_code: String) -> Option<Url> {
        let url = sqlx::query_as::<_, Url>(
            "SELECT * FROM urls WHERE user_id = $1 AND short_code = $2",
        )
        .bind(user_id)
        .bind(short_code)
        .fetch_optional(self.db_conn.get_pool())
        .await
        .unwrap_or(None);

        url
    }

    async fn delete_by_user_and_long_url(
        &self,
        user_id: Uuid,
        long_url: String,
    ) -> Result<u64, DbError> {
        let result = sqlx::query("DELETE FROM urls WHERE user_id = $1 AND original_url = $2")
            .bind(user_id)
            .bind(long_url)
            .execute(self.db_conn.get_pool())
            .await
            .map_err(|e| DbError::SomethingWentWrong(e.to_string()))?;

        Ok(result.rows_affected())
    }

    async fn update_long_url(
        &self,
        user_id: Uuid,
        short_code: String,
        new_url: String,
    ) -> Result<Option<Url>, DbError> {
        let url = sqlx::query_as::<_, Url>(
            r#"
        UPDATE urls
        SET original_url = $3, updated_at = now()
        WHERE user_id = $1 AND short_code = $2
        RETURNING
            id,
            user_id,
            original_url,
            short_code,
            created_at,
            updated_at,
            expires_at
        "#,
        )
        .bind(user_id)
        .bind(short_code)
        .bind(new_url)
        .fetch_optional(self.db_conn.get_pool())
        .await
        .map_err(|e| DbError::SomethingWentWrong(e.to_string()))?;

        Ok(url)
    }
}
