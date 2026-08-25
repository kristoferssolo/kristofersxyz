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
