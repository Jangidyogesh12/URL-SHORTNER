pub use shared::{UrlCreateDto, UrlEditDto, UrlQueryDto, UrlReadDto};
use tracing::error;
use time::format_description::well_known::Rfc3339;

use crate::entity::url::Url;

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
            expires_at: model.expires_at.map(|expires_at| {
                expires_at.format(&Rfc3339).unwrap_or_else(|e| {
                    error!("Failed to format expires_at (url_id={}): {}", model.id, e);
                    "".to_string()
                })
            }),
        }
    }
}
