//! The admin session: the keys it stores for the signed-in owner, and the
//! reads and writes over them.

use tower_sessions::{Session, session};
use uuid::Uuid;

/// Session key under which the authenticated user's id lives.
const USER_ID_KEY: &str = "user_id";
/// Session key under which the authenticated user's name lives, shown on the
/// admin panel.
const USERNAME_KEY: &str = "username";

/// The authenticated user's id, if the session holds one.
pub(super) async fn owner(session: &Session) -> Option<Uuid> {
    session.get::<Uuid>(USER_ID_KEY).await.ok().flatten()
}

/// The authenticated user's name, if the session holds one.
pub(super) async fn username(session: &Session) -> Option<String> {
    session.get::<String>(USERNAME_KEY).await.ok().flatten()
}

/// Rotates the session id to defeat fixation, then records the user's id and
/// name.
pub(super) async fn establish_session(
    session: &Session,
    user_id: Uuid,
    username: &str,
) -> Result<(), session::Error> {
    session.cycle_id().await?;
    session.insert(USER_ID_KEY, user_id).await?;
    session.insert(USERNAME_KEY, username).await
}
