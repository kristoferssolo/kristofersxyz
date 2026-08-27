//! Rendering for portfolio-authored Markdown.

use crate::domain::ProjectDescription;
use pulldown_cmark::{Event, Options, Parser, html};

/// Renders a project description to HTML.
///
/// Raw HTML is discarded. Images, links, code blocks, tables, and normal
/// `CommonMark` prose remain available to the CMS author.
#[must_use]
pub fn render(description: &ProjectDescription) -> String {
    render_source(description.as_str())
}

/// Renders arbitrary Markdown source to HTML with the same rules as [`render`].
///
/// Used by the admin live preview, where the source may still be incomplete or
/// empty, so it takes a plain string rather than a validated description.
#[must_use]
pub fn render_source(source: &str) -> String {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;
    let events = Parser::new_ext(source, options)
        .filter(|event| !matches!(event, Event::Html(_) | Event::InlineHtml(_)));
    let mut output = String::new();
    html::push_html(&mut output, events);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn description(markdown: &str) -> ProjectDescription {
        crate::test_support::parse(markdown)
    }

    #[test]
    fn renders_project_markdown() {
        let html = render(&description("## System\n\n`Telegram` to **Rust**."));

        assert!(html.contains("<h2>System</h2>"));
        assert!(html.contains("<code>Telegram</code>"));
        assert!(html.contains("<strong>Rust</strong>"));
    }

    #[test]
    fn render_source_takes_incomplete_input() {
        assert_eq!(render_source(""), "");
        // An unterminated emphasis renders as literal text rather than panicking.
        assert!(render_source("**bold").contains("bold"));
    }

    #[test]
    fn discards_raw_html() {
        let html = render(&description("before<script>alert('x')</script>after"));

        assert!(!html.contains("<script>"));
        assert!(html.contains("before"));
        assert!(html.contains("after"));
    }
}
