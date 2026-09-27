use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Response returned after a successful login.
#[derive(Clone, Serialize, Deserialize, TS)]
pub struct TokenReadDto {
    pub token: String,
    pub iat: i64,
    pub exp: i64,
}

/// Claims embedded in an authentication token.
#[derive(Clone, Serialize, Deserialize, TS)]
pub struct TokenClaimsDto {
    pub sub: Uuid,
    pub email: String,
    pub iat: i64,
    pub exp: i64,
}

/// Credentials submitted by an existing user.
#[derive(Clone, Serialize, Deserialize, TS)]
pub struct UserLoginDto {
    pub email: String,
    pub password: String,
}

/// Profile fields required to create a user.
#[derive(Clone, Serialize, Deserialize, TS)]
pub struct UserRegisterDto {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub password: String,
}

/// User fields returned to API clients.
#[derive(Clone, Serialize, Deserialize, TS)]
pub struct UserReadDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Website submitted for shortening.
#[derive(Clone, Serialize, Deserialize, TS)]
pub struct UrlCreateDto {
    pub long_url: String,
}

/// Website used when looking up or deleting one URL.
#[derive(Clone, Serialize, Deserialize, TS)]
pub struct UrlQueryDto {
    pub long_url: String,
}

/// Existing short code, replacement destination, and replacement lifetime.
/// A null lifetime means the short URL never expires.
#[derive(Clone, Serialize, Deserialize, TS)]
pub struct UrlEditDto {
    pub short_code: String,
    pub new_url: String,
    pub expires_in_minutes: Option<i64>,
}

/// Short URL returned to API clients. A null expiry means no expiry.
#[derive(Clone, Serialize, Deserialize, TS)]
pub struct UrlReadDto {
    pub id: Uuid,
    pub long_url: String,
    pub short_code: String,
    pub created_at: String,
    pub updated_at: String,
    pub expires_at: Option<String>,
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
            .field("expires_in_minutes", &self.expires_in_minutes)
            .finish()
    }
}

pub fn export_all() -> Result<(), ts_rs::ExportError> {
    let out_dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../packages/shared-types/src/generated"
    );
    let cfg = ts_rs::Config::from_env().with_out_dir(out_dir);

    TokenReadDto::export_all(&cfg)?;
    TokenClaimsDto::export_all(&cfg)?;
    UserLoginDto::export_all(&cfg)?;
    UserRegisterDto::export_all(&cfg)?;
    UserReadDto::export_all(&cfg)?;
    UrlCreateDto::export_all(&cfg)?;
    UrlQueryDto::export_all(&cfg)?;
    UrlEditDto::export_all(&cfg)?;
    UrlReadDto::export_all(&cfg)?;
    Ok(())
}
