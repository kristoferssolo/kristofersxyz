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
            Key::Char('f' | 'F') => enter_mode(state, Mode::Search(String::new())),
            Key::Char('b' | 'B') => super::toggle_sidebar(state),
            _ => Transition::unchanged(state),
        };
    }

    let select = |selection: Option<Selection>| {
        let Some(active) = selection else {
            return Transition::unchanged(state);
        };
        let scrolled = Effect::ScrollTo(active.entry.clone());
        Transition::new(
            EditorState {
                active,
                ..state.clone()
            },
            vec![scrolled],
        )
    };

    let current = &state.active.entry;

    match input.key {
        Key::Char('j') | Key::ArrowDown => select(buffer.next(current)),
        Key::Char('k') | Key::ArrowUp => select(buffer.previous(current)),
        Key::Char('J') => select(buffer.next_section(current)),
        Key::Char('K') => select(buffer.previous_section(current)),
        Key::Char('g') => select(buffer.first()),
        Key::Char('G') => select(buffer.last()),
        Key::Char(digit @ '1'..='9') => {
            let number = digit
                .to_digit(10)
                .and_then(|value| usize::try_from(value).ok());
            select(number.and_then(|number| buffer.by_number(number)))
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
