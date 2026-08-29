use super::credentials::AuthError;
use password_auth::{VerifyError, generate_hash, is_hash_obsolete, verify_password};
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

    pub(crate) fn expose_secret(&self) -> &str {
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
#[derive(Clone, Debug)]
pub struct PasswordHash(SecretString);

impl PasswordHash {
    /// Parses a PHC-formatted password hash loaded from storage.
    ///
    /// # Errors
    ///
    /// Returns [`PasswordHashError`] if `value` is not a valid PHC string.
    pub fn new(value: String) -> Result<Self, PasswordHashError> {
        is_hash_obsolete(&value).map_err(PasswordHashError::InvalidFormat)?;
        Ok(Self(SecretString::from(value)))
    }

    pub(crate) fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl TryFrom<String> for PasswordHash {
    type Error = PasswordHashError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum PasswordHashError {
    #[error("the stored password hash is invalid")]
    InvalidFormat(#[source] password_auth::ParseError),
}

/// Hashes a password using `password-auth`'s current policy and a fresh salt.
#[must_use]
#[tracing::instrument(name = "Compute owner password hash", skip_all)]
pub fn compute_password_hash(password: &Password) -> PasswordHash {
    PasswordHash(SecretString::from(generate_hash(password.expose_secret())))
}

/// Verifies a candidate and returns a replacement when the PHC parameters no
/// longer match the current policy.
#[tracing::instrument(name = "Verify owner password hash", skip_all, err)]
pub(super) fn verify_password_hash(
    expected: &PasswordHash,
    candidate: &Password,
) -> Result<Option<PasswordHash>, AuthError> {
    verify_password(candidate.expose_secret(), expected.expose_secret()).map_err(|error| {
        match error {
            VerifyError::PasswordInvalid => AuthError::InvalidCredentials,
            VerifyError::Parse(error) => {
                AuthError::PasswordHash(PasswordHashError::InvalidFormat(error))
            }
        }
    })?;

    Ok(is_hash_obsolete(expected.expose_secret())
        .map_err(PasswordHashError::InvalidFormat)?
        .then(|| compute_password_hash(candidate)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authentication::test_support::obsolete_password_hash;
    use claims::{assert_err, assert_ok, assert_ok_eq, assert_some};

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

    #[test]
    fn new_hashes_use_the_current_argon2id_policy() {
        let password = assert_ok!(Password::try_from(
            "correct horse battery staple".to_owned()
        ));
        let hash = compute_password_hash(&password);

        assert_ok_eq!(is_hash_obsolete(hash.expose_secret()), false);
    }

    #[test]
    fn password_hashes_require_phc_format() {
        assert_err!(PasswordHash::try_from("not a password hash".to_owned()));
    }

    #[test]
    fn a_valid_old_hash_is_replaced_after_verification() {
        let password = assert_ok!(Password::try_from("password".to_owned()));
        let old_hash = obsolete_password_hash();

        let replacement = assert_some!(assert_ok!(verify_password_hash(&old_hash, &password)));
        assert_ok_eq!(is_hash_obsolete(replacement.expose_secret()), false);
    }
}
