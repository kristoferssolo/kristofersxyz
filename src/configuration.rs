use serde::Deserialize;
use std::{env, str::FromStr};

use axum::http::Uri;

#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("missing environment variable {name}")]
    MissingEnvironmentVariable {
        name: &'static str,
        #[source]
        source: env::VarError,
    },
    #[error("invalid PUBLIC_ORIGIN")]
    InvalidPublicOrigin(#[from] PublicOriginError),
    #[error("invalid DEPLOYMENT_MODE '{0}'")]
    InvalidDeploymentMode(String),
    #[error("local deployment requires an http PUBLIC_ORIGIN")]
    SecureLocalOrigin,
    #[error("production deployment requires an https PUBLIC_ORIGIN")]
    InsecureProductionOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub deployment: DeploymentMode,
    pub http: HttpSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HttpSettings {
    pub public_origin: PublicOrigin,
}

/// How the application reaches browsers. Production traffic must arrive
/// through a controlled TLS terminator because this binary serves plain HTTP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeploymentMode {
    Local,
    ProductionBehindTrustedProxy,
}

impl DeploymentMode {
    #[must_use]
    pub const fn session_cookie(self) -> SessionCookiePolicy {
        match self {
            Self::Local => SessionCookiePolicy {
                name: "kristofersxyz-session",
                secure: false,
            },
            Self::ProductionBehindTrustedProxy => SessionCookiePolicy {
                name: "__Host-kristofersxyz-session",
                secure: true,
            },
        }
    }
}

impl FromStr for DeploymentMode {
    type Err = ConfigurationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "local" => Ok(Self::Local),
            "production-behind-trusted-proxy" => Ok(Self::ProductionBehindTrustedProxy),
            other => Err(ConfigurationError::InvalidDeploymentMode(other.to_owned())),
        }
    }
}

impl<'de> Deserialize<'de> for DeploymentMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        crate::serde_helpers::deserialize_from_str(deserializer)
    }
}

/// Cookie attributes derived from the deployment mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionCookiePolicy {
    name: &'static str,
    secure: bool,
}

impl SessionCookiePolicy {
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    #[must_use]
    pub const fn secure(self) -> bool {
        self.secure
    }
}

/// The canonical scheme, host, and optional port accepted for browser writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicOrigin(String);

impl PublicOrigin {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn is_https(&self) -> bool {
        self.0.starts_with("https://")
    }

    #[must_use]
    pub fn matches_referer(&self, referer: &str) -> bool {
        Uri::from_str(referer).is_ok_and(|uri| {
            uri.scheme_str()
                .zip(uri.authority())
                .is_some_and(|(scheme, authority)| format!("{scheme}://{authority}") == self.0)
        })
    }
}

impl FromStr for PublicOrigin {
    type Err = PublicOriginError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let uri = Uri::from_str(value)?;
        let scheme = uri.scheme_str().ok_or(PublicOriginError::MissingScheme)?;
        if !matches!(scheme, "http" | "https") {
            return Err(PublicOriginError::UnsupportedScheme);
        }
        let authority = uri.authority().ok_or(PublicOriginError::MissingAuthority)?;
        if authority.as_str().contains('@') {
            return Err(PublicOriginError::Credentials);
        }
        if uri
            .path_and_query()
            .is_some_and(|path| path.as_str() != "/")
        {
            return Err(PublicOriginError::PathOrQuery);
        }

        Ok(Self(format!("{scheme}://{authority}")))
    }
}

impl<'de> Deserialize<'de> for PublicOrigin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        crate::serde_helpers::deserialize_from_str(deserializer)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PublicOriginError {
    #[error("the origin is not a valid URI")]
    InvalidUri(#[from] axum::http::uri::InvalidUri),
    #[error("the origin must include a scheme")]
    MissingScheme,
    #[error("the origin scheme must be http or https")]
    UnsupportedScheme,
    #[error("the origin must include a host")]
    MissingAuthority,
    #[error("the origin cannot contain credentials")]
    Credentials,
    #[error("the origin cannot contain a path or query")]
    PathOrQuery,
}

impl Settings {
    /// Creates settings from their three runtime inputs.
    #[must_use]
    pub fn new(
        database_url: impl Into<String>,
        deployment: DeploymentMode,
        public_origin: PublicOrigin,
    ) -> Self {
        Self {
            database: DatabaseSettings {
                url: database_url.into(),
            },
            deployment,
            http: HttpSettings { public_origin },
        }
    }

