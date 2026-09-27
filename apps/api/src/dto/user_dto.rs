use log::error;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

use crate::entity::user::User;

#[derive(Clone, Serialize, Deserialize)]
pub struct UserLoginDto {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserRegisterDto {
    pub name: Uuid,
    pub email: String,
    pub phone: String,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserReadDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub expires_at: String,
}

impl From<User> for UserReadDto {
    fn from(model: User) -> Self {
        Self {
            id: model.id,
            name: model.name,
            email: model.email,
            phone: model.phone,
            is_active: model.is_active,
            created_at: model.created_at.format(&Rfc3339).unwrap_or_else(|e| {
                error!("Failed to format updated_at (user_id={}): {}", model.id, e);
                "".to_string()
            }),
            updated_at: model.updated_at.format(&Rfc3339).unwrap_or_else(|e| {
                error!("Failed to format updated_at (user_id={}): {}", model.id, e);
                "".to_string()
            }),
            expires_at: model.expires_at.format(&Rfc3339).unwrap_or_else(|e| {
                error!("Failed to format updated_at (user_id={}): {}", model.id, e);
                "".to_string()
            }),
        }
    }
}

impl std::fmt::Debug for UserLoginDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User").field("email", &self.email).finish()
    }
}

impl std::fmt::Debug for UserRegisterDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("name", &self.name)
            .field("email", &self.email)
            .field("phone", &self.phone)
            .finish()
    }
}
