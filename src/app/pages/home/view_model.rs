use super::model::{Action, Entry, actions, entries};
use crate::app::{content::PortfolioContent, editor::EntryId, editor_controller::EditorController};
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub(super) struct HomeViewModel {
    editor: EditorController,
    entries: StoredValue<Vec<Entry>>,
    actions: StoredValue<Vec<Action>>,
}

impl HomeViewModel {
    pub(super) fn new(content: &PortfolioContent) -> Self {
        let entries = entries(content);

        Self {
            editor: EditorController::homepage(content, &EntryId::Profile),
            actions: StoredValue::new(actions(&entries)),
            entries: StoredValue::new(entries),
        }
    }

    pub(super) const fn editor(self) -> EditorController {
        self.editor
    }

    pub(super) fn pick(self, entry: &EntryId) {
        self.editor.pick(entry);
    }

    pub(super) fn pick_fragment(self, fragment: &str) {
        self.editor.pick_fragment(fragment);
    }

    pub(super) fn current(self) -> Option<Entry> {
        let active = self.editor.active();
        self.entries
            .with_value(|entries| entries.iter().find(|entry| entry.id == active).cloned())
    }

    pub(super) fn next(self) -> Option<(EntryId, String)> {
        let next = self.editor.next()?;
        let name = self.entries.with_value(|entries| {
            entries
                .iter()
                .find(|entry| entry.id == next)
                .map(|entry| entry.name.clone())
        })?;

        Some((next, name))
    }

    pub(super) fn number_of(self, entry: &EntryId) -> Option<usize> {
        self.editor.number_of(entry)
    }

    pub(super) fn position(self) -> usize {
        self.editor.position()
    }

    pub(super) fn total(self) -> usize {
        self.editor.total()
    }

    pub(super) fn actions(self) -> Vec<Action> {
        self.actions.get_value()
    }
}
