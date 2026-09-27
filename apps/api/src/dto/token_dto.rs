use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct TokenReadDto {
    pub token: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TokenClaimsDto {
    pub sub: Uuid,
    pub email: String,
    pub iat: i64,
    pub exp: i64,
}
