use axum::{
    extract::{Query, State},
    http::StatusCode,
    Extension, Json,
};
use crate::dto::url_dto::{UrlCreateDto, UrlEditDto, UrlQueryDto, UrlReadDto};
use crate::entity::user::User;
use crate::error::api_error::ApiError;
use crate::state::url_state::UrlState;
use crate::utils::api_response::ApiSuccessResponse;

pub async fn create_url(
    State(state): State<UrlState>,
    Extension(user): Extension<User>,
    Json(payload): Json<UrlCreateDto>,
) -> Result<(StatusCode, Json<ApiSuccessResponse<UrlReadDto>>), ApiError> {
    let (url, created) = state
        .url_service
        .create_url(&user, payload.long_url)
        .await?;

    let status = if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };

    Ok((status, Json(ApiSuccessResponse::send(url))))
}

pub async fn get_url(
    State(state): State<UrlState>,
    Extension(user): Extension<User>,
    Query(query): Query<UrlQueryDto>,
) -> Result<Json<ApiSuccessResponse<UrlReadDto>>, ApiError> {
    let url = state.url_service.get_url(&user, query.long_url).await?;
    Ok(Json(ApiSuccessResponse::send(url)))
}

pub async fn get_urls(
    State(state): State<UrlState>,
    Extension(user): Extension<User>,
) -> Result<Json<ApiSuccessResponse<Vec<UrlReadDto>>>, ApiError> {
    let urls = state.url_service.get_urls(&user).await?;
    Ok(Json(ApiSuccessResponse::send(urls)))
}

pub async fn delete_url(
    State(state): State<UrlState>,
    Extension(user): Extension<User>,
    Query(query): Query<UrlQueryDto>,
) -> Result<StatusCode, ApiError> {
    state.url_service.delete_url(&user, query.long_url).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn edit_url(
    State(state): State<UrlState>,
    Extension(user): Extension<User>,
    Json(payload): Json<UrlEditDto>,
) -> Result<Json<ApiSuccessResponse<UrlReadDto>>, ApiError> {
    let url = state
        .url_service
        .edit_url(&user, payload.short_code, payload.new_url)
        .await?;
    Ok(Json(ApiSuccessResponse::send(url)))
}
