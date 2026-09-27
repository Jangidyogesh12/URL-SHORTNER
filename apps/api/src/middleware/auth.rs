use crate::repository::user_repository::UserRepositoryTrait;
use crate::service::token_service::TokenServiceTrait;
use crate::{
    entity::user::User,
    error::{api_error::ApiError, token_error::TokenError, user_error::UserError},
    state::token_state::TokenState,
};
use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::IntoResponse,
};
use jsonwebtoken::errors::ErrorKind;

pub async fn auth(
    State(state): State<TokenState>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, ApiError> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or(TokenError::MissingToken)?
        .to_str()
        .map_err(|_| TokenError::InvalidToken("invalid header".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(TokenError::InvalidToken("invalid bearer".into()))?;

    let token_data = state
        .token_service
        .retrieve_token_claims(token)
        .map_err(|err| match err.kind() {
            ErrorKind::ExpiredSignature => TokenError::TokenExpired,
            _ => TokenError::InvalidToken(token.to_string()),
        })?;

    let user: User = state
        .user_repo
        .find_by_email(token_data.claims.email)
        .await
        .ok_or(UserError::UserNotFound)?;

    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}
