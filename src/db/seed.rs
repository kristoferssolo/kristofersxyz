use super::DbPool;

/// Loads the bundled seed into an empty database.
///
/// The seed is embedded at compile time so no file has to ship. Paired with the
/// volumeless deploy, this makes a fresh boot self-sufficient without a manual
/// seed step.
///
/// The guard keeps this safe once a persistent volume and the CMS arrive: when
/// content already exists it does nothing, so edits are never clobbered.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the emptiness check or the seed fails.
pub async fn seed_if_empty(pool: &DbPool) -> Result<(), sqlx::Error> {
    let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM site")
        .fetch_one(pool)
        .await?;
    if existing > 0 {
        return Ok(());
    }

    sqlx::raw_sql(include_str!("../../seeds/portfolio.sql"))
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_support::migrated_pool;

    #[tokio::test]
    async fn seed_if_empty_populates_a_fresh_database() {
        let pool = migrated_pool().await;
        seed_if_empty(&pool).await.expect("seed the empty database");

        let projects: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM project")
            .fetch_one(&pool)
            .await
            .expect("count projects");
        assert_eq!(projects, 3);
    }

    /// The CMS-safety property: re-running never overwrites existing content.
    #[tokio::test]
    async fn seed_if_empty_leaves_existing_content_alone() {
        let pool = migrated_pool().await;
        seed_if_empty(&pool).await.expect("first seed");

        sqlx::query("UPDATE site SET title = 'edited' WHERE id = 1")
            .execute(&pool)
            .await
            .expect("edit the seeded content");
        seed_if_empty(&pool).await.expect("second seed is a no-op");

        let title: String = sqlx::query_scalar("SELECT title FROM site WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("read the title back");
        assert_eq!(title, "edited");
    }
}
