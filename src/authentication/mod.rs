//! Owner authentication: credential verification and password hashing.
//!
//! Modelled on *Zero to Production in Rust*. The `users` table stores usernames
//! and Argon2 PHC hashes. Unknown usernames still run one hash verification to
//! reduce timing differences that could reveal registered accounts.

mod password;
mod session;

pub use password::{AuthError, Credentials, compute_password_hash, validate_credentials};
pub(crate) use session::{AuthSession, Authenticated, SessionState, Unverified};
