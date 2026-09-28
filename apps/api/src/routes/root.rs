use std::sync::Arc;

use axum::{routing::get, Router};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::{
    config::{cache::Cache, database::Database},
    handler::url_handler,
    routes::{auth, register, url},
    state::{
        auth_state::AuthState, token_state::TokenState, url_state::UrlState, user_state::UserState,
    },
};

pub fn routes(db_conn: Arc<Database>, cache_conn: Arc<Cache>) -> Router {
    let url_state = UrlState::new(&db_conn, &cache_conn);

    let merged_router = {
        let user_state = UserState::new(&db_conn);
        let token_state = TokenState::new(&db_conn);
        let auth_state = AuthState::new(&db_conn);

        auth::routes()
            .with_state(auth_state)
            .merge(register::routes().with_state(user_state))
            .merge(url::routes(url_state.clone(), token_state))
    };

    let redirect_router = Router::new()
        .route("/{short_code}", get(url_handler::redirect))
        .with_state(url_state);

    let app_router = Router::new()
        .nest("/api", merged_router)
        .merge(redirect_router)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    app_router
}
