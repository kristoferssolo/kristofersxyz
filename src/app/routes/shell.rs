use super::App;
use crate::app::content::PortfolioContent;
use leptos::{prelude::*, serde_json};
use leptos_meta::MetaTags;

// `use<>` keeps the returned view from capturing the `content` borrow: it is
// serialized into an owned string here and never referenced afterwards.
#[must_use]
pub fn shell(options: LeptosOptions, content: &PortfolioContent) -> impl IntoView + use<> {
    // The content the client reads back into context before hydrating, so both
    // sides render from identical values. It lives in the head, which
    // hydrate_body does not walk, so it never reads as a hydration mismatch.
    let content = serde_json::to_string(content).unwrap_or_default();

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
                <script id="portfolio-content" type="application/json" inner_html=content></script>
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}
