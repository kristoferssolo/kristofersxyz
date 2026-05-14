use axum::extract::FromRef;
use leptos::prelude::LeptosOptions;
use sqlx::{
    SqlitePool,
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};
use thiserror::Error;

use crate::configuration::DatabaseSettings;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub pool: SqlitePool,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

#[derive(Debug, Error)]
pub enum AppStateError {
    #[error("failed to create database directory")]
    CreateDatabaseDirectory(#[source] std::io::Error),
    #[error("failed to connect to SQLite database")]
    Connect(#[source] sqlx::Error),
    #[error("failed to run database migrations")]
    Migrate(#[source] sqlx::migrate::MigrateError),
}

/// Builds the shared application state, establishes the `SQLite` pool,
/// and runs embedded migrations.
///
/// # Errors
///
/// Returns an error if the database directory cannot be created, `SQLite`
/// cannot be opened, or migrations fail.
pub async fn build_app_state(
    leptos_options: LeptosOptions,
    database: &DatabaseSettings,
) -> Result<AppState, AppStateError> {
    if let Some(parent) = database.path.parent() {
        std::fs::create_dir_all(parent).map_err(AppStateError::CreateDatabaseDirectory)?;
    }

    let connect_options = SqliteConnectOptions::new()
        .filename(&database.path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await
        .map_err(AppStateError::Connect)?;

    MIGRATOR.run(&pool).await.map_err(AppStateError::Migrate)?;

    Ok(AppState {
        leptos_options,
        pool,
    })
}
