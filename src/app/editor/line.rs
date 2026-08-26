//! Command and search line editing. Enter runs a command or executes a search.

use super::{
    Buffer, BufferEntry, Command, Destination, EditorState, Effect, Key, KeyInput, Mode,
    Notification, SectionId, Transition,
};

/// Reduces one key press in the command line.
pub fn command(state: &EditorState, input: KeyInput, buffer: &Buffer, text: &str) -> Transition {
    line(state, input, buffer, text, Line::Command)
}

/// Reduces one key press in the search line.
pub fn search(state: &EditorState, input: KeyInput, buffer: &Buffer, query: &str) -> Transition {
    line(state, input, buffer, query, Line::Search)
}

/// The active line and its Enter behavior.
#[derive(Clone, Copy)]
enum Line {
    Command,
    Search,
}

impl Line {
    const fn mode(self, text: String) -> Mode {
        match self {
            Self::Command => Mode::Command(text),
            Self::Search => Mode::Search(text),
        }
    }
}

fn line(
    state: &EditorState,
    input: KeyInput,
    buffer: &Buffer,
    text: &str,
    which: Line,
) -> Transition {
    if input.ctrl {
        return Transition::unchanged(state);
    }

    let rewrite = |text: String| {
        Transition::new(
            EditorState {
                mode: which.mode(text),
                ..state.clone()
            },
            Vec::new(),
        )
    };

    match input.key {
        Key::Escape => cancel(state),
        Key::Backspace => {
            let mut next = text.to_owned();
            next.pop()
                .map_or_else(|| cancel(state), |_| rewrite(next.clone()))
        }
        Key::Char(character) => rewrite(format!("{text}{character}")),
        Key::Enter => match which {
            Line::Command => run(state, buffer, text),
            Line::Search => jump(state, buffer, text),
        },
        _ => Transition::unchanged(state),
    }
}

/// Returns to Normal mode without moving the selection.
fn cancel(state: &EditorState) -> Transition {
    Transition::new(
        EditorState {
            mode: Mode::Normal,
            ..state.clone()
        },
        vec![Effect::FocusPage],
    )
}

fn run(state: &EditorState, buffer: &Buffer, text: &str) -> Transition {
    let mut next = EditorState {
        mode: Mode::Normal,
        ..state.clone()
    };
    let mut effects = vec![Effect::FocusPage];

    let selected = match Command::parse(text) {
        Ok(Command::Help) => {
            next.help = true;
            None
        }
        Ok(Command::Work) => buffer.first_of_section(SectionId::Work),
        Ok(Command::Contact) => buffer.first_of_section(SectionId::Contact),
        Ok(Command::Edit(None)) => buffer.get(&state.active.entry).map(BufferEntry::selection),
        Ok(Command::Edit(Some(name))) => {
            if let found @ Some(_) = buffer.by_name(&name) {
                found
            } else {
                effects.push(Effect::Navigate(Destination::route(&name)));
                None
            }
        }
        Err(notification) => {
            effects.push(Effect::Notify(notification));
            None
        }
    };

    if let Some(active) = selected {
        effects.push(Effect::ScrollTo(active.entry.clone()));
        next.active = active;
    }

    Transition::new(next, effects)
}

fn jump(state: &EditorState, buffer: &Buffer, query: &str) -> Transition {
    if query.trim().is_empty() {
        return cancel(state);
    }

    let mut next = EditorState {
        mode: Mode::Normal,
        ..state.clone()
    };
    let mut effects = vec![Effect::FocusPage];

    match buffer.search(&state.active.entry, query) {
        Some(hit) => {
            effects.push(Effect::ScrollTo(hit.selection.entry.clone()));
            if hit.wrapped {
                effects.push(Effect::Notify(Notification::SearchWrapped));
            }
            next.active = hit.selection;
        }
        None => effects.push(Effect::Notify(Notification::PatternNotFound(
            query.trim().to_owned(),
        ))),
    }

    Transition::new(next, effects)
}
