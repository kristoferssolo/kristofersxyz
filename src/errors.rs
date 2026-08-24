#[cfg(feature = "ssr")]
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("failed to load configuration")]
    Configuration(#[from] crate::configuration::ConfigurationError),
    #[error("failed to start application")]
    Startup(#[from] crate::startup::StartupError),
    #[error("admin command failed")]
    AdminCli(#[from] crate::admin_cli::AdminCliError),
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}
