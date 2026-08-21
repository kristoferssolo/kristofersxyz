//! Normal-mode reduction: movement, section jumps, number selection,
//! opening an entry, and the keys that enter the command line, search or
//! help.

use super::{
    Buffer, EditorState, Effect, Key, KeyInput, Mode, Notification, Selection, Transition,
};

/// Reduces one key press in Normal mode.
pub(super) fn reduce(state: &EditorState, input: KeyInput, buffer: &Buffer) -> Transition {
    if input.ctrl {
        return match input.key {
            // Captured deliberately: because native find is gone, search has
            // to cover all text, which it does.
            Key::Char('f' | 'F') => enter_mode(state, Mode::Search(String::new())),
            Key::Char('b' | 'B') => Transition::new(
                EditorState {
                    buffers: !state.buffers,
                    ..state.clone()
                },
                Vec::new(),
            ),
            _ => Transition::unchanged(state),
        };
    }

    // `j` and `k` reopen a hidden buffer list, so moving never strands anyone
    // in a pane they cannot navigate out of.
    let select = |selection: Option<Selection>, reveal: bool| {
        let Some(active) = selection else {
            return Transition::unchanged(state);
        };
        let scrolled = Effect::ScrollTo(active.entry.clone());
        Transition::new(
            EditorState {
                active,
                buffers: state.buffers || reveal,
                ..state.clone()
            },
            vec![scrolled],
        )
    };

    let current = &state.active.entry;

    match input.key {
        Key::Char('j') | Key::ArrowDown => select(buffer.next(current), true),
        Key::Char('k') | Key::ArrowUp => select(buffer.previous(current), true),
        Key::Char('J') => select(buffer.next_section(current), false),
        Key::Char('K') => select(buffer.previous_section(current), false),
        Key::Char('g') => select(buffer.first(), false),
        Key::Char('G') => select(buffer.last(), false),
        Key::Char(digit @ '1'..='9') => {
            select(buffer.by_number(digit as usize - '0' as usize), false)
        }
        Key::Enter => open(state, buffer),
        Key::Char('/') => enter_mode(state, Mode::Search(String::new())),
        Key::Char(':') => enter_mode(state, Mode::Command(String::new())),
        Key::Char('?') => Transition::new(
            EditorState {
                help: true,
                ..state.clone()
            },
            Vec::new(),
        ),
        Key::Escape => Transition::new(
            EditorState {
                help: false,
                ..state.clone()
            },
            vec![Effect::Dismiss],
        ),
        _ => Transition::unchanged(state),
    }
}

/// Enters `mode`, leaving the selection and panels untouched.
fn enter_mode(state: &EditorState, mode: Mode) -> Transition {
    Transition::new(
        EditorState {
            mode,
            ..state.clone()
        },
        Vec::new(),
    )
}

/// `Enter` in Normal mode. Profile and Contact have nowhere to go.
fn open(state: &EditorState, buffer: &Buffer) -> Transition {
    let effect = buffer
        .get(&state.active.entry)
        .and_then(|entry| entry.destination.clone())
        .map_or_else(
            || Effect::Notify(Notification::NothingToOpen),
            Effect::Navigate,
        );

    Transition::new(state.clone(), vec![effect])
}
