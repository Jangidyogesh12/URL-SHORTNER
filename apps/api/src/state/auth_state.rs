use std::sync::Arc;

use crate::{
    config::database::Database,
    repository::user_repository::{UserRepository, UserRepositoryTrait},
    service::{
        token_service::{TokenService, TokenServiceTrait},
        user_service::UserService,
    },
};

#[derive(Clone)]
pub struct AuthState {
    pub user_repo: UserRepository,
    pub user_service: UserService,
    pub token_service: TokenService,
}

impl AuthState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            user_repo: UserRepository::new(db_conn),
            user_service: UserService::new(db_conn),
            token_service: TokenService::new(),
        }
    }
}
