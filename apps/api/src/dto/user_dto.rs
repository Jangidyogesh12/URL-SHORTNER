pub use shared::{UserLoginDto, UserReadDto, UserRegisterDto};
use tracing::error;
use time::format_description::well_known::Rfc3339;

use crate::entity::user::User;

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
        }
    }
}
