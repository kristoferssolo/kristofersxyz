use crate::{
    configuration::Settings,
    db::{self, DbPool},
    errors::ApplicationError,
    router::route,
};
use axum::extract::FromRef;
use leptos::{config::errors::LeptosConfigError, prelude::*};
use tokio::{net::TcpListener, task::JoinHandle};

#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error("failed to connect to database")]
    DatabaseConnection(#[from] sqlx::Error),

    #[error("failed to load Leptos configuration")]
    LeptosConfiguration(#[from] LeptosConfigError),
}

#[derive(Debug, Clone)]
pub struct App {
    pub pool: DbPool,
    pub leptos_options: LeptosOptions,
}

pub type AppState = App;

#[derive(Debug)]
pub struct Application {
    port: u16,
    server: JoinHandle<Result<(), std::io::Error>>,
}

impl App {
    /// Builds the shared application state.
    ///
    /// # Errors
    ///
    /// Returns [`StartupError`] if the database connection or Leptos
    /// configuration cannot be initialized.
    pub async fn new(settings: &Settings) -> Result<Self, StartupError> {
        let pool = db::connect(&settings.database.url).await?;
        let leptos_options = get_configuration(None)?.leptos_options;
        Ok(Self {
            pool,
            leptos_options,
        })
    }
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

impl Application {
    /// Builds the Axum router from application state.
    ///
    /// # Errors
    ///
    /// - Returns [`ApplicationError`] if:
    ///   - It fails to bind to the specified address.
    pub async fn build(app: App) -> Result<Self, ApplicationError> {
        let addr = app.leptos_options.site_addr;
        let listener = TcpListener::bind(addr).await?;
        let port = listener.local_addr()?.port();
        let server =
            tokio::spawn(
                async move { axum::serve(listener, route(app).into_make_service()).await },
            );

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
    /// - Returns `std::io::Error` if the server task encounters an error.
    #[inline]
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await?
    }
}
