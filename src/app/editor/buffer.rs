//! The buffer: the entry list the editor moves around in.
//!
//! Entries carry stable ids rather than indices. A `usize` means "the third
//! row in the current collection", which goes wrong the moment content is
//! reordered; [`EntryId::Project`] means the same project wherever it sits.

use crate::{app::content::PortfolioContent, domain::ProjectSlug};

/// A section heading in the buffer list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionId {
    Profile,
    Work,
    Contact,
}

impl SectionId {
    /// The label rendered above the section's rows, and part of what search
    /// matches against.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Profile => "profile",
            Self::Work => "work",
            Self::Contact => "contact",
        }
    }
}

/// A stable entry identifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryId {
    Profile,
    Project(ProjectSlug),
    Contact,
}

impl EntryId {
    /// The addressable fragment, without the leading `#`.
    #[must_use]
    pub fn fragment(&self) -> String {
        match self {
            Self::Profile => "profile".to_owned(),
            Self::Project(slug) => format!("work-{slug}"),
            Self::Contact => "contact".to_owned(),
        }
    }
}

/// Where `Enter` sends the reader. External only, while there are no project
/// pages; the enum is the seam for `/work/:slug` when it returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Destination {
    /// A route inside the portfolio.
    Internal(String),
    /// A repository or demo, opened in a new tab.
    External(String),
}

/// The selected entry, and the section it sits in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub section: SectionId,
    pub entry: EntryId,
}

impl Default for Selection {
    /// The profile entry. The buffer always opens with it, so a page that
    /// cannot read the buffer still has somewhere real to start.
    fn default() -> Self {
        Self {
            section: SectionId::Profile,
            entry: EntryId::Profile,
        }
    }
}

/// One navigable line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BufferEntry {
    pub id: EntryId,
    pub section: SectionId,
    pub name: String,
    /// Where `Enter` goes. Profile and Contact have nowhere to go.
    pub destination: Option<Destination>,
    /// Lowercased name, section label, body and meta, joined. Built once at
    /// construction rather than per keystroke.
    haystack: String,
}

impl BufferEntry {
    fn new(
        id: EntryId,
        section: SectionId,
        name: &str,
        destination: Option<Destination>,
        text: &[&str],
    ) -> Self {
        let haystack = [name, section.label()]
            .into_iter()
            .chain(text.iter().copied())
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();

        Self {
            id,
            section,
            name: name.to_owned(),
            destination,
            haystack,
        }
    }

    #[must_use]
    pub fn selection(&self) -> Selection {
        Selection {
            section: self.section,
            entry: self.id.clone(),
        }
    }

    fn matches(&self, needle: &str) -> bool {
        self.haystack.contains(needle)
    }
}

/// A search result, and whether the scan passed the end of the buffer to
/// find it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchHit {
    pub selection: Selection,
    pub wrapped: bool,
}

/// The whole buffer, in the order it renders.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Buffer {
    entries: Vec<BufferEntry>,
}

impl Buffer {
    /// Flattens the portfolio into lines. Takes content by reference so the
    /// source can later be a database query.
    #[must_use]
    pub fn from_content(content: &PortfolioContent) -> Self {
        let profile = &content.profile;

        let mut profile_text = vec![profile.title.as_str(), profile.about.as_str()];
        profile_text.extend(profile.technologies.iter().map(String::as_str));
        let mut entries = vec![BufferEntry::new(
            EntryId::Profile,
            SectionId::Profile,
            &profile.name,
            None,
            &profile_text,
        )];

        entries.extend(content.projects.iter().map(|project| {
            let mut text = vec![project.summary.as_str()];
            text.extend(project.technologies.iter().map(String::as_str));
            BufferEntry::new(
                EntryId::Project(project.slug.clone()),
                SectionId::Work,
                &project.title,
                Some(Destination::Internal(project.path())),
                &text,
            )
        }));

        entries.push(BufferEntry::new(
            EntryId::Contact,
            SectionId::Contact,
            &content.contact.name,
            None,
            &[content.contact.body.as_str(), profile.email.as_str()],
        ));

        Self { entries }
    }

