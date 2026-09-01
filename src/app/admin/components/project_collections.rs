//! The ordered Technology and Project Link buffers in the project editor.
//!
//! Both collections are edited as numbered lines in the rail beside the
//! Project Description. Every line carries its position and the three actions
//! that change it: move up, move down, and remove. Rows can also be dragged
//! directly; the buttons remain available as the keyboard and assistive
//! technology fallback.
//!
//! Each line owns the signal behind its own field, so typing never re-renders
//! the list and a move never costs the Owner their focus or their text. Field
//! names carry the line's live position, which is what the save reads. Fields
//! render both the value attribute and the value property, so the form carries
//! the stored text whether or not the page has hydrated.

use super::super::error::AdminError;
use crate::domain::{ProjectLinks, ProjectTechnologies};
use leptos::{ev, prelude::*, web_sys::DragEvent};
use lucide_leptos::{ChevronDown, ChevronUp, Trash2};

/// Which line the last save rejected, so the editor can mark it and repeat the
/// reason beside it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RejectedLine {
    Technology(usize),
    LinkLabel(usize),
    LinkUrl(usize),
}

impl RejectedLine {
    /// The line a save rejection names, or `None` for a failure that belongs
    /// to the form as a whole.
    const fn from_error(error: &AdminError) -> Option<Self> {
        match *error {
            AdminError::InvalidTechnology { position }
            | AdminError::RepeatedTechnology { position } => Some(Self::Technology(position)),
            AdminError::InvalidLinkLabel { position }
            | AdminError::RepeatedLinkLabel { position } => Some(Self::LinkLabel(position)),
            AdminError::InvalidLinkUrl { position } => Some(Self::LinkUrl(position)),
            _ => None,
        }
    }
}

/// The save rejection the editor rail reacts to.
#[derive(Clone, Copy)]
pub struct SaveRejection(Signal<Option<AdminError>>);

impl SaveRejection {
    #[must_use]
    pub const fn new(error: Signal<Option<AdminError>>) -> Self {
        Self(error)
    }

    /// Whether the rejection names a line, which is when the form's own
    /// message becomes a summary instead of the whole story.
    #[must_use]
    pub fn marks_a_line(self) -> bool {
        self.0
            .get()
            .as_ref()
            .and_then(RejectedLine::from_error)
            .is_some()
    }

    /// The reason to show beside `line`, if that is the line that failed.
    fn message_for(self, line: RejectedLine) -> Option<String> {
        self.0.get().and_then(|error| {
            (RejectedLine::from_error(&error) == Some(line)).then(|| error.to_string())
        })
    }
}

/// One editable Technology line.
#[derive(Clone, Copy)]
struct TechnologyRow {
    id: usize,
    value: RwSignal<String>,
}

/// One editable Project Link line, holding the label above the URL.
#[derive(Clone, Copy)]
struct LinkRow {
    id: usize,
    label: RwSignal<String>,
    href: RwSignal<String>,
}

/// Tracks one collection's native drag operation and the row currently under
/// the pointer. Technologies and links each get their own state, so a drag
/// cannot cross the collection boundary.
#[derive(Clone, Copy)]
struct DragState {
    dragged: RwSignal<Option<usize>>,
    target: RwSignal<Option<usize>>,
}

impl DragState {
    fn new() -> Self {
        Self {
            dragged: RwSignal::new(None),
            target: RwSignal::new(None),
        }
    }

    fn is_dragging(self, index: usize) -> bool {
        self.dragged.get().is_some_and(|dragged| dragged == index)
    }

    fn is_drop_target(self, index: usize) -> bool {
        self.dragged.get().is_some() && self.target.get().is_some_and(|target| target == index)
    }

    fn start(self, index: usize, event: &DragEvent) {
        if let Some(transfer) = event.data_transfer() {
            transfer.set_effect_allowed("move");
            let _ = transfer.set_data("text/plain", &index.to_string());
        }
        self.target.set(None);
        self.dragged.set(Some(index));
    }

    fn over(self, index: usize, event: &DragEvent) {
        event.prevent_default();
        if self.dragged.get_untracked().is_some() {
            if let Some(transfer) = event.data_transfer() {
                transfer.set_drop_effect("move");
            }
            self.target.set(Some(index));
        }
    }

    fn drop<T: Clone + Send + Sync + 'static>(
        self,
        rows: RowList<T>,
        index: usize,
        event: &DragEvent,
    ) {
        event.prevent_default();
        if let Some(dragged) = self.dragged.get_untracked() {
            rows.move_to(dragged, index);
        }
        self.clear();
    }

    fn clear(self) {
        self.target.set(None);
        self.dragged.set(None);
    }
}

