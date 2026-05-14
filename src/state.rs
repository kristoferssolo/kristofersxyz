use axum::extract::FromRef;
use leptos::prelude::LeptosOptions;
use sqlx::{
    PgPool,
    migrate::{MigrateError, Migrator},
    postgres::PgPoolOptions,
};
use thiserror::Error;

use crate::configuration::DatabaseSettings;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub pool: PgPool,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

#[derive(Debug, Error)]
pub enum AppStateError {
    #[error("failed to connect to PostgreSQL database")]
    Connect(#[source] sqlx::Error),
    #[error("failed to run database migrations")]
    Migrate(#[source] MigrateError),
}

/// Builds the shared application state, establishes the `PostgreSQL` pool,
/// and runs embedded migrations.
///
/// # Errors
///
/// Returns an error if `PostgreSQL` cannot be reached or migrations fail.
pub async fn build_app_state(
    leptos_options: LeptosOptions,
    database: &DatabaseSettings,
) -> Result<AppState, AppStateError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(database.connect_options())
        .await
        .map_err(AppStateError::Connect)?;

    MIGRATOR.run(&pool).await.map_err(AppStateError::Migrate)?;

    Ok(AppState {
        leptos_options,
        pool,
    })
}
