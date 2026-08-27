//! Browser-facing controller for the pure editor reducer.

use crate::app::{
    browser::{activates_a_control, edits_text, focus_row, navigate, navigate_to, reveal_content},
    content::PortfolioContent,
    editor::{
        Buffer, BufferEntry, EditorState, Effect, EntryId, Key, KeyInput, Mode, Transition, reduce,
        select, toggle_sidebar,
    },
    layout::{SidebarPreference, StatusBarState, StatusLocation},
};
use leptos::{prelude::*, web_sys::KeyboardEvent};
use std::time::Duration;

/// How long editor notifications remain visible.
const NOTIFICATION_DURATION: Duration = Duration::from_millis(3_000);

#[derive(Clone, Copy)]
enum SelectionBehavior {
    /// Profile and contact selections update the homepage in place.
    Homepage,
    /// Every selection follows its canonical route.
    Routes,
}

#[derive(Clone)]
struct Notice {
    id: u64,
    message: String,
}

/// One editor session shared by a page's keyboard input, status bar, sidebar,
/// help panel, and notifications.
#[derive(Clone, Copy)]
pub struct EditorController {
    state: RwSignal<EditorState>,
    buffer: StoredValue<Buffer>,
    notice: RwSignal<Option<Notice>>,
    issued: StoredValue<u64>,
    sidebar_preference: SidebarPreference,
    selection_behavior: SelectionBehavior,
    /// When set, buffer movement is inert: only the command line, search and
    /// help respond. Used on pages without a portfolio buffer of their own,
    /// such as the admin surface, so keys never navigate the reader away.
    restricted: bool,
}

impl EditorController {
    #[must_use]
    pub fn homepage(content: &PortfolioContent, active: &EntryId) -> Self {
        Self::new(content, active, SelectionBehavior::Homepage)
    }

    #[must_use]
    pub fn routes(content: &PortfolioContent, active: &EntryId) -> Self {
        Self::new(content, active, SelectionBehavior::Routes)
    }

    /// A session for pages without a buffer of their own. The status bar and
    /// command line work, but buffer movement is inert.
    #[must_use]
    pub fn restricted(content: &PortfolioContent, active: &EntryId) -> Self {
        Self {
            restricted: true,
            ..Self::new(content, active, SelectionBehavior::Routes)
        }
    }

    fn new(
        content: &PortfolioContent,
        active: &EntryId,
        selection_behavior: SelectionBehavior,
    ) -> Self {
        let buffer = Buffer::from_content(content);
        let selection = buffer
            .get(active)
            .map_or_else(Default::default, BufferEntry::selection);
        let sidebar_preference = use_context::<SidebarPreference>().unwrap_or_default();
        let state = EditorState {
            sidebar: sidebar_preference.open_untracked(),
            ..EditorState::new(selection)
        };

        Self {
            state: RwSignal::new(state),
            buffer: StoredValue::new(buffer),
            notice: RwSignal::new(None),
            issued: StoredValue::new(0),
            sidebar_preference,
            selection_behavior,
            restricted: false,
        }
    }

    #[must_use]
    pub fn active(self) -> EntryId {
        self.state.get().active.entry
    }

    #[must_use]
    pub fn mode(self) -> Mode {
        self.state.get().mode
    }

    #[must_use]
    pub fn help(self) -> bool {
        self.state.get().help
    }

    /// Builds the status bar signal for this editor session.
    #[must_use]
    pub fn status(
        self,
        filename: impl Into<String>,
        location: impl Fn() -> StatusLocation + Send + Sync + 'static,
    ) -> Signal<StatusBarState> {
        self.status_signal(filename.into(), location, false)
    }

    /// Builds a status bar signal that also displays page progress.
    #[must_use]
    pub fn status_with_progress(
        self,
        filename: impl Into<String>,
        location: impl Fn() -> StatusLocation + Send + Sync + 'static,
    ) -> Signal<StatusBarState> {
        self.status_signal(filename.into(), location, true)
    }

    fn status_signal(
        self,
        filename: String,
        location: impl Fn() -> StatusLocation + Send + Sync + 'static,
        progress: bool,
    ) -> Signal<StatusBarState> {
        Signal::derive(move || {
            let state = StatusBarState::from_editor_mode(self.mode(), filename.clone(), location())
                .with_help();
            if progress {
                state.with_progress()
            } else {
                state
            }
        })
    }

