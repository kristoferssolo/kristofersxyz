//! Owner authentication: credential verification and password hashing.
//!
//! Modelled on *Zero to Production in Rust*. Usernames and Argon2 PHC hashes
//! live in the `users` table, and a login is one verification against the
//! stored hash. The verification runs even for an unknown username, so a
//! failed login takes the same time whether or not the username exists and
//! cannot be used to enumerate accounts.

mod password;

pub use password::{AuthError, Credentials, compute_password_hash, validate_credentials};
