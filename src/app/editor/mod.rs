//! Pure editor state transitions.
//!
//! This module has no DOM, timers, or `web_sys`. The Leptos adapter converts
//! browser events into [`KeyInput`] values and applies the returned [`Effect`]s.
//!
//! `normal` and `line` reduce input by mode. `buffer`, `command`, and `key`
//! define the values they consume.

mod buffer;
mod command;
mod key;
mod line;
mod normal;
mod state;

pub use buffer::{
    Buffer, BufferEntry, Destination, EntryId, PageStep, SearchHit, SectionId, Selection,
};
pub use command::Command;
pub use key::{Key, KeyInput};
pub use state::{EditorState, Effect, Mode, Notification, Transition};

/// Returns the transition for one key press.
#[must_use]
pub fn reduce(state: &EditorState, input: KeyInput, buffer: &Buffer) -> Transition {
    if input.foreign() {
        return Transition::unchanged(state);
    }

    match &state.mode {
        Mode::Normal => normal::reduce(state, input, buffer),
        Mode::Command(text) => line::command(state, input, buffer, text),
        Mode::Search(query) => line::search(state, input, buffer, query),
    }
}

/// Shows or hides the portfolio navigation.
///
/// Both `Ctrl+B` and the toggle button call this function.
#[must_use]
pub fn toggle_sidebar(state: &EditorState) -> Transition {
    Transition::new(
        EditorState {
            sidebar: !state.sidebar,
            ..state.clone()
        },
        Vec::new(),
    )
}

