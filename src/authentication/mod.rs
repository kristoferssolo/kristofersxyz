//! Owner authentication: credential verification and password hashing.
//!
//! Modelled on *Zero to Production in Rust*. The `users` table stores usernames
//! and Argon2 PHC hashes. Unknown usernames still run one hash verification to
//! reduce timing differences that could reveal registered accounts.

mod credentials;
mod password;
mod session;
mod throttle;

pub use crate::domain::OwnerId;
pub use credentials::{AuthError, Credentials, validate_credentials};
pub use password::{Password, PasswordError, PasswordHash, compute_password_hash};
pub(crate) use session::{AuthSession, Authenticated, SessionState, Unverified};
pub use throttle::{LoginThrottle, RetryAfter};
