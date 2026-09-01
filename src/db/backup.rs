//! Consistent database copies and the checks a restored copy must pass.
//!
//! `VACUUM INTO` reads the source inside a transaction, so the copy holds one
//! committed point in time including everything still in the write-ahead log. A
//! plain file copy can miss the log or catch a page mid-write, which is why the
//! runbook in `docs/backup-recovery.md` never uses one on a live database.

use crate::db::DbPool;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("backup destination '{0}' is not valid UTF-8")]
    NonUtf8Destination(PathBuf),
    #[error("failed to copy the database")]
    Copy(#[source] sqlx::Error),
    #[error("failed to reach the database")]
    Database(#[from] sqlx::Error),
    #[error("the database failed its integrity check: {report}")]
    Corrupt { report: String },
}

/// What preparing a restored database changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestoreReport {
    /// Sessions the backup carried, every one of which was deleted.
    pub revoked_sessions: u64,
}

/// Copies the database behind `pool` into `destination`.
///
/// SQLite refuses to write over an existing file, so an earlier backup can
/// never be replaced by accident.
///
/// # Errors
///
/// Returns [`BackupError::NonUtf8Destination`] when SQLite cannot name the
/// path, or [`BackupError::Copy`] when the destination already exists or cannot
/// be written.
#[tracing::instrument(
    name = "Back up database",
    skip(pool),
    fields(destination = %destination.display()),
    err,
)]
pub async fn back_up(pool: &DbPool, destination: &Path) -> Result<(), BackupError> {
    let destination = destination
        .to_str()
        .ok_or_else(|| BackupError::NonUtf8Destination(destination.to_owned()))?;

    sqlx::query!(
        r#"
        VACUUM INTO ?1
        "#,
        destination
    )
    .execute(pool)
    .await
    .map_err(BackupError::Copy)?;

    Ok(())
}

/// Checks a restored copy and revokes every session it carries.
///
/// A backup holds the sessions that were live when it was taken, so a restore
/// would otherwise hand Owner access back to whoever still holds one. The copy
/// counts as ready only once the integrity check passes and those rows are
/// gone.
///
/// # Errors
///
/// Returns [`BackupError::Corrupt`] when SQLite reports damage, or
/// [`BackupError::Database`] when a query fails.
#[tracing::instrument(name = "Prepare restored database", skip_all, err)]
pub async fn prepare_restored(pool: &DbPool) -> Result<RestoreReport, BackupError> {
    ensure_intact(pool).await?;

    let revoked = sqlx::query!(
        r#"
        DELETE FROM
            sessions
        "#
    )
    .execute(pool)
    .await?;

    Ok(RestoreReport {
        revoked_sessions: revoked.rows_affected(),
    })
}

/// Runs SQLite's own page-level check over the whole file. A healthy database
/// reports the single line `ok`; a damaged one reports what it found.
async fn ensure_intact(pool: &DbPool) -> Result<(), BackupError> {
    let report: Vec<String> = sqlx::query_scalar!(
        r#"
        PRAGMA integrity_check
        "#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .flatten()
    .collect();

    if report == ["ok"] {
        Ok(())
    } else {
        Err(BackupError::Corrupt {
            report: report.join("; "),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{connect_file, migrate, portfolio, seed_if_empty};
    use claims::{assert_err, assert_ok};
    use std::fs::File;
    use tempfile::{TempDir, tempdir};

    /// A seeded database file with one live session, standing in for the
    /// database an incident would be recovered from.
    async fn live_database(workspace: &TempDir) -> (PathBuf, DbPool) {
        let path = workspace.path().join("portfolio.db");
        File::create(&path).expect("create the database file");
        let pool = assert_ok!(connect_file(&path).await);
        assert_ok!(migrate(&pool).await);
        assert_ok!(seed_if_empty(&pool).await);
        sqlx::query!(
            r#"
            INSERT INTO
                sessions (id, data, expiry_date)
            VALUES
                ('live', '{}', 4102444800)
            "#
        )
        .execute(&pool)
        .await
        .expect("insert a live session");
        (path, pool)
    }

    #[tokio::test]
    async fn a_restored_backup_keeps_the_portfolio_and_drops_every_session() {
        let workspace = tempdir().expect("create a temporary workspace");
        let (_, live) = live_database(&workspace).await;
        let before = assert_ok!(portfolio::load(&live).await);

        let backup = workspace.path().join("portfolio-backup.db");
        assert_ok!(back_up(&live, &backup).await);

        // A restore puts this copy where the application expects its database,
        // so the checks run against the copy before anything is moved.
        let restored = assert_ok!(connect_file(&backup).await);
        let report = assert_ok!(prepare_restored(&restored).await);

        let after = assert_ok!(portfolio::load(&restored).await);
        assert_eq!(after.site.title, before.site.title);
        assert_eq!(
            after
                .projects
                .iter()
                .map(|project| &project.slug)
                .collect::<Vec<_>>(),
            before
                .projects
                .iter()
                .map(|project| &project.slug)
                .collect::<Vec<_>>()
        );

        assert_eq!(report.revoked_sessions, 1);
        let sessions = sqlx::query_scalar!(
            r#"
            SELECT
                COUNT(*)
            FROM
                sessions
            "#
        )
        .fetch_one(&restored)
        .await
        .expect("count the restored sessions");
        assert_eq!(sessions, 0);
    }

    #[tokio::test]
    async fn a_backup_never_writes_over_an_existing_file() {
        let workspace = tempdir().expect("create a temporary workspace");
        let (_, live) = live_database(&workspace).await;

        let backup = workspace.path().join("portfolio-backup.db");
        assert_ok!(back_up(&live, &backup).await);
        assert_err!(back_up(&live, &backup).await);
    }
}
