use dotenv::dotenv;
use tracing::{error, info};
use std::env;

pub fn init() -> Result<(), String> {
    match dotenv() {
        Ok(_) => {
            info!("Loading .env file");
            Ok(())
        }
        Err(e) => {
            info!(".env file not found, relying on environment variables: {}", e);
            Ok(())
        }
    }
}

pub fn get(parameter: &str) -> Result<String, String> {
    match env::var(parameter) {
        Ok(value) => Ok(value),
        Err(_) => {
            let msg = format!("environment valiable {} is not set", parameter);
            error!("{}", msg);
            Err(msg)
        }
    }
}
