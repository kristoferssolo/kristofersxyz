//! Loads the portfolio from SQLite into the content model.
//!
//! The database is the source of truth. This module owns the queries and the
//! row shapes; it maps them into [`PortfolioContent`], which stays free of any
//! persistence concern so it can also compile for the WASM client.

mod projects;
mod rows;
#[cfg(test)]
mod tests;

use self::rows::{ContactRow, FocusRow, ProfileRow, SiteRow, SocialRow};
use crate::{
    app::content::{Contact, FocusArea, PortfolioContent, Profile, Site, SocialLink},
    db::DbPool,
};

/// Reads the whole portfolio in a handful of ordered queries and assembles it
/// into [`PortfolioContent`]. Rows come back by `sort_order`, so the buffer
/// lines keep the order the author gave them.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if a query fails or a singleton row is missing.
pub async fn load(pool: &DbPool) -> Result<PortfolioContent, sqlx::Error> {
    let site = sqlx::query_as::<_, SiteRow>(
        "SELECT url, title, description, og_image FROM site WHERE id = 1",
    )
    .fetch_one(pool)
    .await?;

    let profile = sqlx::query_as::<_, ProfileRow>(
        "SELECT name, title, summary, about, email FROM profile WHERE id = 1",
    )
    .fetch_one(pool)
    .await?;

    let stack =
        sqlx::query_scalar::<_, String>("SELECT item FROM profile_stack ORDER BY sort_order")
            .fetch_all(pool)
            .await?;

    let working_style = sqlx::query_as::<_, FocusRow>(
        "SELECT label, detail FROM working_principle ORDER BY sort_order",
    )
    .fetch_all(pool)
    .await?;

    let links = sqlx::query_as::<_, SocialRow>(
        "SELECT label, href, rel FROM social_link ORDER BY sort_order",
    )
    .fetch_all(pool)
    .await?;

    let contact = sqlx::query_as::<_, ContactRow>("SELECT name, body FROM contact WHERE id = 1")
        .fetch_one(pool)
        .await?;

    Ok(PortfolioContent {
        site: Site {
            url: site.url,
            title: site.title,
            description: site.description,
            og_image: site.og_image,
        },
        profile: Profile {
            name: profile.name,
            title: profile.title,
            summary: profile.summary,
            about: profile.about,
            stack,
            working_style: working_style
                .into_iter()
                .map(|row| FocusArea {
                    label: row.label,
                    detail: row.detail,
                })
                .collect(),
            email: profile.email,
            links: links
                .into_iter()
                .map(|row| SocialLink {
                    label: row.label,
                    href: row.href,
                    rel: row.rel,
                })
                .collect(),
        },
        projects: projects::load(pool).await?,
        contact: Contact {
            name: contact.name,
            body: contact.body,
        },
    })
}
