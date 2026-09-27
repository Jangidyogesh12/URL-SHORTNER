use std::sync::Arc;

use crate::{config::database::Database, service::url_service::UrlService};

#[derive(Clone)]
pub struct UrlState {
    pub url_service: UrlService,
}

impl UrlState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            url_service: UrlService::new(db_conn),
        }
    }
}
