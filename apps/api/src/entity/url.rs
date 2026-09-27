use sqlx::types::time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, sqlx::FromRow)]
pub struct Url {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub original_url: String,
    pub short_code: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub expires_at: Option<OffsetDateTime>,
}