/// The rows of one ordered collection and the moves its buttons perform.
///
/// Button moves swap two occupied positions; a drag inserts the row before its
/// target. No action can lose, duplicate, or reorder anything the Owner did not
/// ask for.
#[derive(Clone, Copy)]
struct RowList<T: Send + Sync + 'static> {
    rows: RwSignal<Vec<T>>,
    next_id: RwSignal<usize>,
}

impl<T: Clone + Send + Sync + 'static> RowList<T> {
    fn new(rows: Vec<T>) -> Self {
        let next_id = rows.len();
        Self {
            rows: RwSignal::new(rows),
            next_id: RwSignal::new(next_id),
        }
    }

    /// A row identity that no existing row uses, so a keyed list keeps the
    /// fields it already rendered.
    fn take_id(self) -> usize {
        let id = self.next_id.get_untracked();
        self.next_id.set(id.saturating_add(1));
        id
    }

    fn len(self) -> usize {
        self.rows.with(Vec::len)
    }

    fn has_rows(self) -> bool {
        !self.rows.with(Vec::is_empty)
    }

    fn append(self, row: T) {
        self.rows.update(|rows| rows.push(row));
    }

    fn remove(self, index: usize) {
        self.rows.update(|rows| remove_row(rows, index));
    }

    fn move_up(self, index: usize) {
        self.rows.update(|rows| move_up(rows, index));
    }

    fn move_down(self, index: usize) {
        self.rows.update(|rows| move_down(rows, index));
    }

    fn move_to(self, index: usize, target: usize) {
        self.rows.update(|rows| move_to(rows, index, target));
    }
}

/// Swaps the row at `index` with the one above it. A move off either end of
/// the collection leaves the order alone, which is what the disabled button at
/// that end already tells the Owner.
const fn move_up<T>(rows: &mut [T], index: usize) {
    if let Some(previous) = index.checked_sub(1)
        && index < rows.len()
    {
        rows.swap(previous, index);
    }
}

/// Swaps the row at `index` with the one below it.
const fn move_down<T>(rows: &mut [T], index: usize) {
    let next = index.saturating_add(1);
    if next < rows.len() {
        rows.swap(index, next);
    }
}

/// Moves a row immediately before the row it was dropped on.
fn move_to<T>(rows: &mut Vec<T>, index: usize, target: usize) {
    if index == target || index >= rows.len() || target >= rows.len() {
        return;
    }

    let row = rows.remove(index);
    let insertion = if index < target {
        target.saturating_sub(1)
    } else {
        target
    };
    rows.insert(insertion, row);
}

/// Drops the row at `index`, leaving the rows around it in their order.
fn remove_row<T>(rows: &mut Vec<T>, index: usize) {
    if index < rows.len() {
        rows.remove(index);
    }
}

/// The editor rail: the Technologies buffer above the links buffer.
#[component]
pub fn ProjectCollections(
    technologies: ProjectTechnologies,
    links: ProjectLinks,
    rejection: SaveRejection,
) -> impl IntoView {
    let technology_drag = DragState::new();
    let link_drag = DragState::new();
    let technology_rows = RowList::new(
        technologies
            .into_iter()
            .enumerate()
            .map(|(id, name)| TechnologyRow {
                id,
                value: RwSignal::new(name.to_string()),
            })
            .collect(),
    );
    let link_rows = RowList::new(
        links
            .into_iter()
            .enumerate()
            .map(|(id, link)| LinkRow {
                id,
                label: RwSignal::new(link.label.to_string()),
                href: RwSignal::new(link.href.to_string()),
            })
            .collect(),
    );

    view! {
        <div class="mt-7 border-t border-[#1e2126] pt-6 min-[961px]:sticky min-[961px]:top-12 min-[961px]:mt-0 min-[961px]:border-t-0 min-[961px]:border-l min-[961px]:pt-0 min-[961px]:pl-7">
            <section class="border border-[#1e2126]">
                <BufferHeading name="technologies" count=Signal::derive(move || technology_rows.len()) />
                <Show
                    when=move || technology_rows.has_rows()
                    fallback=|| {
                        view! {
                            <EmptyBuffer>
                                <b class="font-medium text-white">"No technologies yet."</b>
                                " Add the languages, frameworks, and tools that shaped this project."
                            </EmptyBuffer>
                        }
                    }
                >
                    <ForEnumerate
                        each=move || technology_rows.rows.get()
                        key=|row: &TechnologyRow| row.id
                        children=move |index, row| {
                            view! { <TechnologyLine rows=technology_rows index row rejection drag=technology_drag /> }
                        }
                    />
                </Show>
                <AddLine
                    label="add technology"
                    on_add=move || {
                        technology_rows
                            .append(TechnologyRow {
                                id: technology_rows.take_id(),
                                value: RwSignal::new(String::new()),
                            });
                    }
                />
            </section>

            <section class="mt-7 border border-[#1e2126]">
                <BufferHeading name="links" count=Signal::derive(move || link_rows.len()) />
                <Show
                    when=move || link_rows.has_rows()
                    fallback=|| {
                        view! {
                            <EmptyBuffer>
                                <b class="font-medium text-white">"No links yet."</b>
                                " A project can stay link free. Add one when there is somewhere useful to send a reader."
                            </EmptyBuffer>
                        }
                    }
                >
                    <ForEnumerate
                        each=move || link_rows.rows.get()
                        key=|row: &LinkRow| row.id
                        children=move |index, row| {
                            view! { <LinkLine rows=link_rows index row rejection drag=link_drag /> }
                        }
                    />
                </Show>
                <AddLine
                    label="add link"
                    on_add=move || {
                        link_rows
                            .append(LinkRow {
                                id: link_rows.take_id(),
                                label: RwSignal::new(String::new()),
                                href: RwSignal::new(String::new()),
                            });
                    }
                />
            </section>
        </div>
    }
}

