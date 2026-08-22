use super::{
    browser::{focus_row, navigate, reveal_content},
    model::{Action, Entry, Notice, actions, entries},
};
use crate::app::{
    content::PortfolioContent,
    editor::{
        Buffer, EditorState, Effect, EntryId, KeyInput, Notification, Transition, reduce, select,
    },
};
use leptos::prelude::*;
use std::time::Duration;

/// How long a notification stays up. Timing lives here, in the adapter, never
/// in the reducer.
const NOTIFICATION_DURATION: Duration = Duration::from_millis(3_000);

#[derive(Clone, Copy)]
pub(super) struct HomeViewModel {
    pub(super) state: RwSignal<EditorState>,
    notice: RwSignal<Option<Notice>>,
    entries: StoredValue<Vec<Entry>>,
    actions: StoredValue<Vec<Action>>,
    buffer: StoredValue<Buffer>,
    issued: StoredValue<u64>,
    total: usize,
}

impl HomeViewModel {
    pub(super) fn new(content: &PortfolioContent) -> Self {
        let entries = entries(content);
        let buffer = Buffer::from_content(content);
        let start = buffer.first().unwrap_or_default();

        Self {
            state: RwSignal::new(EditorState::new(start)),
            notice: RwSignal::new(None),
            actions: StoredValue::new(actions(&entries)),
            entries: StoredValue::new(entries),
            total: buffer.len(),
            buffer: StoredValue::new(buffer),
            issued: StoredValue::new(0),
        }
    }

    pub(super) fn transition_for(self, input: KeyInput) -> Transition {
        let state = self.state.get_untracked();
        self.buffer
            .with_value(|buffer| reduce(&state, input, buffer))
    }

    pub(super) fn advance(self, transition: Transition) {
        self.state.set(transition.state);
        for effect in &transition.effects {
            self.apply(effect);
        }
    }

    pub(super) fn pick(self, entry: &EntryId) {
        let transition = self
            .buffer
            .with_value(|buffer| select(&self.state.get_untracked(), entry, buffer));
        self.advance(transition);
    }

    pub(super) fn current(self) -> Option<Entry> {
        let active = self.state.get().active.entry;
        self.entries
            .with_value(|entries| entries.iter().find(|entry| entry.id == active).cloned())
    }

    pub(super) fn next(self) -> Option<(EntryId, String)> {
        let active = self.state.get().active.entry;
        let next = self.buffer.with_value(|buffer| buffer.next(&active))?.entry;
        if next == active {
            return None;
        }
        let name = self.entries.with_value(|entries| {
            entries
                .iter()
                .find(|entry| entry.id == next)
                .map(|entry| entry.name.clone())
        })?;

        Some((next, name))
    }

    pub(super) fn number_of(self, entry: &EntryId) -> Option<usize> {
        self.buffer.with_value(|buffer| buffer.number_of(entry))
    }

    pub(super) fn position(self) -> usize {
        self.number_of(&self.state.get().active.entry)
            .unwrap_or_default()
    }

    pub(super) const fn total(self) -> usize {
        self.total
    }

    pub(super) fn actions(self) -> Vec<Action> {
        self.actions.get_value()
    }

    pub(super) fn notice(self) -> Option<Notice> {
        self.notice.get()
    }

    fn apply(self, effect: &Effect) {
        match effect {
            Effect::ScrollTo(entry) => {
                focus_row(entry);
                reveal_content();
            }
            Effect::Navigate(destination) => navigate(destination),
            Effect::Notify(notification) => self.notify(notification),
            Effect::Dismiss => self.notice.set(None),
            Effect::FocusPage => focus_row(&self.state.get_untracked().active.entry),
        }
    }

    fn notify(self, notification: &Notification) {
        self.issued.update_value(|count| *count += 1);
        let id = self.issued.get_value();

        self.notice.set(Some(Notice {
            id,
            message: notification.to_string(),
        }));

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
