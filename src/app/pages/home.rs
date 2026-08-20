use super::AMBER;
use crate::app::content::{
    FocusArea, PortfolioContent, Profile, ProjectLink, SocialLink, portfolio_content,
};
use leptos::{ev, prelude::*, wasm_bindgen::JsCast, web_sys};

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

/// One navigable line in the buffer.
#[derive(Clone, Copy)]
struct Entry {
    section: &'static str,
    name: &'static str,
    /// Shown above the body. Only the profile carries one.
    lead: Option<&'static str>,
    body: &'static str,
    /// Working style lines under the body. Only the profile carries any.
    focus: &'static [FocusArea],
    meta: &'static [&'static str],
    links: Links,
}

/// Moves keyboard focus onto the buffer option at `index`, so screen readers
/// announce the row the vim keys just moved to. No-op during SSR, where there
/// is no document.
fn focus_option(index: usize) {
    if let Some(element) = document().get_element_by_id(&format!("buffer-{index}"))
        && let Ok(element) = element.dyn_into::<web_sys::HtmlElement>()
    {
        let _ = element.focus();
    }
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

/// The portfolio as a modal editor. The buffer list moves with `j`/`k`, `g`/`G`
/// and the number keys; the statusline reports position; and the page sits
/// inside one viewport so navigation replaces scrolling. On a stacked phone
/// layout, selecting a buffer pulls the view down to its content.
#[component]
#[expect(clippy::too_many_lines, reason = "the view! markup is one block")]
pub fn HomePage() -> impl IntoView {
    let entries = entries(&portfolio_content());
    let total = entries.len();
    let last = total - 1;
    let groups = group_by_section(&entries);
    let stored = StoredValue::new(entries);
    let (active, set_active) = signal(0usize);

    // Every selection path funnels through here: update the row, optionally move
    // DOM focus onto it for screen readers, then reveal the content on mobile.
    let select = move |index: usize, focus: bool| {
        set_active.set(index);
        if focus {
            focus_option(index);
        }
        reveal_content();
    };

    // Vim-style keys, bound globally so they work wherever the reader's focus
    // sits. Selecting a row also moves DOM focus onto it for screen readers.
    Effect::new(move |_| {
        let handle = window_event_listener(ev::keydown, move |event| {
            if event.ctrl_key() || event.meta_key() || event.alt_key() {
                return;
            }
            let current = active.get_untracked();
            let next = match event.key().as_str() {
                "j" | "ArrowDown" => (current + 1).min(last),
                "k" | "ArrowUp" => current.saturating_sub(1),
                "g" => 0,
                "G" => last,
                digit @ ("1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9") => {
                    match digit.parse::<usize>() {
                        Ok(n) if n <= total => n - 1,
                        _ => return,
                    }
                }
                _ => return,
            };
            event.prevent_default();
            select(next, true);
        });
        on_cleanup(move || handle.remove());
    });

    let current = move || stored.with_value(|entries| entries[active.get()]);

    view! {
        <main class="flex min-h-dvh flex-col bg-black font-mono text-[#d4d7db] md:h-dvh md:overflow-hidden">
            <div class="grid min-h-0 flex-1 md:grid-cols-[minmax(260px,340px)_minmax(0,1fr)]">
                <div
                    role="listbox"
                    aria-label="Buffers"
                    class="min-h-0 overflow-y-auto border-b border-[#1e2126] py-3 text-[13px] md:border-r md:border-b-0"
                >
                    {groups
                        .into_iter()
                        .map(|(section, rows)| {
                            view! {
                                <div role="group" aria-label=section class="mt-4 first:mt-0">
                                    <p
                                        aria-hidden="true"
                                        class="mb-1 pl-[7ch] text-[10px] tracking-[0.24em] text-[#4c525a] uppercase"
                                    >
                                        {section}
                                    </p>
                                    {rows
                                        .into_iter()
                                        .map(|(i, name)| {
                                            let is_active = move || active.get() == i;
                                            let marker = move || {
                                                if is_active() {
                                                    format!("color:{AMBER}")
                                                } else {
                                                    "color:transparent".to_owned()
                                                }
                                            };
                                            let number = move || {
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
                                                    id=format!("buffer-{i}")
                                                    aria-selected=move || {
                                                        if is_active() { "true" } else { "false" }
                                                    }
                                                    tabindex=move || if is_active() { 0 } else { -1 }
                                                    on:click=move |_| select(i, false)
                                                    class="flex w-full items-baseline gap-[1ch] px-3 py-[3px] text-left hover:bg-[#101317]"
                                                    class=("bg-[#14181d]", is_active)
                                                >
                                                    <span
                                                        aria-hidden="true"
                                                        class="w-[1ch] shrink-0"
                                                        style=marker
                                                    >
                                                        "\u{258e}"
                                                    </span>
                                                    <span
                                                        aria-hidden="true"
                                                        class="w-[3ch] shrink-0 text-right tabular-nums"
                                                        style=number
                                                    >
                                                        {move || {
                                                            if is_active() {
                                                                (i + 1).to_string()
                                                            } else {
                                                                active.get().abs_diff(i).to_string()
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
                            {move || current().section}
                        </p>
                        <h1 class="mt-3 font-sans text-[clamp(1.75rem,4.5vw,2.75rem)] leading-[1.1] font-semibold text-white">
                            {move || current().name}
                        </h1>
                        {move || {
                            current()
                                .lead
                                .map(|lead| {
                                    view! {
                                        <p class="mt-4 font-sans text-[17px] leading-[1.5] text-white sm:text-[19px]">
                                            {lead}
                                        </p>
                                    }
                                })
                        }}
                        <p class="mt-5 font-sans text-[16px] leading-[1.7] text-[#aab2bb] sm:text-[17px]">
                            {move || current().body}
                        </p>
                        {move || {
                            let focus = current().focus;
                            (!focus.is_empty())
                                .then(|| {
                                    view! {
                                        <ul class="mt-5 space-y-1 text-[13px]">
                                            {focus
                                                .iter()
                                                .map(|area| {
                                                    view! {
                                                        <li class="flex flex-wrap gap-x-[1ch]">
                                                            <span class="text-white">{area.label}</span>
                                                            <span class="text-[#6c757f]">{area.detail}</span>
                                                        </li>
                                                    }
                                                })
                                                .collect_view()}
                                        </ul>
                                    }
                                })
                        }}
                        <p class="mt-6 text-[12px] tracking-[0.08em] text-[#6c757f]">
                            {move || current().meta.join("  ·  ")}
                        </p>
                        <div class="mt-6 flex flex-wrap gap-x-6 gap-y-2 text-[13px]">
                            {move || {
                                current()
                                    .links
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
                            let next = (active.get() + 1).min(last);
                            let name = stored.with_value(|entries| entries[next].name);
                            (next != active.get())
                                .then(|| {
                                    view! {
                                        <button
                                            type="button"
                                            on:click=move |_| select(next, true)
                                            class="flex w-full items-baseline gap-[2ch] text-left text-[12px] text-[#6c757f] hover:text-white"
                                        >
                                            <span class="text-[#e2a340]">
                                                <span class="hidden md:inline">"j"</span>
                                                <span aria-hidden="true" class="md:hidden">"\u{2193}"</span>
                                            </span>
                                            <span>{name}</span>
                                        </button>
                                    }
                                })
                        }}
                    </div>
                </section>
            </div>

            <footer class="flex shrink-0 items-stretch text-[12px] text-[#8b939d]" style="background:#0d1013">
                <span
                    class="px-3 py-1 font-semibold text-black"
                    style=format!("background:{AMBER}")
                >
                    "NORMAL"
                </span>
                <span class="truncate px-3 py-1 text-white">"kristofers.xyz"</span>
                <span class="ml-auto px-3 py-1 tabular-nums">
                    {move || format!("{}:{total}", active.get() + 1)}
                </span>
                <span class="hidden px-3 py-1 md:inline">"j k to move"</span>
                <span class="px-3 py-1 tabular-nums">
                    {move || format!("{}%", (active.get() + 1) * 100 / total)}
                </span>
            </footer>
        </main>
    }
}

/// Flattens the portfolio into the buffer's line list. Takes the content by
/// reference so the source can later be a database query rather than the
/// static [`PORTFOLIO`].
fn entries(content: &PortfolioContent) -> Vec<Entry> {
    let profile = content.profile;
    let mut entries = vec![Entry {
        section: "profile",
        name: profile.name,
        lead: Some(profile.title),
        body: profile.about,
        focus: profile.working_style,
        meta: profile.stack,
        links: Links::Social(profile.links),
    }];

    entries.extend(content.projects.iter().map(|project| Entry {
        section: "work",
        name: project.name,
        lead: None,
        body: project.summary,
        focus: &[],
        meta: project.stack,
        links: Links::Project(project.links),
    }));

    entries.push(Entry {
        section: "contact",
        name: content.contact.name,
        lead: None,
        body: content.contact.body,
        focus: &[],
        meta: &[],
        links: Links::Contact(profile),
    });

    entries
}

/// Groups entry indices under their section heading, preserving order, so the
/// listbox can render one `group` per section. Sections are contiguous in
/// [`entries`], so a fold over adjacent rows is enough.
fn group_by_section(entries: &[Entry]) -> Vec<(&'static str, Vec<(usize, &'static str)>)> {
    let mut groups: Vec<(&'static str, Vec<(usize, &'static str)>)> = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        match groups.last_mut() {
            Some((section, rows)) if *section == entry.section => rows.push((i, entry.name)),
            _ => groups.push((entry.section, vec![(i, entry.name)])),
        }
    }
    groups
}