    /// Loads application settings from environment variables.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigurationError`] when a required environment variable is
    /// missing.
    pub fn from_env() -> Result<Self, ConfigurationError> {
        let settings = Self {
            database: DatabaseSettings::from_env()?,
            deployment: DeploymentMode::from_env()?,
            http: HttpSettings::from_env()?,
        };
        settings.validate()?;
        Ok(settings)
    }

    /// Checks relationships between settings that cannot be validated one
    /// field at a time.
    ///
    /// # Errors
    ///
    /// Returns a configuration error when deployment mode and public origin
    /// disagree about whether browsers use HTTPS.
    pub fn validate(&self) -> Result<(), ConfigurationError> {
        match (self.deployment, self.http.public_origin.is_https()) {
            (DeploymentMode::Local, false)
            | (DeploymentMode::ProductionBehindTrustedProxy, true) => Ok(()),
            (DeploymentMode::Local, true) => Err(ConfigurationError::SecureLocalOrigin),
            (DeploymentMode::ProductionBehindTrustedProxy, false) => {
                Err(ConfigurationError::InsecureProductionOrigin)
            }
        }
    }
}

fn required_env(name: &'static str) -> Result<String, ConfigurationError> {
    env::var(name).map_err(|source| ConfigurationError::MissingEnvironmentVariable { name, source })
}

impl DeploymentMode {
    fn from_env() -> Result<Self, ConfigurationError> {
        required_env("DEPLOYMENT_MODE")?.parse()
    }
}

impl HttpSettings {
    fn from_env() -> Result<Self, ConfigurationError> {
        Ok(Self {
            public_origin: required_env("PUBLIC_ORIGIN")?.parse()?,
        })
    }
}

impl DatabaseSettings {
    /// Reads the required portfolio database URL.
    fn from_env() -> Result<Self, ConfigurationError> {
        Ok(Self {
            url: required_env("DATABASE_URL")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok_eq};

    #[test]
    fn public_origins_are_canonical_and_have_no_path() {
        assert_ok_eq!(
            PublicOrigin::from_str("https://kristofers.xyz/"),
            PublicOrigin("https://kristofers.xyz".to_owned())
        );
        assert_ok_eq!(
            PublicOrigin::from_str("http://localhost:3000"),
            PublicOrigin("http://localhost:3000".to_owned())
        );
        assert_err!(PublicOrigin::from_str("https://kristofers.xyz/admin"));
        assert_err!(PublicOrigin::from_str("ftp://kristofers.xyz"));
        assert_err!(PublicOrigin::from_str("kristofers.xyz"));
    }

    #[test]
    fn deployment_mode_determines_a_valid_cookie_and_origin_pair() {
        let local = Settings::new(
            "sqlite::memory:",
            DeploymentMode::Local,
            PublicOrigin("http://localhost:3000".to_owned()),
        );
        assert!(local.validate().is_ok());
        assert!(!local.deployment.session_cookie().secure());

        let production = Settings::new(
            local.database.url,
            DeploymentMode::ProductionBehindTrustedProxy,
            PublicOrigin("https://kristofers.xyz".to_owned()),
        );
        assert!(production.validate().is_ok());
        assert!(production.deployment.session_cookie().secure());
        assert!(
            production
                .deployment
                .session_cookie()
                .name()
                .starts_with("__Host-")
        );
    }

    #[test]
    fn production_rejects_an_insecure_public_origin() {
        let settings = Settings::new(
            "sqlite::memory:",
            DeploymentMode::ProductionBehindTrustedProxy,
            PublicOrigin("http://kristofers.xyz".to_owned()),
        );
        assert_err!(settings.validate());
    }
}
