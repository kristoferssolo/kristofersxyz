//! Owner authentication: credential verification and password hashing.
//!
//! Modelled on *Zero to Production in Rust*. The `users` table stores usernames
//! and PHC password hashes managed by `password-auth`. Unknown usernames still
//! run one hash verification to reduce timing differences that could reveal
//! registered accounts.

mod backend;
mod credentials;
mod password;
mod session;
mod throttle;

pub use crate::domain::OwnerId;
pub use backend::{AuthBackend, AuthenticatedOwner, AxumAuthSession};
pub use credentials::{AuthError, Credentials, validate_credentials};
pub use password::{
    Password, PasswordError, PasswordHash, PasswordHashError, compute_password_hash,
};
pub(crate) use session::{
    Authenticated, OwnerSession, OwnerSessionError, SessionState, Unverified,
};
pub use throttle::{LoginThrottle, RetryAfter};

#[cfg(test)]
pub(crate) mod test_support {
    use super::{Credentials, Password, PasswordHash};
    use crate::domain::Username;
    use claims::assert_ok;
    use secrecy::SecretString;

    const OBSOLETE_PASSWORD_HASH: &str = "$argon2i$v=19$m=65536,t=1,p=1$c29tZXNhbHQAAAAAAAAAAA$+r0d29hqEB0yasKr55ZgICsQGSkl0v0kgwhd+U3wyRo";

    pub fn credentials(username: &str, password: &str) -> Credentials {
        Credentials {
            username: assert_ok!(Username::new(username.to_owned())),
            password: assert_ok!(Password::new(SecretString::from(password.to_owned()))),
        }
    }

    /// An obsolete Argon2i hash of `password`, used to exercise lazy upgrades.
    pub fn obsolete_password_hash() -> PasswordHash {
        assert_ok!(PasswordHash::try_from(OBSOLETE_PASSWORD_HASH.to_owned()))
    }
}
