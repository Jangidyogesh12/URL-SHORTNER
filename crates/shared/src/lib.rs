use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct HelloResponse {
    pub message: String,
    pub service: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}

pub fn export_all() -> Result<(), ts_rs::ExportError> {
    let out_dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../packages/shared-types/src/generated"
    );
    let cfg = ts_rs::Config::from_env().with_out_dir(out_dir);

    HelloResponse::export_all(&cfg)?;
    HealthResponse::export_all(&cfg)?;
    User::export_all(&cfg)?;
    Ok(())
}
