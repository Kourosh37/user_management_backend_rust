pub mod api;
pub mod app;
pub mod config;
pub mod domain;
pub mod infra;
pub mod utils;

use crate::config::AppConfig;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: AppConfig,
}