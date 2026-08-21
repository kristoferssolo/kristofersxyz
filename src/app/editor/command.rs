//! The command line parser.
//!
//! Exact aliases only. Prefix matching waits until there are enough commands
//! to justify it.

use super::Notification;

/// A command the editor understands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Work,
    Contact,
}

impl Command {
    /// Parses command text without the leading colon, which the view renders.
    ///
    /// Leading, trailing and repeated whitespace between arguments is
    /// collapsed first. Whitespace inside an argument is left alone, for when
    /// commands start taking arguments.
    ///
    /// # Errors
    ///
    /// Returns the notification to show for input that is not a command.
    pub fn parse(input: &str) -> Result<Self, Notification> {
        let normalized = input.split_whitespace().collect::<Vec<_>>().join(" ");

        match normalized.as_str() {
            "help" => Ok(Self::Help),
            "w" | "work" => Ok(Self::Work),
            "c" | "contact" => Ok(Self::Contact),
            _ => Err(Notification::NotAnEditorCommand(normalized)),
        }
    }
}
