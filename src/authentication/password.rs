use super::credentials::AuthError;
use argon2::{
    Algorithm, Argon2, Params, PasswordHash as ArgonPasswordHash, PasswordHasher, PasswordVerifier,
    Version, password_hash::SaltString,
};
use secrecy::{ExposeSecret, SecretString};

/// The longest password accepted anywhere. A bound on the hasher's input, so a
/// login attempt cannot force Argon2 to chew through an unbounded string.
/// Matches the OWASP guidance used in *Zero to Production in Rust*.
const MAX_PASSWORD_LENGTH: usize = 128;

/// The shortest password the owner may choose. Enforced only when a password is
/// set (see [`Password::ensure_owner_strength`]), never at login, so verifying
/// an existing password never reveals the policy.
const MIN_OWNER_PASSWORD_LENGTH: usize = 15;

/// A plaintext password supplied by the owner.
#[derive(Debug)]
pub struct Password(SecretString);

impl Password {
    /// Creates a password that is non-blank and within the length bound.
    ///
    /// This is the lenient constructor used for both login and creation: it
    /// does not impose the minimum-length policy, so verifying an existing
    /// password never leaks it.
    ///
    /// # Errors
    ///
    /// Returns [`PasswordError::Empty`] if `value` is blank, or
    /// [`PasswordError::TooLong`] if it exceeds `MAX_PASSWORD_LENGTH`.
    pub fn new(value: SecretString) -> Result<Self, PasswordError> {
        let exposed = value.expose_secret();
        if exposed.trim().is_empty() {
            return Err(PasswordError::Empty);
        }
        if exposed.chars().count() > MAX_PASSWORD_LENGTH {
            return Err(PasswordError::TooLong);
        }
        Ok(Self(value))
    }

    /// Checks the strength policy applied when the owner sets a new password.
    ///
    /// # Errors
    ///
    /// Returns [`PasswordError::TooShort`] if the password is shorter than
    /// `MIN_OWNER_PASSWORD_LENGTH`.
    pub fn ensure_owner_strength(&self) -> Result<(), PasswordError> {
        if self.expose_secret().chars().count() < MIN_OWNER_PASSWORD_LENGTH {
            Err(PasswordError::TooShort)
        } else {
            Ok(())
        }
    }

    fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl TryFrom<String> for Password {
    type Error = PasswordError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(SecretString::from(value))
    }
}

impl TryFrom<SecretString> for Password {
    type Error = PasswordError;

    fn try_from(value: SecretString) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum PasswordError {
    #[error("a password cannot be empty")]
    Empty,
    #[error("a password cannot be longer than {MAX_PASSWORD_LENGTH} characters")]
    TooLong,
    #[error("a password must be at least {MIN_OWNER_PASSWORD_LENGTH} characters")]
    TooShort,
}

/// An encoded password hash suitable for persistent storage.
pub struct PasswordHash(SecretString);

impl PasswordHash {
    pub(crate) fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl From<String> for PasswordHash {
    fn from(value: String) -> Self {
        Self(SecretString::from(value))
    }
}

/// Hashes a password with Argon2id and a fresh random salt.
///
/// # Errors
///
/// Returns an [`AuthError`] if the hasher cannot be configured or the hash
/// cannot be computed.
#[tracing::instrument(name = "Compute owner password hash", skip_all, err)]
pub fn compute_password_hash(password: &Password) -> Result<PasswordHash, AuthError> {
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let params = Params::new(15_000, 2, 1, None).map_err(AuthError::Params)?;
    let hash = Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password(password.expose_secret().as_bytes(), &salt)
        .map_err(AuthError::PasswordHash)?
        .to_string();
    Ok(PasswordHash::from(hash))
}

/// Verifies a candidate against the parameters encoded in a PHC hash.
#[tracing::instrument(name = "Verify owner password hash", skip_all, err)]
pub fn verify_password_hash(
    expected: &PasswordHash,
    candidate: &Password,
) -> Result<(), AuthError> {
    let expected =
        ArgonPasswordHash::new(expected.expose_secret()).map_err(AuthError::PasswordHash)?;
    Argon2::default()
        .verify_password(candidate.expose_secret().as_bytes(), &expected)
        .map_err(|error| match error {
            argon2::password_hash::Error::Password => AuthError::InvalidCredentials,
            other => AuthError::PasswordHash(other),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn passwords_require_visible_content() {
        assert_err!(Password::try_from(String::new()));
        assert_err!(Password::try_from(" \t".to_owned()));
        assert_ok!(Password::try_from("correct horse".to_owned()));
    }

    #[test]
    fn passwords_are_bounded_in_length() {
        assert_ok!(Password::try_from("a".repeat(MAX_PASSWORD_LENGTH)));
        assert_err!(Password::try_from("a".repeat(MAX_PASSWORD_LENGTH + 1)));
    }

    #[test]
    fn the_owner_strength_policy_sets_a_minimum() {
        let short = assert_ok!(Password::try_from(
            "a".repeat(MIN_OWNER_PASSWORD_LENGTH - 1)
        ));
        assert_err!(short.ensure_owner_strength());

        let long_enough = assert_ok!(Password::try_from("a".repeat(MIN_OWNER_PASSWORD_LENGTH)));
        assert_ok!(long_enough.ensure_owner_strength());
    }
}
