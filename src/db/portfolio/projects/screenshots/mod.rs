//! Project Screenshot persistence.
//!
//! Image bytes and metadata share a SQLite transaction. Stored bytes are immutable;
//! metadata edits keep the screenshot identity, while replacement creates a new one.

mod image;
mod order;
mod write;

pub use image::{StoredScreenshot, image};
pub use order::move_within_project;
pub use write::{append, remove, set_details};

#[cfg(test)]
use order::stepped;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::portfolio::projects::ScreenshotError;
    use crate::{
        db::{DbPool, portfolio::load, seed_if_empty, test_support::migrated_pool},
        domain::{
            Project, ProjectScreenshot, ProjectSlug, ScreenshotAltText, ScreenshotCaption,
            ScreenshotId, ScreenshotMediaType, ScreenshotMove, ScreenshotSize,
        },
    };
    use claims::{assert_matches, assert_none, assert_ok, assert_some, assert_some_eq};
    use rstest::rstest;

    /// A one-pixel PNG, small enough to keep the fixtures readable and real
    /// enough to survive a round trip through SQLite.
    const PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89,
    ];
    const WEBP: &[u8] = &[0x52, 0x49, 0x46, 0x46, 0x00, 0x57, 0x45, 0x42, 0x50];

    async fn seeded_pool() -> DbPool {
        let pool = migrated_pool().await;
        seed_if_empty(&pool).await.expect("seed the database");
        pool
    }

    fn slug(value: &str) -> ProjectSlug {
        crate::test_support::parse(value)
    }

    fn size() -> ScreenshotSize {
        assert_ok!(ScreenshotSize::try_from((1600, 1000)))
    }

    async fn store(pool: &DbPool, project: &str, alt: &str, bytes: &[u8]) -> ScreenshotId {
        assert_ok!(
            append(
                pool,
                &slug(project),
                ScreenshotMediaType::Png,
                bytes,
                size(),
                &crate::test_support::parse::<ScreenshotAltText>(alt),
                None,
            )
            .await
        )
    }

    async fn stored(pool: &DbPool, project: &str) -> Vec<ProjectScreenshot> {
        assert_ok!(load(pool).await)
            .projects
            .into_iter()
            .find(|candidate: &Project| candidate.slug.as_str() == project)
            .map(|found| found.screenshots.as_slice().to_vec())
            .unwrap_or_default()
    }

    async fn alt_texts(pool: &DbPool, project: &str) -> Vec<String> {
        stored(pool, project)
            .await
            .iter()
            .map(|shot| shot.alt.to_string())
            .collect()
    }

    async fn sort_orders(pool: &DbPool) -> Vec<i64> {
        sqlx::query_scalar!(
            r#"
            SELECT
                sort_order
            FROM
                project_screenshot
            ORDER BY
                sort_order
            "#
        )
        .fetch_all(pool)
        .await
        .expect("read the stored order")
    }

    #[tokio::test]
    async fn a_screenshot_lands_after_the_project_s_final_one() {
        let pool = seeded_pool().await;

        store(&pool, "traxor", "First", PNG).await;
        store(&pool, "traxor", "Second", PNG).await;

        assert_eq!(alt_texts(&pool, "traxor").await, ["First", "Second"]);
        assert_eq!(sort_orders(&pool).await, [1, 2]);
    }

    #[tokio::test]
    async fn a_screenshot_belongs_to_one_project_only() {
        let pool = seeded_pool().await;

        store(&pool, "traxor", "Queue", PNG).await;
        store(&pool, "guenther", "Chat", WEBP).await;

        assert_eq!(alt_texts(&pool, "traxor").await, ["Queue"]);
        assert_eq!(alt_texts(&pool, "guenther").await, ["Chat"]);
        assert!(stored(&pool, "cipher-workshop").await.is_empty());
    }

    #[tokio::test]
    async fn an_unknown_project_stores_nothing() {
        let pool = seeded_pool().await;

        let rejected = append(
            &pool,
            &slug("ghost"),
            ScreenshotMediaType::Png,
            PNG,
            size(),
            &crate::test_support::parse::<ScreenshotAltText>("Nothing"),
            None,
        )
        .await;

        assert_matches!(rejected, Err(ScreenshotError::UnknownProject));
        let orders: [i64; 0] = [];
        assert_eq!(sort_orders(&pool).await, orders);
    }

    #[tokio::test]
    async fn editing_the_details_keeps_the_identity_and_the_bytes() {
        let pool = seeded_pool().await;
        let id = store(&pool, "traxor", "Before", PNG).await;

        assert_ok!(
            set_details(
                &pool,
                &id,
                &crate::test_support::parse::<ScreenshotAltText>("After"),
                Some(&crate::test_support::parse::<ScreenshotCaption>(
                    "A caption"
                )),
            )
            .await
        );

        let screenshots = stored(&pool, "traxor").await;
        let screenshot = assert_some!(screenshots.first());
        assert_eq!(screenshot.id, id);
        assert_eq!(screenshot.alt.to_string(), "After");
        assert_some_eq!(
            screenshot.caption.as_ref().map(ToString::to_string),
            "A caption".to_owned()
        );
        assert_eq!(assert_some!(assert_ok!(image(&pool, &id).await)).bytes, PNG);
    }

    #[tokio::test]
    async fn a_caption_can_be_cleared() {
        let pool = seeded_pool().await;
        let id = store(&pool, "traxor", "Queue", PNG).await;
        let alt = crate::test_support::parse::<ScreenshotAltText>("Queue");
        assert_ok!(
            set_details(
                &pool,
                &id,
                &alt,
                Some(&crate::test_support::parse::<ScreenshotCaption>("Present")),
            )
            .await
        );

        assert_ok!(set_details(&pool, &id, &alt, None).await);

        let screenshots = stored(&pool, "traxor").await;
        assert_none!(assert_some!(screenshots.first()).caption.as_ref());
    }

    /// Nothing rewrites the bytes of a stored screenshot, so a different image
    /// is a different identity and no cached response can go stale.
    #[tokio::test]
    async fn replacing_the_image_takes_a_new_identity() {
        let pool = seeded_pool().await;
        let first = store(&pool, "traxor", "Queue", PNG).await;

        assert_ok!(remove(&pool, &first).await);
        let second = store(&pool, "traxor", "Queue", WEBP).await;

        assert_ne!(first, second);
        assert_none!(assert_ok!(image(&pool, &first).await));
        assert_eq!(
            assert_some!(assert_ok!(image(&pool, &second).await)).bytes,
            WEBP
        );
    }

    #[rstest]
    #[case(ScreenshotMove::Up, 1, ["Second", "First", "Third"])]
    #[case(ScreenshotMove::Down, 0, ["Second", "First", "Third"])]
    #[case(ScreenshotMove::Down, 1, ["First", "Third", "Second"])]
    #[tokio::test]
    async fn a_step_swaps_with_its_neighbour(
        #[case] movement: ScreenshotMove,
        #[case] moved: usize,
        #[case] expected: [&str; 3],
    ) {
        let pool = seeded_pool().await;
        for alt in ["First", "Second", "Third"] {
            store(&pool, "traxor", alt, PNG).await;
        }
        let screenshots = stored(&pool, "traxor").await;
        let id = assert_some!(screenshots.get(moved)).id.clone();

        assert_ok!(move_within_project(&pool, &id, movement).await);

        assert_eq!(alt_texts(&pool, "traxor").await, expected);
        assert_eq!(sort_orders(&pool).await, [1, 2, 3]);
    }

    #[tokio::test]
    async fn a_move_off_either_end_leaves_the_order_alone() {
        let pool = seeded_pool().await;
        let first = store(&pool, "traxor", "First", PNG).await;
        let last = store(&pool, "traxor", "Second", PNG).await;

        assert_matches!(
            move_within_project(&pool, &first, ScreenshotMove::Up).await,
            Err(ScreenshotError::InvalidMovement)
        );
        assert_matches!(
            move_within_project(&pool, &last, ScreenshotMove::Down).await,
            Err(ScreenshotError::InvalidMovement)
        );

        assert_eq!(alt_texts(&pool, "traxor").await, ["First", "Second"]);
        assert_eq!(sort_orders(&pool).await, [1, 2]);
    }

    /// A move only reaches the Project it belongs to, so a screenshot cannot
    /// step past the end of its own order into another Project's.
    #[tokio::test]
    async fn a_move_stays_inside_its_project() {
        let pool = seeded_pool().await;
        let only = store(&pool, "traxor", "Queue", PNG).await;
        store(&pool, "guenther", "Chat", PNG).await;

        assert_matches!(
            move_within_project(&pool, &only, ScreenshotMove::Down).await,
            Err(ScreenshotError::InvalidMovement)
        );
    }

    #[tokio::test]
    async fn deleting_removes_the_screenshot_and_closes_the_gap() {
        let pool = seeded_pool().await;
        let first = store(&pool, "traxor", "First", PNG).await;
        store(&pool, "traxor", "Second", PNG).await;
        store(&pool, "traxor", "Third", PNG).await;

        assert_ok!(remove(&pool, &first).await);

        assert_eq!(alt_texts(&pool, "traxor").await, ["Second", "Third"]);
        assert_eq!(sort_orders(&pool).await, [1, 2]);
        assert_none!(assert_ok!(image(&pool, &first).await));
    }

    #[tokio::test]
    async fn an_unknown_screenshot_is_rejected_by_every_write() {
        let pool = seeded_pool().await;
        let missing =
            crate::test_support::parse::<ScreenshotId>("9c1f4e2a-1f2b-4a3c-8d4e-5f6a7b8c9d0e");

        assert_matches!(
            remove(&pool, &missing).await,
            Err(ScreenshotError::UnknownScreenshot)
        );
        assert_matches!(
            move_within_project(&pool, &missing, ScreenshotMove::Up).await,
            Err(ScreenshotError::UnknownScreenshot)
        );
        assert_matches!(
            set_details(
                &pool,
                &missing,
                &crate::test_support::parse::<ScreenshotAltText>("Nothing"),
                None,
            )
            .await,
            Err(ScreenshotError::UnknownScreenshot)
        );
        assert_none!(assert_ok!(image(&pool, &missing).await));
    }

    /// The cascade is the database's own, so a Project cannot leave orphaned
    /// image bytes behind however it is deleted.
    #[tokio::test]
    async fn deleting_a_project_deletes_its_screenshots() {
        let pool = seeded_pool().await;
        store(&pool, "traxor", "Queue", PNG).await;
        store(&pool, "guenther", "Chat", PNG).await;

        sqlx::query!(
            r#"
            DELETE FROM
                project
            WHERE
                slug = 'traxor'
            "#
        )
        .execute(&pool)
        .await
        .expect("delete the project");

        let remaining = sqlx::query_scalar!(
            r#"
            SELECT
                alt_text
            FROM
                project_screenshot
            "#
        )
        .fetch_all(&pool)
        .await
        .expect("read the remaining screenshots");
        assert_eq!(remaining, ["Chat"]);
    }

    #[tokio::test]
    async fn stored_bytes_and_media_type_come_back_unchanged() {
        let pool = seeded_pool().await;
        let id = assert_ok!(
            append(
                &pool,
                &slug("traxor"),
                ScreenshotMediaType::Webp,
                WEBP,
                size(),
                &crate::test_support::parse::<ScreenshotAltText>("Queue"),
                None,
            )
            .await
        );

        let found = assert_some!(assert_ok!(image(&pool, &id).await));

        assert_eq!(found.media_type, ScreenshotMediaType::Webp);
        assert_eq!(found.bytes, WEBP);
    }

    /// Loading the portfolio reads metadata only. A blob in that query would
    /// put every stored image in memory on every page render.
    #[tokio::test]
    async fn loading_a_project_reads_metadata_without_the_bytes() {
        let pool = seeded_pool().await;
        store(&pool, "traxor", "Queue", PNG).await;

        let screenshots = stored(&pool, "traxor").await;

        let screenshot = assert_some!(screenshots.first());
        assert_eq!(screenshot.media_type, ScreenshotMediaType::Png);
        assert_eq!(
            (screenshot.size.width(), screenshot.size.height()),
            (1600, 1000)
        );
        assert_eq!(
            screenshot.media_path(),
            format!("/media/project/{}", screenshot.id)
        );
    }

    #[rstest]
    #[case(0, ScreenshotMove::Down, vec![1, 0, 2])]
    #[case(2, ScreenshotMove::Up, vec![0, 2, 1])]
    fn a_step_reorders_positions(
        #[case] from: usize,
        #[case] movement: ScreenshotMove,
        #[case] expected: Vec<usize>,
    ) {
        assert_eq!(assert_ok!(stepped(3, from, movement)), expected);
    }

    #[rstest]
    #[case(0, ScreenshotMove::Up)]
    #[case(2, ScreenshotMove::Down)]
    #[case(0, ScreenshotMove::Down)]
    fn a_step_past_either_end_is_rejected(#[case] from: usize, #[case] movement: ScreenshotMove) {
        let total = if from == 0 && movement == ScreenshotMove::Down {
            1
        } else {
            3
        };
        assert_matches!(
            stepped(total, from, movement),
            Err(ScreenshotError::InvalidMovement)
        );
    }
}
