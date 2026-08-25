use serde::Deserialize;
use std::env;

#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("missing environment variable {name}")]
    MissingEnvironmentVariable {
        name: &'static str,
        #[source]
        source: env::VarError,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub session: SessionSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SessionSettings {
    /// Whether the session cookie carries the `Secure` attribute, which keeps
    /// the browser from sending it over plain HTTP. On by default; a local HTTP
    /// deployment opts out with `SESSION_COOKIE_SECURE=false`.
    pub secure_cookie: bool,
}

impl Settings {
    /// Loads application settings from environment variables.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigurationError`] when a required environment variable is
    /// missing.
    pub fn from_env() -> Result<Self, ConfigurationError> {
        Ok(Self {
            database: DatabaseSettings::from_env()?,
            session: SessionSettings::from_env(),
        })
    }
}

impl DatabaseSettings {
    /// The database is the source of truth for the portfolio, so `DATABASE_URL`
    /// is required: without it there is nothing to render.
    fn from_env() -> Result<Self, ConfigurationError> {
        let url = env::var("DATABASE_URL").map_err(|source| {
            ConfigurationError::MissingEnvironmentVariable {
                name: "DATABASE_URL",
                source,
            }
        })?;
        Ok(Self { url })
    }
}

impl SessionSettings {
    /// Reads the cookie policy from the environment. Unset means secure, so a
    /// forgotten variable fails closed rather than leaking the cookie over HTTP;
    /// only an explicit `false` turns it off, and an unparseable value stays
    /// secure.
    fn from_env() -> Self {
        let secure_cookie = env::var("SESSION_COOKIE_SECURE")
            .map_or(true, |value| value.trim().parse().unwrap_or(true));
        Self { secure_cookie }
    }
}
