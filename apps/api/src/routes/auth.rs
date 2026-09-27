use axum::{routing::post, Router};

use crate::{handler::auth_handler, state::auth_state::AuthState};

pub fn routes() -> Router<AuthState> {
    let route = Router::new().route("/auth", post(auth_handler::auth));
    route
}
