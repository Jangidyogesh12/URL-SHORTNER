use crate::config::database::{Database, DatabaseTrait};
use crate::dto::user_dto::UserRegisterDto;
use crate::entity::user::User;
use crate::error::db_error::DbError;
use async_trait::async_trait;
use sqlx::Error as SqlxError;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserRepository {
    pub(crate) db_conn: Arc<Database>,
}

#[async_trait]
pub trait UserRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn create(&self, payload: UserRegisterDto) -> Result<User, DbError>;
    async fn find_by_email(&self, email: String) -> Option<User>;
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }
    async fn create(&self, payload: UserRegisterDto) -> Result<User, DbError> {
        let user = sqlx::query_as::<_, User>(
            r#"
        INSERT INTO users (
            id,
            name,
            email,
            phone,
            password
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
            id,
            name,
            email,
            phone,
            password,
            is_active,
            created_at,
            updated_at
        "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(payload.name)
        .bind(payload.email)
        .bind(payload.phone)
        .bind(payload.password)
        .fetch_one(self.db_conn.get_pool())
        .await
        .map_err(|e| match e {
            SqlxError::Database(e) => match e.code() {
                Some(code) if code == "23505" => DbError::UniqueConstraintViolation(e.to_string()),
                _ => DbError::SomethingWentWrong(e.to_string()),
            },
            _ => DbError::SomethingWentWrong(e.to_string()),
        })?;

        Ok(user)
    }

    async fn find_by_email(&self, email: String) -> Option<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(self.db_conn.get_pool())
            .await
            .unwrap_or(None);

        user
    }
}
