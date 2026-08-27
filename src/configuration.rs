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
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub http: HttpSettings,
    pub session: SessionSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HttpSettings {
    pub public_origin: PublicOrigin,
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
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
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
            http: HttpSettings::from_env()?,
            session: SessionSettings::from_env(),
        })
    }
}

impl HttpSettings {
    fn from_env() -> Result<Self, ConfigurationError> {
        let value = env::var("PUBLIC_ORIGIN").map_err(|source| {
            ConfigurationError::MissingEnvironmentVariable {
                name: "PUBLIC_ORIGIN",
                source,
            }
        })?;
        Ok(Self {
            public_origin: value.parse()?,
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
}
