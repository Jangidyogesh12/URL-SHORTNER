use axum::{routing::post, Router};

use crate::{handler::register_handler, state::user_state::UserState};

pub fn routes() -> Router<UserState> {
    let route = Router::new().route("/register", post(register_handler::register));
    route
}
