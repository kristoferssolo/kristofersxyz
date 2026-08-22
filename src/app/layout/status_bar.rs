use leptos::prelude::*;
use std::fmt::{self, Display, Formatter};

/// Statusline amber, the only colour on the site besides greys.
const AMBER: &str = "#e2a340";

/// The position vocabulary rendered at the right edge of the status bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatusLocation {
    Page { current: usize, total: usize },
    Cursor { line: usize, column: usize },
}

impl StatusLocation {
    fn progress(&self) -> Option<usize> {
        match self {
            Self::Page { current, total } if *total > 0 => {
                Some(current.saturating_mul(100).div_euclid(*total).min(100))
            }
            Self::Page { .. } | Self::Cursor { .. } => None,
        }
    }
}

impl Display for StatusLocation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Page { current, total } => write!(formatter, "[{current}/{total}]"),
            Self::Cursor { line, column } => write!(formatter, "{line}:{column}"),
        }
    }
}

/// Everything the shared status bar can display. Its representation stays
/// private so page modules cannot depend on markup or layout details.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusBarState(StatusBarContent);

#[derive(Clone, Debug, PartialEq, Eq)]
enum StatusBarContent {
    Normal {
        filename: String,
        location: StatusLocation,
        show_help: bool,
        show_progress: bool,
    },
    Command {
        prompt: char,
        text: String,
    },
}

impl StatusBarState {
    pub fn normal(filename: impl Into<String>, location: StatusLocation) -> Self {
        Self(StatusBarContent::Normal {
            filename: filename.into(),
            location,
            show_help: false,
            show_progress: false,
        })
    }

    #[must_use]
    pub const fn with_help(mut self) -> Self {
        if let StatusBarContent::Normal { show_help, .. } = &mut self.0 {
            *show_help = true;
        }
        self
    }

    #[must_use]
    pub const fn with_progress(mut self) -> Self {
        if let StatusBarContent::Normal { show_progress, .. } = &mut self.0 {
            *show_progress = true;
        }
        self
    }

    pub const fn command(prompt: char, text: String) -> Self {
        Self(StatusBarContent::Command { prompt, text })
    }
}

#[component]
pub(super) fn StatusBar(#[prop(into)] state: Signal<StatusBarState>) -> impl IntoView {
    view! {
        <footer class="h-7 shrink-0 overflow-hidden">
            {move || match state.get().0 {
                StatusBarContent::Normal {
                    filename,
                    location,
                    show_help,
                    show_progress,
                } => {
                    let progress = show_progress.then(|| location.progress()).flatten();
                    view! {
                        <div
                            class="flex h-full items-stretch text-[12px] leading-none text-[#8b939d]"
                            style="background:#0d1013"
                        >
                            <span
                                class="flex items-center px-3 font-semibold text-black"
                                style=format!("background:{AMBER}")
                            >
                                "NORMAL"
                            </span>
                            <span class="flex min-w-0 items-center truncate px-3 text-white">
                                {filename}
                            </span>
                            {show_help.then(|| {
                                view! {
                                    <span class="hidden items-center px-3 text-[#6c757f] md:flex">
                                        ":help"
                                    </span>
                                }
                            })}
                            <span class="ml-auto flex items-stretch">
                                <span class="flex items-center px-3 tabular-nums">
                                    {location.to_string()}
                                </span>
                                {progress.map(|percentage| {
                                    view! {
                                        <span class="flex items-center px-3 tabular-nums">
                                            {format!("{percentage}%")}
                                        </span>
                                    }
                                })}
                            </span>
                        </div>
                    }
                    .into_any()
                }
                StatusBarContent::Command { prompt, text } => {
                    view! { <CommandLine prompt text /> }.into_any()
                }
            }}
        </footer>
    }
}

#[component]
fn CommandLine(prompt: char, text: String) -> impl IntoView {
    view! {
        <div class="flex h-full items-center gap-[1ch] rounded-md border border-[#2b3037] bg-[#0b0e11] px-2.5 text-[13px] text-white">
            <span aria-hidden="true" class="text-[#e2a340]">"\u{276f}"</span>
            <span>{format!("{prompt}{text}")}</span>
            <span aria-hidden="true" class="inline-block h-[15px] w-[1ch] bg-white"></span>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_none;

    #[test]
    fn locations_use_editor_notation() {
        assert_eq!(
            StatusLocation::Page {
                current: 2,
                total: 5
            }
            .to_string(),
            "[2/5]"
        );
        assert_eq!(
            StatusLocation::Cursor { line: 0, column: 0 }.to_string(),
            "0:0"
        );
    }

    #[test]
    fn page_progress_handles_an_empty_sequence() {
        let empty = StatusLocation::Page {
            current: 0,
            total: 0,
        };

        assert_none!(empty.progress());
    }
}
