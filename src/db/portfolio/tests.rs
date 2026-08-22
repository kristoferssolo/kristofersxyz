use super::*;
use crate::db::test_support::seeded_pool;

#[tokio::test]
async fn the_seed_loads_into_the_content_model() {
    let content = load(&seeded_pool().await)
        .await
        .expect("load the seeded portfolio");

    assert_eq!(
        content.site.title,
        "Kristofers Solo, Rust software developer"
    );
    assert_eq!(content.profile.name, "Kristofers Solo");
    assert_eq!(
        content.profile.technologies,
        ["Rust", "Leptos", "Axum", "Tailwind"]
    );
    assert_eq!(content.profile.working_style.len(), 4);
    assert_eq!(content.profile.links.len(), 4);
    assert_eq!(content.contact.name, "Write to me");
}

#[tokio::test]
async fn projects_keep_their_order_technologies_and_links() {
    let content = load(&seeded_pool().await)
        .await
        .expect("load the seeded portfolio");

    let names = content
        .projects
        .iter()
        .map(|project| project.slug.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, ["guenther", "traxor", "cipher-workshop"]);

    let cipher = &content.projects[2];
    assert_eq!(
        cipher.technologies,
        ["Rust", "AES-128", "CLI", "WebAssembly"]
    );
    assert_eq!(cipher.links.len(), 1);
    assert_eq!(cipher.links[0].label, "GitHub");
    assert_eq!(
        cipher.links[0].href,
        "https://github.com/kristoferssolo/cipher-workshop"
    );
}

/// The seed exists to reproduce the static fixture exactly. If they drift,
/// the page renders different content depending on where it was loaded.
#[tokio::test]
async fn the_seed_matches_the_static_fixture() {
    use crate::app::content::portfolio_content;

    let loaded = load(&seeded_pool().await)
        .await
        .expect("load the seeded portfolio");
    let fixture = portfolio_content();

    assert_eq!(loaded.profile.about, fixture.profile.about);
    assert_eq!(loaded.projects.len(), fixture.projects.len());
    for (loaded, fixture) in loaded.projects.iter().zip(&fixture.projects) {
        assert_eq!(loaded.slug, fixture.slug);
        assert_eq!(loaded.title, fixture.title);
        assert_eq!(loaded.summary, fixture.summary);
        assert_eq!(loaded.technologies, fixture.technologies);
    }
}
