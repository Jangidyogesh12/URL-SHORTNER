use axum::{extract::State, Json};
use shared::{HealthResponse, HelloResponse, User};

use crate::AppState;

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

pub async fn hello() -> Json<HelloResponse> {
    Json(HelloResponse {
        message: "Hello from Rust + Axum".to_string(),
        service: "api".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub async fn users(State(state): State<AppState>) -> Json<Vec<User>> {
    let rows = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, name, email FROM users ORDER BY id",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    Json(
        rows.into_iter()
            .map(|(id, name, email)| User { id, name, email })
            .collect(),
    )
}