#[component]
fn BufferHeading(name: &'static str, count: Signal<usize>) -> impl IntoView {
    view! {
        <div class="flex items-baseline justify-between gap-[1ch] border-b border-[#1e2126] bg-[#050607] px-[.8rem] py-[.5rem]">
            <span class="text-[11px] tracking-[.04em] text-[#c3c9cf]">{name}</span>
            <span class="text-[11px] text-[#767d87]">
                {move || {
                    let count = count.get();
                    let unit = if count == 1 { "line" } else { "lines" };
                    format!("{count} {unit}")
                }}
            </span>
        </div>
    }
}

#[component]
fn EmptyBuffer(children: Children) -> impl IntoView {
    view! {
        <p class="m-0 px-[.8rem] py-[.9rem] text-[12px] leading-[1.55] text-[#8b939d]">
            {children()}
        </p>
    }
}

const LINE: &str = "grid grid-cols-[3.4ch_minmax(0,1fr)_auto] items-stretch gap-x-[.8ch] \
    relative cursor-grab border-b border-[#101317] pr-[.45rem] last:border-b-0";
const GUTTER: &str = "flex items-center justify-end border-r border-[#1a1d21] py-[.3rem] \
    pr-[.7ch] pl-[.5rem] text-[11px] text-[#3f454d]";
const FIELD: &str = "m-0 w-full border-0 border-b border-transparent bg-transparent px-[.25rem] \
    py-[.32rem] font-[inherit] text-[13px] text-white focus:border-b-[#e2a340] \
    focus:bg-[#0b0e11] focus:outline-none";
const LINE_MESSAGE: &str = "col-start-2 col-span-2 m-0 pb-[.4rem] pl-[.25rem] text-[11px] \
    leading-[1.45] text-[#e2a340]";

#[component]
fn TechnologyLine(
    rows: RowList<TechnologyRow>,
    index: ReadSignal<usize>,
    row: TechnologyRow,
    rejection: SaveRejection,
    drag: DragState,
) -> impl IntoView {
    let message = move || rejection.message_for(RejectedLine::Technology(line_number(index.get())));

    view! {
        <div
            class=LINE
            class=("cursor-grabbing", move || {
                drag.is_dragging(index.get())
            })
            draggable="true"
            on:dragstart=move |event: DragEvent| drag.start(index.get_untracked(), &event)
            on:dragover=move |event: DragEvent| drag.over(index.get_untracked(), &event)
            on:drop=move |event: DragEvent| drag.drop(rows, index.get_untracked(), &event)
            on:dragend=move |_: DragEvent| drag.clear()
        >
            <span
                class="pointer-events-none absolute inset-x-0 top-[-1px] z-10 h-px bg-[#e2a340]"
                class=("hidden", move || !drag.is_drop_target(index.get()))
                aria-hidden="true"
            ></span>
            <span class=GUTTER>{move || position(index.get())}</span>
            <div class="grid min-w-0 py-[.15rem]">
                <label class="sr-only" for=move || field_id("technology", index.get())>
                    {move || format!("Technology {}", line_number(index.get()))}
                </label>
                <input
                    id=move || field_id("technology", index.get())
                    name=move || format!("technologies[{}]", index.get())
                    class=FIELD
                    class=("border-b-[#c2542f]", move || message().is_some())
                    value=move || row.value.get()
                    prop:value=move || row.value.get()
                    on:input=move |event| row.value.set(event_target_value(&event))
                    aria-invalid=move || message().is_some().then_some("true")
                    placeholder="language, framework, or tool"
                    spellcheck="false"
                />
            </div>
            <LineControls
                index
                total=Signal::derive(move || rows.len())
                subject=Signal::derive(move || format!("technology {}", line_number(index.get())))
                on_up=move || rows.move_up(index.get_untracked())
                on_down=move || rows.move_down(index.get_untracked())
                on_remove=move || rows.remove(index.get_untracked())
            />
            {move || message().map(|message| view! { <p class=LINE_MESSAGE>{message}</p> })}
        </div>
    }
}

