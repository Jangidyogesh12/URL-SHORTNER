use crate::dto::{token_dto::TokenReadDto, user_dto::UserLoginDto};
use crate::error::{api_error::ApiError, user_error::UserError};
use crate::repository::user_repository::UserRepositoryTrait;
use crate::service::token_service::TokenServiceTrait;
use crate::state::auth_state::AuthState;
use crate::utils::api_response::ApiSuccessResponse;
use axum::{extract::State, Json};

pub async fn auth(
    State(state): State<AuthState>,
    Json(payload): Json<UserLoginDto>,
) -> Result<Json<ApiSuccessResponse<TokenReadDto>>, ApiError> {
    let user = state
        .user_repo
        .find_by_email(payload.email)
        .await
        .ok_or(UserError::UserNotFound)?;

    match state.user_service.verify_password(&user, &payload.password) {
        true => Ok(Json(ApiSuccessResponse::send(
            state.token_service.generate_token(user)?,
        ))),
        false => Err(UserError::InvalidPassword)?,
    }
}
