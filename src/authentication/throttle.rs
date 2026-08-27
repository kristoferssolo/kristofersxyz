use crate::domain::Username;
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const SOURCE_ATTEMPTS: u32 = 20;
const SOURCE_WINDOW: Duration = Duration::from_secs(60);
const ACCOUNT_GRACE_ATTEMPTS: u32 = 5;
const MAX_ACCOUNT_DELAY: Duration = Duration::from_secs(60);
const RETENTION: Duration = Duration::from_hours(1);
const MAX_TRACKED_SOURCES: usize = 4_096;
const MAX_TRACKED_ACCOUNTS: usize = 1_024;

/// The delay returned to an HTTP client after login throttling activates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetryAfter(u64);

impl RetryAfter {
    #[must_use]
    pub const fn seconds(self) -> u64 {
        self.0
    }

    fn from_duration(duration: Duration) -> Self {
        let rounded = duration
            .as_secs()
            .saturating_add(u64::from(duration.subsec_nanos() != 0));
        Self(rounded.max(1))
    }

    const fn one_minute() -> Self {
        Self(60)
    }
}

#[derive(Clone, Debug)]
pub struct LoginThrottle {
    state: Arc<Mutex<ThrottleState>>,
}

#[derive(Debug, Default)]
struct ThrottleState {
    sources: HashMap<IpAddr, SourceWindow>,
    accounts: HashMap<String, AccountFailures>,
}

#[derive(Clone, Copy, Debug)]
struct SourceWindow {
    started_at: Instant,
    attempts: u32,
}

#[derive(Clone, Copy, Debug)]
struct AccountFailures {
    last_failure: Instant,
    failures: u32,
    blocked_until: Instant,
}

impl Default for LoginThrottle {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(ThrottleState::default())),
        }
    }
}

impl LoginThrottle {
    /// Reserves one attempt in the source-address window.
    ///
    /// Forwarded headers never enter this interface. Callers must use the
    /// transport peer address supplied by Axum.
    ///
    /// # Errors
    ///
    /// Returns the required delay when the source has exhausted its burst.
    pub fn check_source(&self, address: IpAddr) -> Result<(), RetryAfter> {
        self.check_source_at(address, Instant::now())
    }

    /// Rejects a username while its failure cooldown remains active.
    ///
    /// # Errors
    ///
    /// Returns the remaining delay while the account is cooling down.
    pub fn check_account(&self, username: &Username) -> Result<(), RetryAfter> {
        self.check_account_at(username, Instant::now())
    }

    /// Adds a failed credential check and starts its increasing cooldown.
    pub fn record_failure(&self, username: &Username) {
        self.record_failure_at(username, Instant::now());
    }

    /// A successful authentication clears the username penalty.
    pub fn record_success(&self, username: &Username) {
        if let Ok(mut state) = self.state.lock() {
            state.accounts.remove(username.as_str());
        }
    }

    fn check_source_at(&self, address: IpAddr, now: Instant) -> Result<(), RetryAfter> {
        let Ok(mut state) = self.state.lock() else {
            return Err(RetryAfter::one_minute());
        };
        state.purge(now);

        if !state.sources.contains_key(&address) && state.sources.len() >= MAX_TRACKED_SOURCES {
            return Err(RetryAfter::one_minute());
        }

        let window = state.sources.entry(address).or_insert(SourceWindow {
            started_at: now,
            attempts: 0,
        });
        let elapsed = now.saturating_duration_since(window.started_at);
        if elapsed >= SOURCE_WINDOW {
            *window = SourceWindow {
                started_at: now,
                attempts: 0,
            };
        } else if window.attempts >= SOURCE_ATTEMPTS {
            return Err(RetryAfter::from_duration(
                SOURCE_WINDOW.saturating_sub(elapsed),
            ));
        }

        window.attempts = window.attempts.saturating_add(1);
        Ok(())
    }

    fn check_account_at(&self, username: &Username, now: Instant) -> Result<(), RetryAfter> {
        let Ok(mut state) = self.state.lock() else {
            return Err(RetryAfter::one_minute());
        };
        state.purge(now);

        state
            .accounts
            .get(username.as_str())
            .map_or(Ok(()), |entry| {
                if now < entry.blocked_until {
                    Err(RetryAfter::from_duration(
                        entry.blocked_until.saturating_duration_since(now),
                    ))
                } else {
                    Ok(())
                }
            })
    }

    fn record_failure_at(&self, username: &Username, now: Instant) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        state.purge(now);

        if !state.accounts.contains_key(username.as_str())
            && state.accounts.len() >= MAX_TRACKED_ACCOUNTS
        {
            return;
        }

        let failures = state
            .accounts
            .get(username.as_str())
            .map_or(1, |entry| entry.failures.saturating_add(1));
        let delay = account_delay(failures);
        state.accounts.insert(
            username.as_str().to_owned(),
            AccountFailures {
                last_failure: now,
                failures,
                blocked_until: now.checked_add(delay).unwrap_or(now),
            },
        );
    }
}

impl ThrottleState {
    fn purge(&mut self, now: Instant) {
        self.sources
            .retain(|_, entry| now.saturating_duration_since(entry.started_at) < RETENTION);
        self.accounts
            .retain(|_, entry| now.saturating_duration_since(entry.last_failure) < RETENTION);
    }
}

fn account_delay(failures: u32) -> Duration {
    let exponent = failures.saturating_sub(ACCOUNT_GRACE_ATTEMPTS).min(7);
    if exponent == 0 {
        return Duration::ZERO;
    }

    let seconds = 1_u64.checked_shl(exponent.saturating_sub(1)).unwrap_or(60);
    Duration::from_secs(seconds).min(MAX_ACCOUNT_DELAY)
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};
    use std::net::{IpAddr, Ipv4Addr};

    fn username() -> Username {
        Username::new("owner".to_owned()).expect("the fixture username is valid")
    }

    #[test]
    fn one_source_gets_a_bounded_burst() {
        let throttle = LoginThrottle::default();
        let now = Instant::now();
        let address = IpAddr::V4(Ipv4Addr::LOCALHOST);

        for _ in 0..SOURCE_ATTEMPTS {
            assert_ok!(throttle.check_source_at(address, now));
        }
        assert_eq!(
            assert_err!(throttle.check_source_at(address, now)),
            RetryAfter(SOURCE_WINDOW.as_secs())
        );
        assert_ok!(throttle.check_source_at(address, now + SOURCE_WINDOW));
    }

    #[test]
    fn account_failures_add_a_capped_increasing_delay() {
        let throttle = LoginThrottle::default();
        let username = username();
        let now = Instant::now();

        for _ in 0..=ACCOUNT_GRACE_ATTEMPTS {
            throttle.record_failure_at(&username, now);
        }
        assert_eq!(
            assert_err!(throttle.check_account_at(&username, now)),
            RetryAfter(1)
        );

        for _ in 0..10 {
            throttle.record_failure_at(&username, now);
        }
        assert_eq!(
            assert_err!(throttle.check_account_at(&username, now)),
            RetryAfter(MAX_ACCOUNT_DELAY.as_secs())
        );
    }

    #[test]
    fn success_clears_the_account_penalty() {
        let throttle = LoginThrottle::default();
        let username = username();
        let now = Instant::now();

        for _ in 0..=ACCOUNT_GRACE_ATTEMPTS {
            throttle.record_failure_at(&username, now);
        }
        assert_err!(throttle.check_account_at(&username, now));
        throttle.record_success(&username);
        assert_ok!(throttle.check_account_at(&username, now));
    }
}
