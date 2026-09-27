use std::sync::Arc;

use axum::Router;
use tower_http::trace::TraceLayer;

use crate::{
    config::database::Database,
    routes::{auth, register, url},
    state::{
        auth_state::AuthState, token_state::TokenState, url_state::UrlState, user_state::UserState,
    },
};

pub fn routes(db_conn: Arc<Database>) -> Router {
    let merged_router = {
        let user_state = UserState::new(&db_conn);
        let token_state = TokenState::new(&db_conn);
        let auth_state = AuthState::new(&db_conn);
        let url_state = UrlState::new(&db_conn);

        auth::routes()
            .with_state(auth_state)
            .merge(register::routes().with_state(user_state))
            .merge(url::routes(url_state, token_state))
    };

    let app_router = Router::new()
        .nest("/api", merged_router)
        .layer(TraceLayer::new_for_http());

    app_router
}
