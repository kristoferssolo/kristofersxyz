//! Browser-facing controller for the pure editor reducer.

use crate::app::{
    browser::{activates_a_control, focus_row, navigate, navigate_to, reveal_content},
    content::PortfolioContent,
    editor::{
        Buffer, BufferEntry, EditorState, Effect, EntryId, Key, KeyInput, Mode, Transition, reduce,
        select, toggle_sidebar,
    },
    layout::SidebarPreference,
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

        event.prevent_default();
        self.advance(transition);
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
