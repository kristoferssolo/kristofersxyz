//! Owner authentication: credential verification and password hashing.
//!
//! Modelled on *Zero to Production in Rust*. The `users` table stores usernames
//! and Argon2 PHC hashes. Unknown usernames still run one hash verification to
//! reduce timing differences that could reveal registered accounts.

mod backend;
mod credentials;
mod password;
mod session;
mod throttle;

pub use crate::domain::OwnerId;
pub use backend::{AuthBackend, AuthenticatedOwner, AxumAuthSession};
pub use credentials::{AuthError, Credentials, validate_credentials};
pub use password::{Password, PasswordError, PasswordHash, compute_password_hash};
pub(crate) use session::{AuthSession, Authenticated, SessionState, Unverified};
pub use throttle::{LoginThrottle, RetryAfter};

#[cfg(test)]
pub(crate) mod test_support {
    use super::{Credentials, Password, PasswordHash};
    use crate::domain::Username;
    use argon2::{Argon2, Params, PasswordHasher, Version, password_hash::SaltString};
    use claims::assert_ok;
    use secrecy::SecretString;

    pub fn credentials(username: &str, password: &str) -> Credentials {
        Credentials {
            username: assert_ok!(Username::new(username.to_owned())),
            password: assert_ok!(Password::new(SecretString::from(password.to_owned()))),
        }
    }

    pub fn old_password_hash(password: &Password) -> PasswordHash {
        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
        let params = assert_ok!(Params::new(15_000, 2, 1, None));
        let hash = assert_ok!(
            Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params,)
                .hash_password(password.expose_secret().as_bytes(), &salt)
        );
        PasswordHash::from(hash.to_string())
    }
}
