//! Secret-safe structured events for authentication and Owner activity.

use crate::{
    authentication::RetryAfter,
    domain::{OwnerId, Username},
};

const TARGET: &str = "kristofersxyz::security";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthenticationFailure {
    InvalidInput,
    InvalidCredentials,
    Internal,
}

impl AuthenticationFailure {
    const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::InvalidCredentials => "invalid_credentials",
            Self::Internal => "internal_error",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoginThrottleScope {
    Source,
    Account,
    PasswordCapacity,
}

impl LoginThrottleScope {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Account => "account",
            Self::PasswordCapacity => "password_capacity",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionRejection {
    Unrecognized,
    PartialIdentity,
    Corrupt,
    IdleExpired,
    AbsoluteExpired,
}

impl SessionRejection {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Unrecognized => "unrecognized",
            Self::PartialIdentity => "partial_identity",
            Self::Corrupt => "corrupt",
            Self::IdleExpired => "idle_expired",
            Self::AbsoluteExpired => "absolute_expired",
        }
    }
}

/// Why a request was not addressed to the configured public authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostRejection {
    Missing,
    Malformed,
    Conflicting,
    Unexpected,
}

impl HostRejection {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Malformed => "malformed",
            Self::Conflicting => "conflicting",
            Self::Unexpected => "unexpected",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestKind {
    Login,
    ServerFunction,
    ScreenshotUpload,
}

impl RequestKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::ServerFunction => "server_function",
            Self::ScreenshotUpload => "screenshot_upload",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortfolioResource {
    Project,
    ProjectScreenshot,
    Profile,
    Contact,
    SiteMetadata,
}

impl PortfolioResource {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::ProjectScreenshot => "project_screenshot",
            Self::Profile => "profile",
            Self::Contact => "contact",
            Self::SiteMetadata => "site_metadata",
        }
    }
}

/// A security or audit event whose fields cannot contain credentials, hashes,
/// cookies, session identifiers, request bodies, or raw database errors.
pub enum SecurityEvent<'a> {
    AuthenticationFailed {
        username: Option<&'a Username>,
        reason: AuthenticationFailure,
    },
    AuthenticationSucceeded {
        owner_id: OwnerId,
        username: &'a Username,
    },
    LoginThrottled {
        username: Option<&'a Username>,
        scope: LoginThrottleScope,
        retry_after: RetryAfter,
    },
    AuthorizationRejected,
    CsrfRejected,
    HostRejected {
        reason: HostRejection,
    },
    RequestBodyRejected {
        kind: RequestKind,
        limit_bytes: usize,
    },
    SessionRejected {
        reason: SessionRejection,
    },
    SessionStarted {
        owner_id: OwnerId,
        username: &'a Username,
        revoked_sessions: u64,
    },
    SessionStartFailed {
        owner_id: OwnerId,
        username: &'a Username,
    },
    SessionEnded {
        owner_id: OwnerId,
        username: &'a Username,
    },
    SessionEndFailed {
        owner_id: OwnerId,
        username: &'a Username,
    },
    SessionCleanupFailed,
    PasswordChanged {
        username: &'a Username,
        revoked_sessions: u64,
    },
    DatabaseRestored {
        revoked_sessions: u64,
    },
    PortfolioChanged {
        owner_id: OwnerId,
        resource: PortfolioResource,
    },
}