/// Selects an entry by id.
///
/// Rows, action links, and URL fragments use this path. Selection also closes
/// an open command or search line.
#[must_use]
pub fn select(state: &EditorState, entry: &EntryId, buffer: &Buffer) -> Transition {
    let Some(active) = buffer.get(entry).map(BufferEntry::selection) else {
        return Transition::unchanged(state);
    };

    let scrolled = Effect::ScrollTo(active.entry.clone());
    Transition::new(
        EditorState {
            mode: Mode::Normal,
            active,
            ..state.clone()
        },
        vec![scrolled],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::content::portfolio_content;
    use claims::{assert_none, assert_some_eq};
    use rstest::rstest;

    fn buffer() -> Buffer {
        Buffer::from_content(&portfolio_content())
    }

    fn project(slug: &str) -> EntryId {
        EntryId::Project(crate::test_support::parse(slug))
    }

    /// Normal mode, sitting on `entry`.
    fn on(entry: EntryId) -> EditorState {
        let section = buffer()
            .get(&entry)
            .expect("the entry is in the buffer")
            .section;
        EditorState::new(Selection { section, entry })
    }

    fn press(state: &EditorState, key: Key) -> Transition {
        reduce(state, KeyInput::plain(key), &buffer())
    }

    /// Types `text` one character at a time into whatever line is open.
    fn typing(state: &EditorState, text: &str) -> EditorState {
        text.chars().fold(state.clone(), |state, character| {
            press(&state, Key::Char(character)).state
        })
    }

    /// Opens the line `key` starts and types `text` into it.
    fn line_with(entry: EntryId, key: char, text: &str) -> EditorState {
        typing(&press(&on(entry), Key::Char(key)).state, text)
    }

    #[rstest]
    #[case::down(Key::Char('j'), EntryId::Profile, project("guenther"))]
    #[case::down_arrow(Key::ArrowDown, EntryId::Profile, project("guenther"))]
    #[case::up(Key::Char('k'), project("traxor"), project("guenther"))]
    #[case::up_arrow(Key::ArrowUp, project("traxor"), project("guenther"))]
    #[case::first(Key::Char('g'), EntryId::Contact, EntryId::Profile)]
    #[case::last(Key::Char('G'), EntryId::Profile, EntryId::Contact)]
    #[case::next_section(Key::Char('J'), EntryId::Profile, project("guenther"))]
    #[case::next_section_mid(Key::Char('J'), project("traxor"), EntryId::Contact)]
    #[case::previous_section(Key::Char('K'), project("traxor"), EntryId::Profile)]
    #[case::previous_section_from_last(Key::Char('K'), EntryId::Contact, project("guenther"))]
    fn movement_selects_and_scrolls(
        #[case] key: Key,
        #[case] from: EntryId,
        #[case] expected: EntryId,
    ) {
        let transition = press(&on(from), key);
        assert_eq!(transition.effects, vec![Effect::ScrollTo(expected.clone())]);
        assert_eq!(transition.state.active.entry, expected);
    }

    #[rstest]
    #[case::past_the_end(Key::Char('j'), EntryId::Contact)]
    #[case::before_the_start(Key::Char('k'), EntryId::Profile)]
    #[case::past_the_last_section(Key::Char('J'), EntryId::Contact)]
    #[case::before_the_first_section(Key::Char('K'), EntryId::Profile)]
    fn movement_clamps_at_both_ends(#[case] key: Key, #[case] at: EntryId) {
        assert_eq!(press(&on(at.clone()), key).state.active.entry, at);
    }

    #[rstest]
    #[case('1', EntryId::Profile)]
    #[case('2', project("guenther"))]
    #[case('4', project("cipher-workshop"))]
    #[case('5', EntryId::Contact)]
    fn digits_select_by_visible_number(#[case] digit: char, #[case] expected: EntryId) {
        let transition = press(&on(EntryId::Profile), Key::Char(digit));
        assert_eq!(transition.state.active.entry, expected);
    }

    #[test]
    fn a_number_past_the_end_does_nothing() {
        let state = on(EntryId::Profile);
        let transition = press(&state, Key::Char('9'));

        assert_eq!(transition.state, state);
        assert_eq!(transition.effects, Vec::new());
    }

    #[rstest]
    #[case(Key::Char(':'), Mode::Command(String::new()))]
    #[case(Key::Char('/'), Mode::Search(String::new()))]
    fn a_line_opens_empty(#[case] key: Key, #[case] expected: Mode) {
        assert_eq!(press(&on(EntryId::Profile), key).state.mode, expected);
    }

    /// Captured deliberately: native find is gone, so search covers all text.
    #[test]
    fn ctrl_f_opens_search() {
        let transition = reduce(
            &on(EntryId::Profile),
            KeyInput::ctrl(Key::Char('f')),
            &buffer(),
        );

        assert_eq!(transition.state.mode, Mode::Search(String::new()));
    }

    #[rstest]
    #[case(':')]
    #[case('/')]
    fn escape_closes_the_line_and_keeps_the_selection(#[case] key: char) {
        let open = line_with(project("traxor"), key, "guenther");
        let transition = press(&open, Key::Escape);

        assert_eq!(transition.state.mode, Mode::Normal);
        assert_eq!(transition.state.active.entry, project("traxor"));
        assert_eq!(transition.effects, vec![Effect::FocusPage]);
    }

    #[test]
    fn backspace_deletes_one_character() {
        let state = press(&line_with(EntryId::Profile, ':', "work"), Key::Backspace).state;
        assert_eq!(state.mode, Mode::Command("wor".to_owned()));
    }

    /// Vim leaves the command line when you backspace past its start.
    #[test]
    fn backspace_on_an_empty_line_leaves_the_mode() {
        let state = press(&line_with(EntryId::Profile, ':', ""), Key::Backspace).state;
        assert_eq!(state.mode, Mode::Normal);
    }

    #[test]
    fn question_mark_opens_help_and_escape_closes_it() {
        let opened = press(&on(EntryId::Profile), Key::Char('?')).state;
        assert!(opened.help);

        let closed = press(&opened, Key::Escape);
        assert!(!closed.state.help);
        assert_eq!(closed.effects, vec![Effect::Dismiss]);
    }

    #[test]
    fn ctrl_b_collapses_the_sidebar_and_movement_leaves_it_collapsed() {
        let hidden = reduce(
            &on(EntryId::Profile),
            KeyInput::ctrl(Key::Char('b')),
            &buffer(),
        )
        .state;
        assert!(!hidden.sidebar);

        assert!(
            !press(&hidden, Key::Char('j')).state.sidebar,
            "movement must not undo a collapse the reader asked for"
        );

        let shown = reduce(&hidden, KeyInput::ctrl(Key::Char('b')), &buffer()).state;
        assert!(shown.sidebar);
    }

    /// The toggle button dispatches [`toggle_sidebar`] directly, so it has to
    /// produce exactly what the key press produces.
    #[test]
    fn the_toggle_button_and_ctrl_b_agree() {
        let state = on(EntryId::Profile);

        assert_eq!(
            toggle_sidebar(&state),
            reduce(&state, KeyInput::ctrl(Key::Char('b')), &buffer())
        );
    }

    #[test]
    fn enter_opens_a_project_detail() {
        let transition = press(&on(project("traxor")), Key::Enter);

        assert_eq!(
            transition.effects,
            vec![Effect::Navigate(Destination::Internal(
                "/work/traxor".to_owned()
            ))]
        );
    }

    #[rstest]
    #[case(EntryId::Profile)]
    #[case(EntryId::Contact)]
    fn enter_where_there_is_nothing_to_open_says_so(#[case] entry: EntryId) {
        let transition = press(&on(entry), Key::Enter);

        assert_eq!(
            transition.effects,
            vec![Effect::Notify(Notification::NothingToOpen)]
        );
    }

    #[rstest]
    #[case("work", project("guenther"))]
    #[case("c", EntryId::Contact)]
    fn a_command_selects_its_entry(#[case] input: &str, #[case] expected: EntryId) {
        let transition = press(&line_with(EntryId::Profile, ':', input), Key::Enter);

        assert_eq!(transition.state.mode, Mode::Normal);
        assert_eq!(transition.state.active.entry, expected);
        assert!(transition.effects.contains(&Effect::ScrollTo(expected)));
    }

    #[test]
    fn an_unknown_command_notifies_and_leaves_the_selection() {
        let transition = press(&line_with(project("traxor"), ':', "wrok"), Key::Enter);

        assert_eq!(transition.state.active.entry, project("traxor"));
        assert!(
            transition
                .effects
                .contains(&Effect::Notify(Notification::NotAnEditorCommand(
                    "wrok".to_owned()
                )))
        );
    }

    #[test]
    fn the_help_command_opens_the_panel() {
        let transition = press(&line_with(EntryId::Profile, ':', "help"), Key::Enter);
        assert!(transition.state.help);
    }

    #[rstest]
    #[case::name("e traxor", project("traxor"))]
    #[case::fragment("edit profile", EntryId::Profile)]
    #[case::part_of_a_name("e cipher", project("cipher-workshop"))]
    #[case::ignoring_case("e WRITE", EntryId::Contact)]
    fn edit_opens_the_entry_its_name_addresses(#[case] input: &str, #[case] expected: EntryId) {
        let transition = press(&line_with(project("guenther"), ':', input), Key::Enter);

        assert_eq!(transition.state.active.entry, expected);
        assert!(transition.effects.contains(&Effect::ScrollTo(expected)));
    }

    #[test]
    fn edit_without_a_name_rereads_the_current_entry() {
        let transition = press(&line_with(project("traxor"), ':', "e"), Key::Enter);

        assert_eq!(transition.state.active.entry, project("traxor"));
        assert!(
            transition
                .effects
                .contains(&Effect::ScrollTo(project("traxor")))
        );
    }

    #[test]
    fn edit_opens_an_arbitrary_route_when_no_entry_matches() {
        let transition = press(&line_with(EntryId::Profile, ':', "e admin"), Key::Enter);

        assert_eq!(transition.state.active.entry, EntryId::Profile);
        assert!(
            transition
                .effects
                .contains(&Effect::Navigate(Destination::Internal(
                    "/admin".to_owned()
                )))
        );
    }

    /// No incremental search: the jump happens on Enter, never while typing.
    #[test]
    fn typing_a_query_never_moves_the_selection() {
        let state = line_with(EntryId::Profile, '/', "transmission");

        assert_eq!(state.mode, Mode::Search("transmission".to_owned()));
        assert_eq!(state.active.entry, EntryId::Profile);
    }

    #[test]
    fn search_jumps_forward_to_the_match() {
        let transition = press(
            &line_with(EntryId::Profile, '/', "transmission"),
            Key::Enter,
        );

        assert_eq!(transition.state.active.entry, project("traxor"));
        assert!(
            !transition
                .effects
                .contains(&Effect::Notify(Notification::SearchWrapped)),
            "the match sits after the cursor, so nothing wrapped"
        );
    }

    /// The query matches the entry the search starts on, so the scan has to
    /// go all the way round to find it.
    #[test]
    fn search_wraps_and_reports_it() {
        let transition = press(&line_with(EntryId::Contact, '/', "guenther"), Key::Enter);

        assert_eq!(transition.state.active.entry, project("guenther"));
        assert!(
            transition
                .effects
                .contains(&Effect::Notify(Notification::SearchWrapped))
        );
    }

    #[test]
    fn search_matches_case_insensitively_across_body_and_meta() {
        let transition = press(&line_with(EntryId::Profile, '/', "RATATUI"), Key::Enter);
        assert_eq!(transition.state.active.entry, project("traxor"));
    }

    #[test]
    fn no_match_keeps_the_selection_and_reports_e486() {
        let transition = press(&line_with(project("traxor"), '/', "kubernetes"), Key::Enter);

        assert_eq!(transition.state.active.entry, project("traxor"));
        assert!(
            transition
                .effects
                .contains(&Effect::Notify(Notification::PatternNotFound(
                    "kubernetes".to_owned()
                )))
        );
    }

    #[rstest]
    #[case(EntryId::Profile, "profile")]
    #[case(project("guenther"), "work-guenther")]
    #[case(EntryId::Contact, "contact")]
    fn entries_are_addressable_by_fragment(#[case] entry: EntryId, #[case] fragment: &str) {
        assert_eq!(entry.fragment(), fragment);
        assert_some_eq!(
            buffer()
                .by_fragment(fragment)
                .map(|selection| selection.entry),
            entry
        );
    }

    #[test]
    fn selecting_by_id_closes_an_open_line() {
        let open = line_with(EntryId::Profile, ':', "wo");
        let transition = select(&open, &EntryId::Contact, &buffer());

        assert_eq!(transition.state.mode, Mode::Normal);
        assert_eq!(transition.state.active.entry, EntryId::Contact);
    }

    #[test]
    fn the_buffer_opens_on_the_default_selection() {
        assert_some_eq!(buffer().first(), Selection::default());
    }

    #[test]
    fn an_unknown_fragment_selects_nothing() {
        assert_none!(buffer().by_fragment("work-kubernetes"));
    }

    #[rstest]
    #[case::alt(true, false)]
    #[case::meta(false, true)]
    fn modifiers_the_editor_does_not_bind_are_left_to_the_browser(
        #[case] alt: bool,
        #[case] meta: bool,
    ) {
        let state = on(EntryId::Profile);
        let transition = reduce(
            &state,
            KeyInput {
                key: Key::Char('j'),
                ctrl: false,
                alt,
                meta,
            },
            &buffer(),
        );

        assert_eq!(transition.state, state);
        assert_eq!(transition.effects, Vec::new());
    }
}
