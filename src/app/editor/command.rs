//! The command line parser.
//!
//! Vim accepts unambiguous command prefixes. The current commands differ at
//! their first character, making every non-empty prefix unambiguous.

use super::Notification;

/// A command the editor understands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Work,
    Contact,
    /// The entry to open. With no name, `:edit` reloads the current entry.
    Edit(Option<String>),
}

impl Command {
    /// Parses command text without the leading colon, which the view renders.
    ///
    /// Leading, trailing and repeated whitespace is collapsed first. The first
    /// word names the command and the rest is its argument, which only
    /// `:edit` takes.
    ///
    /// # Errors
    ///
    /// Returns the notification to show for input that names no command, or
    /// that hands an argument to one taking none.
    pub fn parse(input: &str) -> Result<Self, Notification> {
        let normalized = input.split_whitespace().collect::<Vec<_>>().join(" ");
        let (name, argument) = normalized
            .split_once(' ')
            .map_or((normalized.as_str(), None), |(name, argument)| {
                (name, Some(argument))
            });

        if abbreviates("edit", name) {
            return Ok(Self::Edit(argument.map(str::to_owned)));
        }

        let command = if abbreviates("help", name) {
            Self::Help
        } else if abbreviates("work", name) {
            Self::Work
        } else if abbreviates("contact", name) {
            Self::Contact
        } else {
            return Err(Notification::NotAnEditorCommand(normalized));
        };

        argument.map_or(Ok(command), |argument| {
            Err(Notification::TrailingCharacters(argument.to_owned()))
        })
    }
}

/// Whether `input` is a non-empty prefix of `name`.
fn abbreviates(name: &str, input: &str) -> bool {
    !input.is_empty() && name.starts_with(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err_eq, assert_ok_eq};
    use rstest::rstest;

    /// Every command name is reachable from its first letter on, so this also
    /// checks that no two names shadow each other.
    #[rstest]
    #[case("c", Command::Contact)]
    #[case("contact", Command::Contact)]
    #[case("e", Command::Edit(None))]
    #[case("edit", Command::Edit(None))]
    #[case("h", Command::Help)]
    #[case("hel", Command::Help)]
    #[case("help", Command::Help)]
    #[case("w", Command::Work)]
    #[case("work", Command::Work)]
    #[case("   work   ", Command::Work)]
    fn commands_parse_from_any_prefix(#[case] input: &str, #[case] expected: Command) {
        assert_ok_eq!(Command::parse(input), expected);
    }

    #[rstest]
    #[case("e traxor", "traxor")]
    #[case("edit   cipher workshop  ", "cipher workshop")]
    fn edit_carries_its_argument(#[case] input: &str, #[case] expected: &str) {
        assert_ok_eq!(
            Command::parse(input),
            Command::Edit(Some(expected.to_owned()))
        );
    }

    #[rstest]
    #[case::help("help me", "me")]
    #[case::work("w traxor", "traxor")]
    fn a_command_taking_no_argument_rejects_one(#[case] input: &str, #[case] rest: &str) {
        assert_err_eq!(
            Command::parse(input),
            Notification::TrailingCharacters(rest.to_owned())
        );
    }

    #[rstest]
    #[case::misspelled("wrok")]
    #[case::not_a_command_name("x")]
    #[case::nothing("")]
    fn anything_else_is_not_an_editor_command(#[case] input: &str) {
        assert_err_eq!(
            Command::parse(input),
            Notification::NotAnEditorCommand(input.trim().to_owned())
        );
    }
}
