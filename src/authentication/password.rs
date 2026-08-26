use super::credentials::AuthError;
use argon2::{
    Argon2, Params, PasswordHash as ArgonPasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::SaltString,
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
#[derive(Clone)]
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

/// The current Argon2id policy. `argon2`'s defaults are OWASP's minimum:
/// 19 MiB of memory, two iterations, and one lane.
fn password_hasher() -> Argon2<'static> {
    Argon2::default()
}

/// Hashes a password with the current Argon2id policy and a fresh random salt.
///
/// # Errors
///
/// Returns an [`AuthError`] if the hasher cannot be configured or the hash
/// cannot be computed.
#[tracing::instrument(name = "Compute owner password hash", skip_all, err)]
pub fn compute_password_hash(password: &Password) -> Result<PasswordHash, AuthError> {
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let hash = password_hasher()
        .hash_password(password.expose_secret().as_bytes(), &salt)
        .map_err(AuthError::PasswordHash)?
        .to_string();
    Ok(PasswordHash::from(hash))
}

/// Verifies a candidate and returns a replacement when the PHC parameters no
/// longer match the current policy.
#[tracing::instrument(name = "Verify owner password hash", skip_all, err)]
pub(super) fn verify_password_hash(
    expected: &PasswordHash,
    candidate: &Password,
) -> Result<Option<PasswordHash>, AuthError> {
    let expected =
        ArgonPasswordHash::new(expected.expose_secret()).map_err(AuthError::PasswordHash)?;
    password_hasher()
        .verify_password(candidate.expose_secret().as_bytes(), &expected)
        .map_err(|error| match error {
            argon2::password_hash::Error::Password => AuthError::InvalidCredentials,
            other => AuthError::PasswordHash(other),
        })?;

    needs_rehash(&expected)
        .then(|| compute_password_hash(candidate))
        .transpose()
}

fn needs_rehash(hash: &ArgonPasswordHash<'_>) -> bool {
    let current_params = Params::try_from(hash).is_ok_and(|params| {
        params.m_cost() == Params::DEFAULT_M_COST
            && params.t_cost() == Params::DEFAULT_T_COST
            && params.p_cost() == Params::DEFAULT_P_COST
            && matches!(params.output_len(), None | Some(Params::DEFAULT_OUTPUT_LEN))
    });
    hash.algorithm.as_str() != "argon2id"
        || hash.version != Some(u32::from(Version::V0x13))
        || !current_params
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::Algorithm;
    use claims::{assert_err, assert_ok, assert_some};

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
        let hash = assert_ok!(compute_password_hash(&password));
        let parsed = assert_ok!(ArgonPasswordHash::new(hash.expose_secret()));

        assert_eq!(parsed.algorithm.as_str(), "argon2id");
        assert_eq!(parsed.version, Some(u32::from(Version::V0x13)));
        let params = assert_ok!(Params::try_from(&parsed));
        assert_eq!(params.m_cost(), Params::DEFAULT_M_COST);
        assert_eq!(params.t_cost(), Params::DEFAULT_T_COST);
        assert_eq!(params.p_cost(), Params::DEFAULT_P_COST);
    }

    #[test]
    fn a_valid_old_hash_is_replaced_after_verification() {
        let password = assert_ok!(Password::try_from(
            "correct horse battery staple".to_owned()
        ));
        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
        let old_params = assert_ok!(Params::new(15_000, 2, 1, None));
        let old_hash = assert_ok!(
            Argon2::new(Algorithm::Argon2id, Version::V0x13, old_params)
                .hash_password(password.expose_secret().as_bytes(), &salt)
        );
        let old_hash = PasswordHash::from(old_hash.to_string());

        let replacement = assert_some!(assert_ok!(verify_password_hash(&old_hash, &password)));
        let parsed = assert_ok!(ArgonPasswordHash::new(replacement.expose_secret()));
        let params = assert_ok!(Params::try_from(&parsed));
        assert_eq!(params.m_cost(), Params::DEFAULT_M_COST);
        assert_eq!(params.t_cost(), Params::DEFAULT_T_COST);
        assert_eq!(params.p_cost(), Params::DEFAULT_P_COST);
    }
}
