mod config;
mod dto;
mod entity;
mod error;
mod handler;
mod middleware;
mod repository;
mod routes;
mod service;
mod state;
use std::{process::exit, sync::Arc};
use tokio::signal;

use tracing::{error, info};
use tokio::net::TcpListener;

use crate::config::{
    cache::{Cache, CacheTrait},
    database::{Database, DatabaseTrait},
    parameter,
};
mod utils;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    if let Err(e) = parameter::init() {
        error!("Parameter init failed: {}", e);
        exit(1);
    }

    info!("Environment variables loaded");

    let db_conn = Database::init().await.unwrap_or_else(|e| {
        error!("Failed to initialize the database : {}", e);
        exit(1);
    });

    info!("Database Connected");

    let cache_conn = Cache::init().await.unwrap_or_else(|e| {
        error!("Failed to initialize the cache : {}", e);
        exit(1);
    });

    info!("Redis Cache Connected");

    let app_url = parameter::get("APP_URL").unwrap_or_else(|e| {
        error!("{}", e);
        exit(1);
    });

    let app_port = parameter::get("APP_PORT").unwrap_or_else(|e| {
        error!("{}", e);
        exit(1);
    });

    let host = format!("{}:{}", app_url, app_port);

    let listener = TcpListener::bind(&host).await.unwrap_or_else(|e| {
        error!("Error binding address {}: {}", host, e);
        exit(1);
    });

    let app = routes::root::routes(Arc::new(db_conn), Arc::new(cache_conn));

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap_or_else(|e| {
            error!("Server error: {}", e);
        });

    info!("Server stopped");
}

async fn shutdown_signal() {
    signal::ctrl_c().await.unwrap_or_else(|_| {
        error!("Failed to install CTRL+C handler");
        exit(1);
    });
}
