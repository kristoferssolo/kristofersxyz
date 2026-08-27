use crate::{
    authentication::LoginThrottle, configuration::Settings, db, db::DbPool,
    errors::ApplicationError, router::route,
};
use axum::extract::FromRef;
use leptos::{config::errors::LeptosConfigError, prelude::*};
use sqlx::migrate::MigrateError;
use std::net::SocketAddr;
use tokio::{net::TcpListener, task::JoinHandle};

#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error("failed to connect to or query the database")]
    Database(#[from] sqlx::Error),

    #[error("failed to load portfolio content")]
    Portfolio(#[from] db::portfolio::LoadError),

    #[error("failed to run migrations")]
    Migration(#[from] MigrateError),

    #[error("failed to load Leptos configuration")]
    LeptosConfiguration(#[from] LeptosConfigError),
}

#[derive(Debug, Clone)]
pub struct App {
    /// Shared by login and content-edit requests after startup.
    pub pool: DbPool,
    pub leptos_options: LeptosOptions,
    /// Cookie policy used when the router builds its session layer.
    pub secure_cookie: bool,
    pub login_throttle: LoginThrottle,
}

pub type AppState = App;

#[derive(Debug)]
pub struct Application {
    port: u16,
    server: JoinHandle<Result<(), std::io::Error>>,
}

impl App {
    /// Connects to SQLite, prepares its content, and stores the portfolio for
    /// server rendering.
    ///
    /// # Errors
    ///
    /// Returns [`StartupError`] if the database cannot be reached, a migration
    /// fails, the content cannot be loaded, or the Leptos configuration cannot
    /// be initialized.
    pub async fn new(settings: &Settings) -> Result<Self, StartupError> {
        let pool = db::connect(&settings.database.url).await?;
        db::migrate(&pool).await?;
        db::seed_if_empty(&pool).await?;
        let content = db::portfolio::load(&pool).await?;

        crate::app::content::store_server_content(content);

        let leptos_options = get_configuration(None)?.leptos_options;
        Ok(Self {
            pool,
            leptos_options,
            secure_cookie: settings.session.secure_cookie,
            login_throttle: LoginThrottle::default(),
        })
    }
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

impl FromRef<AppState> for DbPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl Application {
    /// Builds the Axum router from application state.
    ///
    /// # Errors
    ///
    /// Returns [`ApplicationError`] if the listener cannot bind.
    pub async fn build(app: App) -> Result<Self, ApplicationError> {
        let addr = app.leptos_options.site_addr;
        let listener = TcpListener::bind(addr).await?;
        let port = listener.local_addr()?.port();
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                route(app).into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await
        });

        Ok(Self { port, server })
    }

    #[must_use]
    #[inline]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Runs the application until it is stopped.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if the server task fails.
    #[inline]
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::{DatabaseSettings, SessionSettings};
    use claims::assert_ok;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn a_fresh_database_boots_with_portfolio_content() {
        let database = NamedTempFile::new().expect("create a temporary database");
        let settings = Settings {
            database: DatabaseSettings {
                url: format!("sqlite://{}", database.path().display()),
            },
            session: SessionSettings {
                secure_cookie: false,
            },
        };

        assert_ok!(App::new(&settings).await);
        assert_eq!(crate::app::content::server_content().projects.len(), 3);
    }
}
