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
    /// Whether the session cookie carries the `Secure` attribute. Defaults to
    /// true. Local HTTP deployments can set `SESSION_COOKIE_SECURE=false`.
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
    /// Reads the required portfolio database URL.
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
    /// Reads the cookie policy. Missing or invalid values default to secure;
    /// only an explicit `false` permits plain HTTP.
    fn from_env() -> Self {
        let secure_cookie = env::var("SESSION_COOKIE_SECURE")
            .map_or(true, |value| value.trim().parse().unwrap_or(true));
        Self { secure_cookie }
    }
}
