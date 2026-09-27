use axum::{
    middleware::from_fn_with_state,
    routing::{delete, get, post, put},
    Router,
};

use crate::{
    handler::url_handler,
    middleware,
    state::{token_state::TokenState, url_state::UrlState},
};

pub fn routes(url_state: UrlState, token_state: TokenState) -> Router {
    let route = Router::new()
        .route("/url", get(url_handler::get_url))
        .route("/urls", get(url_handler::get_urls))
        .route("/create_url", post(url_handler::create_url))
        .route("/delete_url", delete(url_handler::delete_url))
        .route("/edit_url", put(url_handler::edit_url))
        .route_layer(from_fn_with_state(
            token_state,
            middleware::auth::auth,
        ))
        .with_state(url_state);

    route
}
