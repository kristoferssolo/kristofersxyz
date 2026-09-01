//! The Project Screenshots on a Project Detail.
//!
//! Screenshots are the visual half of the Project Evidence, so they run the
//! full width of the reading column rather than sitting in the aside beside
//! it. Each one is a figure with the alternative text the Owner wrote and, when
//! there is one, a caption. Width and height are always set, so the page holds
//! the right space before an image arrives and nothing below it jumps.
//!
//! A Project without screenshots renders nothing at all: no heading, no rule,
//! and no placeholder standing in for evidence that does not exist.

use crate::domain::{ProjectScreenshot, ProjectScreenshots};
use leptos::prelude::*;

/// The screenshot figures, in the order the Owner arranged them.
#[component]
pub fn ScreenshotFigures(screenshots: ProjectScreenshots) -> impl IntoView {
    let count = screenshots.len();

    (count != 0).then(|| {
        view! {
            <section class="mt-16">
                <div class="flex items-baseline justify-between gap-[2ch] border-t border-[#1e2126] pt-[.8rem]">
                    <h2 class="m-0 text-[10px] font-normal tracking-[0.2em] text-[#8b939d] uppercase">
                        "Screenshots"
                    </h2>
                    <span class="text-[11px] text-[#59616a]">{tally(count)}</span>
                </div>
                {screenshots
                    .into_iter()
                    .enumerate()
                    .map(|(index, screenshot)| {
                        view! { <Figure screenshot below_the_fold=index != 0 /> }
                    })
                    .collect_view()}
            </section>
        }
    })
}

/// One screenshot at the full width of the reading column.
///
/// Only the first figure is likely to be on screen when the page loads, so
/// every other one is left to load lazily and decode off the main thread.
#[component]
fn Figure(screenshot: ProjectScreenshot, below_the_fold: bool) -> impl IntoView {
    let caption = screenshot
        .caption
        .as_ref()
        .map(|caption| view! { <figcaption class="mt-3 max-w-[62ch] font-sans text-[13px] leading-[1.6] text-[#8b939d]">{caption.to_string()}</figcaption> });

    view! {
        <figure class="mt-6">
            <img
                class="block h-auto w-full border border-[#1e2126] bg-[#0b0e11]"
                src=screenshot.media_path()
                width=screenshot.size.width()
                height=screenshot.size.height()
                alt=screenshot.alt.to_string()
                loading=below_the_fold.then_some("lazy")
                decoding=below_the_fold.then_some("async")
            />
            {caption}
        </figure>
    }
}

/// How many screenshots the section holds, said in words a reader expects.
fn tally(count: usize) -> String {
    if count == 1 {
        "1 screenshot".to_owned()
    } else {
        format!("{count} screenshots")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tally_names_one_screenshot_in_the_singular() {
        assert_eq!(tally(1), "1 screenshot");
        assert_eq!(tally(3), "3 screenshots");
    }
}
