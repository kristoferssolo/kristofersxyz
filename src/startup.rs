use crate::{
    configuration::Settings,
    db::{self},
};
use axum::Router;
use sqlx::SqlitePool;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StartupError {
    #[error("failed to connect to database")]
    DatabaseConnection(#[from] sqlx::Error),
}

pub struct App {
    pub pool: SqlitePool,
}

pub async fn build_app(settings: Settings) -> Result<Router, StartupError> {
    let db_pool = db::connect(&settings.database.url).await?;
    todo!()
}
