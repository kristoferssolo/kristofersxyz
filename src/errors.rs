#[cfg(feature = "ssr")]
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("failed to load configuration")]
    Configuration(#[from] crate::configuration::ConfigurationError),
    #[error("failed to start application")]
    Startup(#[from] crate::startup::StartupError),
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}