    #[must_use]
    pub fn entries(&self) -> &[BufferEntry] {
        &self.entries
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn get(&self, id: &EntryId) -> Option<&BufferEntry> {
        self.entries.iter().find(|entry| &entry.id == id)
    }

    /// The 1-based position, for the statusline counter and the number keys.
    #[must_use]
    pub fn number_of(&self, id: &EntryId) -> Option<usize> {
        self.index_of(id).map(|index| index + 1)
    }

    #[must_use]
    pub fn first(&self) -> Option<Selection> {
        self.entries.first().map(BufferEntry::selection)
    }

    #[must_use]
    pub fn last(&self) -> Option<Selection> {
        self.entries.last().map(BufferEntry::selection)
    }

    /// Next entry, clamped at the last.
    #[must_use]
    pub fn next(&self, from: &EntryId) -> Option<Selection> {
        let index = self.index_of(from)?;
        self.entries
            .get(index.saturating_add(1))
            .or_else(|| self.entries.get(index))
            .map(BufferEntry::selection)
    }

    /// Previous entry, clamped at the first.
    #[must_use]
    pub fn previous(&self, from: &EntryId) -> Option<Selection> {
        let index = self.index_of(from)?;
        self.entries
            .get(index.saturating_sub(1))
            .map(BufferEntry::selection)
    }

    /// First entry of the next section. `None` in the last section, which the
    /// reducer turns into staying put.
    #[must_use]
    pub fn next_section(&self, from: &EntryId) -> Option<Selection> {
        let current = self.get(from)?.section;
        self.entries
            .iter()
            .skip_while(|entry| entry.section != current)
            .find(|entry| entry.section != current)
            .map(BufferEntry::selection)
    }

    /// First entry of the previous section. `None` in the first section.
    #[must_use]
    pub fn previous_section(&self, from: &EntryId) -> Option<Selection> {
        let current = self.get(from)?.section;
        let previous = self
            .entries
            .iter()
            .take_while(|entry| entry.section != current)
            .last()?
            .section;
        self.first_of_section(previous)
    }

    /// The section's first entry. What `:work` and `:contact` select.
    #[must_use]
    pub fn first_of_section(&self, section: SectionId) -> Option<Selection> {
        self.entries
            .iter()
            .find(|entry| entry.section == section)
            .map(BufferEntry::selection)
    }

    /// Select by the number shown in the list, counting from one.
    #[must_use]
    pub fn by_number(&self, number: usize) -> Option<Selection> {
        self.entries
            .get(number.checked_sub(1)?)
            .map(BufferEntry::selection)
    }

    /// The entry a `#fragment` addresses, comparing against generated
    /// fragments so an unknown one cannot invent an entry.
    #[must_use]
    pub fn by_fragment(&self, fragment: &str) -> Option<Selection> {
        let fragment = fragment.trim_start_matches('#');
        self.entries
            .iter()
            .find(|entry| entry.id.fragment() == fragment)
            .map(BufferEntry::selection)
    }

    /// Searches forward from `from`, wrapping, matching case insensitively
    /// against name, section label, body and meta. The starting entry is
    /// checked last, so a search never reports the row already selected
    /// without having looked everywhere else first.
    #[must_use]
    pub fn search(&self, from: &EntryId, query: &str) -> Option<SearchHit> {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return None;
        }

        let start = self.index_of(from)?;
        let len = self.entries.len();

        (1..=len).find_map(|offset| {
            let index = (start + offset) % len;
            let entry = self.entries.get(index)?;
            entry.matches(&needle).then(|| SearchHit {
                selection: entry.selection(),
                wrapped: index <= start,
            })
        })
    }

    fn index_of(&self, id: &EntryId) -> Option<usize> {
        self.entries.iter().position(|entry| &entry.id == id)
    }
}