    #[must_use]
    pub fn sidebar(self) -> bool {
        self.state.get().sidebar
    }

    /// The toggle button's way into the reducer. Identical to `Ctrl+B`, so the
    /// two controls cannot disagree about the layout.
    pub fn toggle_sidebar(self) {
        self.advance(toggle_sidebar(&self.state.get_untracked()));
    }

    #[must_use]
    pub fn notice(self) -> Option<String> {
        self.notice.get().map(|notice| notice.message)
    }

    #[must_use]
    pub fn position(self) -> usize {
        let active = self.state.get().active.entry;
        self.number_of(&active).unwrap_or_default()
    }

    #[must_use]
    pub fn total(self) -> usize {
        self.buffer.with_value(Buffer::len)
    }

    #[must_use]
    pub fn number_of(self, entry: &EntryId) -> Option<usize> {
        self.buffer.with_value(|buffer| buffer.number_of(entry))
    }

    #[must_use]
    pub fn next(self) -> Option<EntryId> {
        let active = self.state.get().active.entry;
        self.buffer
            .with_value(|buffer| buffer.next(&active))
            .map(|selection| selection.entry)
            .filter(|next| next != &active)
    }

    pub fn pick(self, entry: &EntryId) {
        let transition = self
            .buffer
            .with_value(|buffer| select(&self.state.get_untracked(), entry, buffer));
        self.advance(transition);
    }

    pub fn pick_fragment(self, fragment: &str) {
        let entry = self.buffer.with_value(|buffer| {
            buffer
                .by_fragment(fragment)
                .map(|selection| selection.entry)
        });

        if let Some(entry) = entry {
            self.pick(&entry);
        }
    }

    /// Normalizes and dispatches one browser key event. Returns whether the
    /// editor handled it so the caller can preserve native browser shortcuts.
    pub fn handle_keydown(self, event: &KeyboardEvent) {
        if edits_text(event) {
            return;
        }

        let current = self.state.get_untracked();

        if matches!(current.mode, Mode::Normal) && activates_a_control(event) {
            return;
        }

        let input = KeyInput {
            key: Key::from_name(&event.key()),
            ctrl: event.ctrl_key(),
            alt: event.alt_key(),
            meta: event.meta_key(),
        };
        let transition = self
            .buffer
            .with_value(|buffer| reduce(&current, input, buffer));

        if transition.state == current && transition.effects.is_empty() {
            return;
        }

        if self.suppresses(&current, &transition) {
            return;
        }

        event.prevent_default();
        self.advance(transition);
    }

    /// On a restricted session, keeps buffer movement from taking effect. A key
    /// is only honored when it opens the command line or search, or toggles the
    /// help panel; everything else leaves the reader where they are.
    const fn suppresses(self, current: &EditorState, transition: &Transition) -> bool {
        if !self.restricted || !matches!(current.mode, Mode::Normal) {
            return false;
        }

        let opens_line = !matches!(transition.state.mode, Mode::Normal);
        let toggles_help = transition.state.help != current.help;
        !opens_line && !toggles_help
    }

    fn advance(self, transition: Transition) {
        if transition.state.sidebar != self.state.get_untracked().sidebar {
            self.sidebar_preference.set(transition.state.sidebar);
        }

        self.state.set(transition.state);
        for effect in &transition.effects {
            self.apply(effect);
        }
    }

    fn apply(self, effect: &Effect) {
        match effect {
            Effect::ScrollTo(entry) => match self.selection_behavior {
                SelectionBehavior::Homepage if !matches!(entry, EntryId::Project(_)) => {
                    focus_row(entry);
                    reveal_content();
                }
                SelectionBehavior::Homepage | SelectionBehavior::Routes => navigate_to(entry),
            },
            Effect::Navigate(destination) => navigate(destination),
            Effect::Notify(notification) => self.notify(notification.to_string()),
            Effect::Dismiss => self.notice.set(None),
            Effect::FocusPage => focus_row(&self.state.get_untracked().active.entry),
        }
    }

    fn notify(self, message: String) {
        self.issued
            .update_value(|count| *count = count.saturating_add(1));
        let id = self.issued.get_value();
        self.notice.set(Some(Notice { id, message }));

        set_timeout(
            move || {
                if self
                    .notice
                    .get_untracked()
                    .is_some_and(|notice| notice.id == id)
                {
                    self.notice.set(None);
                }
            },
            NOTIFICATION_DURATION,
        );
    }
}
