use crate::{
    config::database::Database,
    dto::user_dto::{UserReadDto, UserRegisterDto},
    entity::user::User,
    error::{api_error::ApiError, user_error::UserError},
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
                let user = self.user_repo.create(payload).await?;
                Ok(UserReadDto::from(user))
            }
        }
    }

    pub fn verify_password(&self, user: &User, password: &str) -> bool {
        bcrypt::verify(password, &user.password).unwrap_or(false)
    }
}
