use super::App;
use crate::app::content::PortfolioContent;
use leptos::{prelude::*, serde_json};
use leptos_meta::MetaTags;

/// Builds the server-rendered document and embeds the portfolio used during
/// hydration. The returned view does not borrow `content`.
#[must_use]
pub fn shell(options: LeptosOptions, content: &PortfolioContent) -> impl IntoView + use<> {
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