impl SecurityEvent<'_> {
    pub fn record(self) {
        match self {
            Self::AuthenticationFailed { username, reason } => {
                record_authentication_failure(username, reason);
            }
            Self::AuthenticationSucceeded { owner_id, username } => tracing::info!(
                target: TARGET,
                security_event = "authentication_succeeded",
                outcome = "success",
                owner_id = %owner_id,
                username = %username,
                "Owner authentication succeeded"
            ),
            Self::LoginThrottled {
                username,
                scope,
                retry_after,
            } => record_login_throttle(username, scope, retry_after),
            Self::AuthorizationRejected => tracing::warn!(
                target: TARGET,
                security_event = "authorization_rejected",
                outcome = "denied",
                "Rejected unauthenticated Owner operation"
            ),
            Self::CsrfRejected => tracing::warn!(
                target: TARGET,
                security_event = "csrf_rejected",
                outcome = "denied",
                "Rejected cross-origin request"
            ),
            Self::HostRejected { reason } => record_host_rejection(reason),
            Self::RequestBodyRejected { kind, limit_bytes } => tracing::warn!(
                target: TARGET,
                security_event = "request_body_rejected",
                outcome = "denied",
                request_kind = kind.as_str(),
                limit_bytes,
                "Rejected oversized request body"
            ),
            Self::SessionRejected { reason } => tracing::warn!(
                target: TARGET,
                security_event = "session_rejected",
                outcome = "denied",
                reason = reason.as_str(),
                "Rejected invalid Owner session"
            ),
            Self::SessionStarted {
                owner_id,
                username,
                revoked_sessions,
            } => tracing::info!(
                target: TARGET,
                security_event = "session_started",
                outcome = "success",
                owner_id = %owner_id,
                username = %username,
                session_rotated = true,
                revoked_sessions,
                "Started Owner session"
            ),
            Self::SessionStartFailed { owner_id, username } => tracing::error!(
                target: TARGET,
                security_event = "session_start_failed",
                outcome = "failure",
                owner_id = %owner_id,
                username = %username,
                "Failed to start Owner session"
            ),
            Self::SessionEnded { owner_id, username } => tracing::info!(
                target: TARGET,
                security_event = "session_ended",
                outcome = "success",
                owner_id = %owner_id,
                username = %username,
                "Ended Owner session"
            ),
            Self::SessionEndFailed { owner_id, username } => tracing::error!(
                target: TARGET,
                security_event = "session_end_failed",
                outcome = "failure",
                owner_id = %owner_id,
                username = %username,
                "Failed to end Owner session"
            ),
            Self::SessionCleanupFailed => tracing::error!(
                target: TARGET,
                security_event = "session_cleanup_failed",
                outcome = "failure",
                "Failed to clear incomplete Owner session"
            ),
            Self::PasswordChanged {
                username,
                revoked_sessions,
            } => record_password_change(username, revoked_sessions),
            Self::DatabaseRestored { revoked_sessions } => {
                record_database_restore(revoked_sessions);
            }
            Self::PortfolioChanged { owner_id, resource } => {
                record_portfolio_change(owner_id, resource);
            }
        }
    }
}

fn record_host_rejection(reason: HostRejection) {
    tracing::warn!(
        target: TARGET,
        security_event = "host_rejected",
        outcome = "denied",
        reason = reason.as_str(),
        "Rejected request addressed to an unexpected host"
    );
}

fn record_password_change(username: &Username, revoked_sessions: u64) {
    tracing::warn!(
        target: TARGET,
        security_event = "owner_password_changed",
        outcome = "success",
        username = %username,
        revoked_sessions,
        "Changed Owner password and revoked sessions"
    );
}

fn record_database_restore(revoked_sessions: u64) {
    tracing::warn!(
        target: TARGET,
        security_event = "database_restored",
        outcome = "success",
        revoked_sessions,
        "Prepared a restored database and revoked its sessions"
    );
}

fn record_portfolio_change(owner_id: OwnerId, resource: PortfolioResource) {
    tracing::info!(
        target: TARGET,
        security_event = "portfolio_changed",
        outcome = "success",
        owner_id = %owner_id,
        resource = resource.as_str(),
        "Changed portfolio content"
    );
}

fn record_authentication_failure(username: Option<&Username>, reason: AuthenticationFailure) {
    if let Some(username) = username {
        tracing::warn!(
            target: TARGET,
            security_event = "authentication_failed",
            outcome = "failure",
            reason = reason.as_str(),
            username = %username,
            "Owner authentication failed"
        );
    } else {
        tracing::warn!(
            target: TARGET,
            security_event = "authentication_failed",
            outcome = "failure",
            reason = reason.as_str(),
            "Owner authentication failed"
        );
    }
}

fn record_login_throttle(
    username: Option<&Username>,
    scope: LoginThrottleScope,
    retry_after: RetryAfter,
) {
    if let Some(username) = username {
        tracing::warn!(
            target: TARGET,
            security_event = "login_throttled",
            outcome = "denied",
            scope = scope.as_str(),
            username = %username,
            retry_after_seconds = retry_after.seconds(),
            "Throttled Owner login"
        );
    } else {
        tracing::warn!(
            target: TARGET,
            security_event = "login_throttled",
            outcome = "denied",
            scope = scope.as_str(),
            retry_after_seconds = retry_after.seconds(),
            "Throttled Owner login"
        );
    }
}