#[component]
fn LinkLine(
    rows: RowList<LinkRow>,
    index: ReadSignal<usize>,
    row: LinkRow,
    rejection: SaveRejection,
    drag: DragState,
) -> impl IntoView {
    let label_message =
        move || rejection.message_for(RejectedLine::LinkLabel(line_number(index.get())));
    let href_message =
        move || rejection.message_for(RejectedLine::LinkUrl(line_number(index.get())));

    view! {
        <div
            class=LINE
            class=("cursor-grabbing", move || {
                drag.is_dragging(index.get())
            })
            draggable="true"
            on:dragstart=move |event: DragEvent| drag.start(index.get_untracked(), &event)
            on:dragover=move |event: DragEvent| drag.over(index.get_untracked(), &event)
            on:drop=move |event: DragEvent| drag.drop(rows, index.get_untracked(), &event)
            on:dragend=move |_: DragEvent| drag.clear()
        >
            <span
                class="pointer-events-none absolute inset-x-0 top-[-1px] z-10 h-px bg-[#e2a340]"
                class=("hidden", move || !drag.is_drop_target(index.get()))
                aria-hidden="true"
            ></span>
            <span class=GUTTER>{move || position(index.get())}</span>
            <div class="grid min-w-0 py-[.15rem]">
                <label class="sr-only" for=move || field_id("link-label", index.get())>
                    {move || format!("Link {} label", line_number(index.get()))}
                </label>
                <input
                    id=move || field_id("link-label", index.get())
                    name=move || format!("links[{}][label]", index.get())
                    class=FIELD
                    class=("border-b-[#c2542f]", move || label_message().is_some())
                    value=move || row.label.get()
                    prop:value=move || row.label.get()
                    on:input=move |event| row.label.set(event_target_value(&event))
                    aria-invalid=move || label_message().is_some().then_some("true")
                    placeholder="label, e.g. GitHub"
                    spellcheck="false"
                />
                <label class="sr-only" for=move || field_id("link-href", index.get())>
                    {move || format!("Link {} URL", line_number(index.get()))}
                </label>
                <input
                    id=move || field_id("link-href", index.get())
                    name=move || format!("links[{}][href]", index.get())
                    class=format!("{FIELD} text-[12px] text-[#a7aeb6]")
                    class=("border-b-[#c2542f]", move || href_message().is_some())
                    value=move || row.href.get()
                    prop:value=move || row.href.get()
                    on:input=move |event| row.href.set(event_target_value(&event))
                    aria-invalid=move || href_message().is_some().then_some("true")
                    inputmode="url"
                    placeholder="URL, e.g. https://github.com/..."
                    spellcheck="false"
                />
            </div>
            <LineControls
                index
                total=Signal::derive(move || rows.len())
                subject=Signal::derive(move || format!("link {}", line_number(index.get())))
                on_up=move || rows.move_up(index.get_untracked())
                on_down=move || rows.move_down(index.get_untracked())
                on_remove=move || rows.remove(index.get_untracked())
            />
            {move || {
                label_message()
                    .or_else(href_message)
                    .map(|message| view! { <p class=LINE_MESSAGE>{message}</p> })
            }}
        </div>
    }
}

