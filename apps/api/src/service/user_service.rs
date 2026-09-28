use crate::{
    config::database::Database,
    dto::user_dto::{UserReadDto, UserRegisterDto},
    entity::user::User,
    error::{api_error::ApiError, db_error::DbError, user_error::UserError},
    repository::user_repository::{UserRepository, UserRepositoryTrait},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct UserService {
    user_repo: UserRepository,
}

impl UserService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            user_repo: UserRepository::new(db_conn),
        }
    }

    pub async fn create_user(&self, payload: UserRegisterDto) -> Result<UserReadDto, ApiError> {
        match self.user_repo.find_by_email(payload.email.to_owned()).await {
            Some(_) => Err(UserError::UserAlreadyExists)?,
            None => {
                let mut payload = payload;
                payload.phone = payload
                    .phone
                    .map(|phone| phone.trim().to_owned())
                    .filter(|phone| !phone.is_empty());
                payload.password = bcrypt::hash(payload.password, 4).unwrap();
                match self.user_repo.create(payload).await {
                    Ok(user) => Ok(UserReadDto::from(user)),
                    // Race guard: if two requests with the same email slip past
                    // the find_by_email check, Postgres raises 23505.
                    Err(DbError::UniqueConstraintViolation(msg))
                        if msg.contains("users_email_key")
                            || msg.contains("email") =>
                    {
                        Err(UserError::UserAlreadyExists.into())
                    }
                    Err(e) => Err(e.into()),
                }
            }
        }
    }

    pub fn verify_password(&self, user: &User, password: &str) -> bool {
        bcrypt::verify(password, &user.password).unwrap_or(false)
    }
}
