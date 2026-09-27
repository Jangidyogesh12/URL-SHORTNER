use log::error;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

use crate::entity::url::Url;

#[derive(Clone, Serialize, Deserialize)]
pub struct UrlCreateDto {
    pub long_url: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UrlQueryDto {
    pub long_url: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UrlEditDto {
    pub short_code: String,
    pub new_url: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UrlReadDto {
    pub id: Uuid,
    pub long_url: String,
    pub short_code: String,
    pub created_at: String,
    pub updated_at: String,
    pub expires_at: String,
}

impl From<Url> for UrlReadDto {
    fn from(model: Url) -> Self {
        Self {
            id: model.id,
            long_url: model.original_url,
            short_code: model.short_code,
            created_at: model.created_at.format(&Rfc3339).unwrap_or_else(|e| {
                error!("Failed to format created_at (url_id={}): {}", model.id, e);
                "".to_string()
            }),
            updated_at: model.updated_at.format(&Rfc3339).unwrap_or_else(|e| {
                error!("Failed to format updated_at (url_id={}): {}", model.id, e);
                "".to_string()
            }),
            expires_at: model.expires_at.format(&Rfc3339).unwrap_or_else(|e| {
                error!("Failed to format expires_at (url_id={}): {}", model.id, e);
                "".to_string()
            }),
        }
    }
}

impl std::fmt::Debug for UrlCreateDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UrlCreateDto").finish()
    }
}

impl std::fmt::Debug for UrlQueryDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UrlQueryDto").finish()
    }
}

impl std::fmt::Debug for UrlEditDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UrlEditDto")
            .field("short_code", &self.short_code)
            .finish()
    }
}