/// The move and remove buttons a line carries. Each one names the line it acts
/// on, so the accessible name stays unique as positions change.
#[component]
fn LineControls<Up, Down, Remove>(
    index: ReadSignal<usize>,
    total: Signal<usize>,
    subject: Signal<String>,
    on_up: Up,
    on_down: Down,
    on_remove: Remove,
) -> impl IntoView
where
    Up: Fn() + 'static,
    Down: Fn() + 'static,
    Remove: Fn() + 'static,
{
    let first = move || index.get() == 0;
    let last = move || index.get().saturating_add(1) >= total.get();

    view! {
        <div class="flex items-center gap-[.1rem]">
            <LineButton
                label=Signal::derive(move || format!("Move {} up", subject.get()))
                disabled=Signal::derive(first)
                on_press=on_up
            >
                <ChevronUp size=14 />
            </LineButton>
            <LineButton
                label=Signal::derive(move || format!("Move {} down", subject.get()))
                disabled=Signal::derive(last)
                on_press=on_down
            >
                <ChevronDown size=14 />
            </LineButton>
            <LineButton
                label=Signal::derive(move || format!("Remove {}", subject.get()))
                disabled=Signal::derive(|| false)
                on_press=on_remove
            >
                <Trash2 size=14 />
            </LineButton>
        </div>
    }
}

#[component]
fn LineButton<Press>(
    label: Signal<String>,
    disabled: Signal<bool>,
    on_press: Press,
    children: Children,
) -> impl IntoView
where
    Press: Fn() + 'static,
{
    view! {
        <button
            type="button"
            class="inline-flex h-6 w-6 items-center justify-center border border-transparent bg-transparent p-0 text-[#767d87] enabled:cursor-pointer enabled:hover:border-[#2b3037] enabled:hover:text-white disabled:cursor-not-allowed disabled:text-[#2f343a]"
            disabled=move || disabled.get()
            aria-label=move || label.get()
            on:click=move |_| on_press()
        >
            {children()}
        </button>
    }
}

/// The ghost line that appends an empty row to a buffer.
#[component]
fn AddLine<Add>(label: &'static str, on_add: Add) -> impl IntoView
where
    Add: Fn() + 'static,
{
    view! {
        <button
            type="button"
            class="grid w-full cursor-pointer grid-cols-[3.4ch_minmax(0,1fr)] items-stretch gap-x-[.8ch] border-0 border-t border-dashed border-[#1e2126] bg-transparent pr-[.45rem] text-left font-[inherit] text-[12px] text-[#8b939d] hover:text-[#e2a340]"
            on:click=move |_: ev::MouseEvent| on_add()
        >
            <span class=GUTTER>"+"</span>
            <span class="inline-flex items-center gap-[.8ch] py-[.5rem]">{label}</span>
        </button>
    }
}

/// The one-based line the Owner sees for a zero-based row index.
const fn line_number(index: usize) -> usize {
    index.saturating_add(1)
}

/// The gutter's two-digit line marker.
fn position(index: usize) -> String {
    format!("{:02}", line_number(index))
}

/// A field identity that stays unique as rows move.
fn field_id(kind: &str, index: usize) -> String {
    format!("{kind}-{}", line_number(index))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ordering the buttons perform, checked without a browser or a
    /// reactive owner.
    fn lines() -> Vec<usize> {
        vec![1, 2, 3]
    }

    #[test]
    fn moving_up_swaps_with_the_line_above() {
        let mut rows = lines();
        move_up(&mut rows, 2);
        assert_eq!(rows, [1, 3, 2]);
    }

    #[test]
    fn moving_down_swaps_with_the_line_below() {
        let mut rows = lines();
        move_down(&mut rows, 0);
        assert_eq!(rows, [2, 1, 3]);
    }

    #[test]
    fn moving_past_either_end_changes_nothing() {
        let mut rows = lines();
        move_up(&mut rows, 0);
        move_down(&mut rows, 2);
        assert_eq!(rows, [1, 2, 3]);
    }

    #[test]
    fn removing_keeps_the_remaining_order() {
        let mut rows = lines();
        remove_row(&mut rows, 1);
        assert_eq!(rows, [1, 3]);
    }

    #[test]
    fn dropping_a_line_moves_it_before_the_target() {
        let mut rows = lines();
        move_to(&mut rows, 0, 2);
        assert_eq!(rows, [2, 1, 3]);

        move_to(&mut rows, 2, 0);
        assert_eq!(rows, [3, 2, 1]);
    }

    #[test]
    fn only_a_line_rejection_marks_a_line() {
        assert_eq!(
            RejectedLine::from_error(&AdminError::InvalidLinkUrl { position: 2 }),
            Some(RejectedLine::LinkUrl(2))
        );
        assert_eq!(RejectedLine::from_error(&AdminError::Save), None);
    }
}
