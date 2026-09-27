use crate::utils::api_response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UrlError {
    #[error("Url not found")]
    UrlNotFound,
    #[error("Short code not found")]
    ShortCodeNotFound,
    #[error("Url already exists")]
    UrlAlreadyExists,
    #[error("Invalid link lifetime")]
    InvalidLifetime,
    #[error("Url has expired")]
    UrlExpired,
    #[error("Failed to generate a unique short code")]
    ShortCodeGenerationFailed,
}

impl IntoResponse for UrlError {
    fn into_response(self) -> Response {
        let status_code = match self {
            UrlError::UrlNotFound => StatusCode::NOT_FOUND,
            UrlError::ShortCodeNotFound => StatusCode::NOT_FOUND,
            UrlError::UrlAlreadyExists => StatusCode::CONFLICT,
            UrlError::InvalidLifetime => StatusCode::BAD_REQUEST,
            UrlError::UrlExpired => StatusCode::GONE,
            UrlError::ShortCodeGenerationFailed => StatusCode::INTERNAL_SERVER_ERROR,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
