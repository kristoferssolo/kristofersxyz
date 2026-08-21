use super::AMBER;
use crate::app::{
    content::{FocusArea, PortfolioContent, Profile, ProjectLink, SocialLink, portfolio_content},
    editor::{
        Buffer, Destination, EditorState, Effect, EntryId, Key, KeyInput, Mode, Notification,
        SectionId, Transition, reduce, select,
    },
};
// The editor's `Effect` is the one worth reading unqualified here, so Leptos's
// reactive primitive takes the alias.
use leptos::prelude::Effect as ReactiveEffect;
use leptos::{ev, prelude::*, wasm_bindgen::JsCast, web_sys};
use std::time::Duration;

/// How long a notification stays up. Timing lives here, in the adapter, never
/// in the reducer.
const NOTIFICATION_DURATION: Duration = Duration::from_millis(3_000);

/// Below this width the buffer list and content pane stack, so a selection has
/// to pull the view down to the content. Matches Tailwind's `md` breakpoint.
const STACK_BELOW_PX: f64 = 768.0;

/// A link as it is rendered: visible label, destination, relationship.
struct Link {
    label: &'static str,
    href: &'static str,
    rel: &'static str,
}

/// Where an entry's links come from. `Contact` leads with the address itself
/// so the mail entry shows something you can read rather than the word "Email".
#[derive(Clone, Copy)]
enum Links {
    Social(&'static [SocialLink]),
    Project(&'static [ProjectLink]),
    Contact(Profile),
}

impl Links {
    fn resolve(self) -> Vec<Link> {
        let social = |link: &SocialLink| Link {
            label: link.label,
            href: link.href,
            rel: link.rel,
        };

        match self {
            Self::Social(links) => links.iter().map(social).collect(),
            Self::Project(links) => links
                .iter()
                .map(|link| Link {
                    label: link.label,
                    href: link.href,
                    rel: "noopener noreferrer",
                })
                .collect(),
            Self::Contact(profile) => {
                let address = Link {
                    label: profile.email.trim_start_matches("mailto:"),
                    href: profile.email,
                    rel: "noopener noreferrer",
                };
                std::iter::once(address)
                    .chain(
                        profile
                            .links
                            .iter()
                            .filter(|link| link.label != "Email")
                            .map(social),
                    )
                    .collect()
            }
        }
    }
}

/// One item in the profile pane's action row. `target` is the buffer entry the
/// link selects; the CV download has none, because it leaves the page.
#[derive(Clone)]
struct Action {
    label: &'static str,
    href: String,
    target: Option<EntryId>,
    /// Filename for the download attribute. Only the CV carries one.
    download: Option<&'static str>,
}

/// One navigable line in the buffer, with everything the content pane renders.
/// The editor's own [`Buffer`] carries the same entries in the same order,
/// keyed by the same [`EntryId`].
#[derive(Clone)]
struct Entry {
    id: EntryId,
    section: SectionId,
    name: &'static str,
    /// Shown above the body. Only the profile carries one.
    lead: Option<&'static str>,
    body: &'static str,
    /// Working style lines under the body. Only the profile carries any.
    focus: &'static [FocusArea],
    meta: &'static [&'static str],
    links: Links,
}

/// A notification with the id that owns its timer, so an older timer cannot
/// erase a newer message.
#[derive(Clone)]
struct Notice {
    id: u64,
    message: String,
}

/// The commit the running binary was built from. Absent when the build had no
/// repository and nothing supplied it, in which case the statusline omits the
/// segment rather than showing a placeholder.
const COMMIT: Option<&str> = option_env!("GIT_COMMIT");

/// Moves keyboard focus onto an entry's row, so screen readers announce the
/// row the vim keys just moved to. The rows suppress the global focus ring: a
/// roving `tabindex` means the only focusable row is the selected one, which
/// already reads as selected through the amber marker, the number and the
/// tinted background. A ring on top of that draws a line across the pane on
/// every `j`. No-op during SSR, where there is no document.
fn focus_row(entry: &EntryId) {
    if let Some(element) = document().get_element_by_id(&row_id(entry))
        && let Ok(element) = element.dyn_into::<web_sys::HtmlElement>()
    {
        let _ = element.focus();
    }
}

/// The DOM id of an entry's row in the buffer list.
fn row_id(entry: &EntryId) -> String {
    format!("buffer-{}", entry.fragment())
}

/// True when the viewport is narrow enough that the panes stack vertically.
fn viewport_is_stacked() -> bool {
    web_sys::window()
        .and_then(|window| window.inner_width().ok())
        .and_then(|width| width.as_f64())
        .is_some_and(|width| width < STACK_BELOW_PX)
}

/// On the stacked phone layout the content sits below the whole list, so a
/// selection has to bring it into view. Instant scroll: no motion to undo for
/// reduced-motion readers. A no-op on desktop, where both panes are visible,
/// and during SSR, where there is no document.
fn reveal_content() {
    if viewport_is_stacked()
        && let Some(section) = document().get_element_by_id("buffer-content")
    {
        section.scroll_into_view();
    }
}

/// Opens a repository in a new tab.
fn navigate(destination: &Destination) {
    let Destination::External(url) = destination;
    if let Some(window) = web_sys::window() {
        let _ = window.open_with_url_and_target(url, "_blank");
    }
}

/// The portfolio as a modal editor. Keys are normalized here and handed to the
/// reducer, which owns the whole editor state; this component only renders it
/// and carries out the effects.
#[component]
#[expect(clippy::too_many_lines, reason = "the view! markup is one block")]
pub fn HomePage() -> impl IntoView {
    let content = portfolio_content();
    let rows = entries(&content);
    let groups = group_by_section(&rows);
    let editor = Buffer::from_content(&content);
    let total = editor.len();

    let start = editor.first().unwrap_or_default();
    let actions = StoredValue::new(actions(&rows));
    let entries = StoredValue::new(rows);
    let buffer = StoredValue::new(editor);

    let (state, set_state) = signal(EditorState::new(start));
    let (notice, set_notice) = signal(None::<Notice>);
    let issued = StoredValue::new(0u64);

    // Each notification carries an id; when its timer fires it clears the
    // message only if that id is still the one showing, so an older timer
    // cannot erase a newer message.
    let notify = move |notification: &Notification| {
        issued.update_value(|count| *count += 1);
        let id = issued.get_value();

        set_notice.set(Some(Notice {
            id,
            message: notification.to_string(),
        }));

        set_timeout(
            move || {
                if notice
                    .get_untracked()
                    .is_some_and(|showing| showing.id == id)
                {
                    set_notice.set(None);
                }
            },
            NOTIFICATION_DURATION,
        );
    };

    let apply = move |effect: &Effect| match effect {
        Effect::ScrollTo(entry) => {
            focus_row(entry);
            reveal_content();
        }
        Effect::Navigate(destination) => navigate(destination),
        Effect::Notify(notification) => notify(notification),
        Effect::Dismiss => set_notice.set(None),
        Effect::FocusPage => focus_row(&state.get_untracked().active.entry),
    };

    let advance = move |transition: Transition| {
        set_state.set(transition.state);
        for effect in &transition.effects {
            apply(effect);
        }
    };

    // Bound globally so the keys work wherever the reader's focus sits.
    ReactiveEffect::new(move |_| {
        let handle = window_event_listener(ev::keydown, move |event| {
            let input = KeyInput {
                key: Key::from_name(&event.key()),
                ctrl: event.ctrl_key(),
                alt: event.alt_key(),
                meta: event.meta_key(),
            };

            let current = state.get_untracked();
            let transition = buffer.with_value(|buffer| reduce(&current, input, buffer));

            // A key the editor does not bind changes nothing, so leave it to
            // the browser rather than swallowing it.
            if transition.state == current && transition.effects.is_empty() {
                return;
            }

            event.prevent_default();
            advance(transition);
        });
        on_cleanup(move || handle.remove());
    });

    let pick = move |entry: &EntryId| {
        let transition = buffer.with_value(|buffer| select(&state.get_untracked(), entry, buffer));
        advance(transition);
    };

    let current = move || {
        let active = state.get().active.entry;
        entries.with_value(|entries| entries.iter().find(|entry| entry.id == active).cloned())
    };

    let number_of = move |entry: &EntryId| buffer.with_value(|buffer| buffer.number_of(entry));
    let position = move || number_of(&state.get().active.entry).unwrap_or_default();

    view! {
        <main class="flex min-h-dvh flex-col bg-black font-mono text-[#d4d7db] md:h-dvh md:overflow-hidden">
            <div
                aria-live="polite"
                class="pointer-events-none fixed top-4 right-4 z-10 text-[12.5px]"
            >
                {move || {
                    notice
                        .get()
                        .map(|showing| {
                            view! {
                                <p class="max-w-[46ch] rounded-lg border border-[#2b3037] bg-[#0b0e11] px-3.5 py-2 text-white">
                                    {showing.message}
                                </p>
                            }
                        })
                }}
            </div>

            <div class="grid min-h-0 flex-1 md:grid-cols-[minmax(260px,340px)_minmax(0,1fr)]">
                <div
                    role="listbox"
                    aria-label="Buffers"
                    class="min-h-0 overflow-y-auto border-b border-[#1e2126] py-3 text-[13px] md:border-r md:border-b-0"
                >
                    {groups
                        .into_iter()
                        .map(|(section, ids)| {
                            view! {
                                <div
                                    role="group"
                                    aria-label=section.label()
                                    class="mt-4 first:mt-0"
                                >
                                    <p
                                        aria-hidden="true"
                                        class="mb-1 pl-[7ch] text-[10px] tracking-[0.24em] text-[#4c525a] uppercase"
                                    >
                                        {section.label()}
                                    </p>
                                    {ids
                                        .into_iter()
                                        .map(|(id, name)| {
                                            // Row numbers are stable, so this
                                            // compares two `usize` and stays
                                            // `Copy` for every closure below.
                                            let number = number_of(&id).unwrap_or_default();
                                            let is_active = move || position() == number;
                                            let clicked = id.clone();
                                            let tint = move || {
                                                if is_active() {
                                                    format!("color:{AMBER}")
                                                } else {
                                                    "color:transparent".to_owned()
                                                }
                                            };
                                            let counter = move || {
                                                if is_active() {
                                                    format!("color:{AMBER}")
                                                } else {
                                                    "color:#3c424a".to_owned()
                                                }
                                            };
                                            view! {
                                                <button
                                                    type="button"
                                                    role="option"
                                                    id=row_id(&id)
                                                    aria-selected=move || {
                                                        if is_active() { "true" } else { "false" }
                                                    }
                                                    tabindex=move || if is_active() { 0 } else { -1 }
                                                    on:click=move |_| pick(&clicked)
                                                    class="flex w-full items-baseline gap-[1ch] px-3 py-[3px] text-left hover:bg-[#101317] focus-visible:outline-none"
                                                    class=("bg-[#14181d]", is_active)
                                                >
                                                    <span
                                                        aria-hidden="true"
                                                        class="w-[1ch] shrink-0"
                                                        style=tint
                                                    >
                                                        "\u{258e}"
                                                    </span>
                                                    <span
                                                        aria-hidden="true"
                                                        class="w-[3ch] shrink-0 text-right tabular-nums"
                                                        style=counter
                                                    >
                                                        {move || {
                                                            if is_active() {
                                                                number.to_string()
                                                            } else {
                                                                position().abs_diff(number).to_string()
                                                            }
                                                        }}
                                                    </span>
                                                    <span
                                                        class="truncate"
                                                        class=("text-white", is_active)
                                                        class=("text-[#8b939d]", move || !is_active())
                                                    >
                                                        {name}
                                                    </span>
                                                </button>
                                            }
                                        })
                                        .collect_view()}
                                </div>
                            }
                        })
                        .collect_view()}
                </div>

                <section
                    id="buffer-content"
                    class="min-h-0 overflow-y-auto px-5 py-8 sm:px-10 md:px-14 md:py-14"
                >
                    <div class="max-w-[62ch]">
                        <p class="text-[11px] tracking-[0.24em] text-[#4c525a] uppercase">
                            {move || current().map(|entry| entry.section.label())}
                        </p>
                        <h1 class="mt-3 font-sans text-[clamp(1.75rem,4.5vw,2.75rem)] leading-[1.1] font-semibold text-white">
                            {move || current().map(|entry| entry.name)}
                        </h1>
                        {move || {
                            current()
                                .and_then(|entry| entry.lead)
                                .map(|lead| {
                                    view! {
                                        <p class="mt-4 font-sans text-[17px] leading-[1.5] text-white sm:text-[19px]">
                                            {lead}
                                        </p>
                                    }
                                })
                        }}
                        <p class="mt-5 font-sans text-[16px] leading-[1.7] text-[#aab2bb] sm:text-[17px]">
                            {move || current().map(|entry| entry.body)}
                        </p>
                        {move || {
                            let focus = current().map(|entry| entry.focus).unwrap_or_default();
                            (!focus.is_empty())
                                .then(|| {
                                    view! {
                                        <dl class="mt-5 grid grid-cols-[max-content_minmax(0,1fr)] gap-x-[2ch] gap-y-[5px] text-[13px]">
                                            {focus
                                                .iter()
                                                .map(|area| {
                                                    // One root per item: `contents` keeps the
                                                    // pair in the parent grid without adding a
                                                    // box, and a single root is what hydration
                                                    // walks cleanly.
                                                    view! {
                                                        <div class="contents">
                                                            <dt class="text-white">{area.label}</dt>
                                                            <dd class="text-[#6c757f]">
                                                                {area.detail}
                                                            </dd>
                                                        </div>
                                                    }
                                                })
                                                .collect_view()}
                                        </dl>
                                    }
                                })
                        }}
                        <p class="mt-6 text-[12px] tracking-[0.08em] text-[#6c757f]">
                            {move || {
                                current().map(|entry| entry.meta.join("  \u{b7}  "))
                            }}
                        </p>
                        {move || {
                            current()
                                .filter(|entry| entry.section == SectionId::Profile)
                                .map(|_| {
                                    view! {
                                        <div class="mt-8 flex flex-wrap gap-x-6 gap-y-2 text-[13px]">
                                            {actions
                                                .get_value()
                                                .into_iter()
                                                .map(|action| {
                                                    let hint = action
                                                        .target
                                                        .as_ref()
                                                        .and_then(&number_of);
                                                    let target = action.target.clone();
                                                    view! {
                                                        <a
                                                            class="text-white underline decoration-[#3c424a] underline-offset-[5px] hover:decoration-[#e2a340]"
                                                            href=action.href
                                                            download=action.download
                                                            on:click=move |_| {
                                                                if let Some(entry) = target.as_ref() {
                                                                    pick(entry);
                                                                }
                                                            }
                                                        >
                                                            {action.label}
                                                            {hint
                                                                .map(|number| {
                                                                    view! {
                                                                        <span
                                                                            aria-hidden="true"
                                                                            class="ml-[1ch] hidden text-[#4c525a] no-underline md:inline"
                                                                        >
                                                                            {format!("[{number}]")}
                                                                        </span>
                                                                    }
                                                                })}
                                                        </a>
                                                    }
                                                })
                                                .collect_view()}
                                        </div>
                                    }
                                })
                        }}
                        <div class="mt-6 flex flex-wrap gap-x-6 gap-y-2 text-[13px]">
                            {move || {
                                current()
                                    .map_or(Links::Project(&[]), |entry| entry.links)
                                    .resolve()
                                    .into_iter()
                                    .map(|link| {
                                        view! {
                                            <a
                                                class="text-white underline decoration-[#3c424a] underline-offset-[5px] hover:decoration-[#e2a340]"
                                                href=link.href
                                                rel=link.rel
                                            >
                                                {link.label}
                                            </a>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </div>
                    </div>

                    <div class="mt-14 max-w-[62ch] border-t border-[#1e2126] pt-4">
                        {move || {
                            let active = state.get().active.entry;
                            let next = buffer
                                .with_value(|buffer| buffer.next(&active))
                                .map(|selection| selection.entry)
                                .filter(|entry| entry != &active)?;
                            let name = entries
                                .with_value(|entries| {
                                    entries
                                        .iter()
                                        .find(|entry| entry.id == next)
                                        .map(|entry| entry.name)
                                })?;
                            Some(
                                view! {
                                    <button
                                        type="button"
                                        on:click=move |_| pick(&next)
                                        class="flex w-full items-baseline gap-[2ch] text-left text-[12px] text-[#6c757f] hover:text-white"
                                    >
                                        <span class="text-[#e2a340]">
                                            <span class="hidden md:inline">"j"</span>
                                            <span aria-hidden="true" class="md:hidden">
                                                "\u{2193}"
                                            </span>
                                        </span>
                                        <span>{name}</span>
                                    </button>
                                },
                            )
                        }}
                    </div>
                </section>
            </div>

            // One row, flush with the viewport edge. At rest it is the
            // statusline; opening `:` or `/` replaces it in place at the same
            // height, so the content pane never reflows.
            <footer class="h-7 shrink-0">
                {move || match state.get().mode {
                    Mode::Normal => {
                        view! {
                            <div
                                class="flex h-full items-stretch text-[12px] text-[#8b939d]"
                                style="background:#0d1013"
                            >
                                <span
                                    class="flex items-center px-3 font-semibold text-black"
                                    style=format!("background:{AMBER}")
                                >
                                    "NORMAL"
                                </span>
                                <span class="flex items-center px-3 text-white">
                                    "kristofers.xyz"
                                </span>
                                <span class="hidden items-center px-3 text-[#6c757f] md:flex">
                                    ":help"
                                </span>
                                <span class="ml-auto flex items-stretch">
                                    <span class="flex items-center px-3 tabular-nums">
                                        {move || format!("[{}/{total}]", position())}
                                    </span>
                                    <span class="flex items-center px-3 tabular-nums">
                                        {move || format!("{}%", position() * 100 / total)}
                                    </span>
                                    {COMMIT
                                        .map(|hash| {
                                            view! {
                                                <span class="hidden items-center px-3 md:flex">
                                                    {hash}
                                                </span>
                                            }
                                        })}
                                </span>
                            </div>
                        }
                            .into_any()
                    }
                    Mode::Command(text) => command_line(":", &text).into_any(),
                    Mode::Search(query) => command_line("/", &query).into_any(),
                }}
            </footer>
        </main>
    }
}

/// The command line, in place of the statusline. The prompt is rendered here;
/// the reducer only ever holds the text after it.
fn command_line(prompt: &'static str, text: &str) -> impl IntoView {
    view! {
        <div class="flex h-full items-center gap-[1ch] rounded-md border border-[#2b3037] bg-[#0b0e11] px-2.5 text-[13px] text-white">
            <span aria-hidden="true" class="text-[#e2a340]">
                "\u{276f}"
            </span>
            <span>{format!("{prompt}{text}")}</span>
            <span aria-hidden="true" class="inline-block h-[15px] w-[1ch] bg-white"></span>
        </div>
    }
}

/// Flattens the portfolio into the buffer's line list, in the same order and
/// under the same ids as the editor's own [`Buffer`].
fn entries(content: &PortfolioContent) -> Vec<Entry> {
    let profile = content.profile;
    let mut entries = vec![Entry {
        id: EntryId::Profile,
        section: SectionId::Profile,
        name: profile.name,
        lead: Some(profile.title),
        body: profile.about,
        focus: profile.working_style,
        meta: profile.stack,
        links: Links::Social(profile.links),
    }];

    entries.extend(content.projects.iter().map(|project| Entry {
        id: EntryId::Project(project.name.to_owned()),
        section: SectionId::Work,
        name: project.name,
        lead: None,
        body: project.summary,
        focus: &[],
        meta: project.stack,
        links: Links::Project(project.links),
    }));

    entries.push(Entry {
        id: EntryId::Contact,
        section: SectionId::Contact,
        name: content.contact.name,
        lead: None,
        body: content.contact.body,
        focus: &[],
        meta: &[],
        links: Links::Contact(profile),
    });

    entries
}

/// The profile pane's explicit next steps, for a reader who never discovers
/// the vim layer. Ordered by what a hiring reader wants first.
fn actions(entries: &[Entry]) -> Vec<Action> {
    let find = |section| {
        entries
            .iter()
            .find(|entry| entry.section == section)
            .map(|entry| entry.id.clone())
    };
    let go = |label, entry: EntryId| Action {
        label,
        href: format!("#{}", entry.fragment()),
        target: Some(entry),
        download: None,
    };

    find(SectionId::Work)
        .map(|entry| go("View work", entry))
        .into_iter()
        .chain(std::iter::once(Action {
            label: "Download CV",
            href: "/cv.pdf".to_owned(),
            target: None,
            download: Some("kristofers-solo-cv.pdf"),
        }))
        .chain(find(SectionId::Contact).map(|entry| go("Contact", entry)))
        .collect()
}

/// Groups entry ids under their section, preserving order, so the listbox can
/// render one `group` per section. Sections are contiguous in [`entries`], so
/// a fold over adjacent rows is enough.
fn group_by_section(entries: &[Entry]) -> Vec<(SectionId, Vec<(EntryId, &'static str)>)> {
    let mut groups: Vec<(SectionId, Vec<(EntryId, &'static str)>)> = Vec::new();

    for entry in entries {
        let row = (entry.id.clone(), entry.name);
        match groups.last_mut() {
            Some((section, rows)) if *section == entry.section => rows.push(row),
            _ => groups.push((entry.section, vec![row])),
        }
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_none, assert_some_eq};

    /// The content pane and the editor read the same entries in the same
    /// order. They are built separately, so this is what keeps them from
    /// drifting apart.
    #[test]
    fn the_rendered_entries_match_the_editor_buffer() {
        let content = portfolio_content();
        let rendered = entries(&content)
            .into_iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>();
        let editor = Buffer::from_content(&content)
            .entries()
            .iter()
            .map(|entry| entry.id.clone())
            .collect::<Vec<_>>();

        assert_eq!(rendered, editor);
    }

    #[test]
    fn sections_group_in_order() {
        let sections = group_by_section(&entries(&portfolio_content()))
            .into_iter()
            .map(|(section, rows)| (section, rows.len()))
            .collect::<Vec<_>>();

        assert_eq!(
            sections,
            [
                (SectionId::Profile, 1),
                (SectionId::Work, 3),
                (SectionId::Contact, 1),
            ]
        );
    }

    #[test]
    fn the_action_row_points_at_work_the_cv_and_contact() {
        let actions = actions(&entries(&portfolio_content()));
        let hrefs = actions
            .iter()
            .map(|action| action.href.as_str())
            .collect::<Vec<_>>();

        assert_eq!(hrefs, ["#work-guenther", "/cv.pdf", "#contact"]);
        assert_some_eq!(
            actions[0].target.clone(),
            EntryId::Project("guenther".to_owned())
        );
        assert_none!(actions[1].target.clone(), "the CV leaves the page");
        assert_some_eq!(actions[2].target.clone(), EntryId::Contact);
    }
}
