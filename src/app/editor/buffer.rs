//! Ordered portfolio entries used by editor navigation.
//!
//! Stable ids survive content reordering; numeric indices do not.

use crate::{
    app::content::PortfolioContent,
    domain::{Project, ProjectSlug, TechnologyName},
};

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

    /// The canonical location of this page in the portfolio.
    #[must_use]
    pub fn path(&self) -> String {
        match self {
            Self::Profile => "/".to_owned(),
            Self::Project(slug) => Project::path_for_slug(slug),
            Self::Contact => "/#contact".to_owned(),
        }
    }
}

/// A single move through the ordered portfolio pages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageStep {
    Next,
    Previous,
}

/// Where `Enter` sends the reader.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Destination {
    /// A route inside the portfolio.
    Internal(String),
    /// A repository or demo, opened in a new tab.
    External(String),
}

impl Destination {
    /// Interprets raw `:edit` input as a place to open. Input carrying a URL
    /// scheme opens externally; anything else is an internal route, with a
    /// leading slash supplied when missing so `admin` reaches `/admin`.
    #[must_use]
    pub fn route(input: &str) -> Self {
        let target = input.trim();
        if target.contains("://") {
            Self::External(target.to_owned())
        } else if target.starts_with('/') {
            Self::Internal(target.to_owned())
        } else {
            Self::Internal(format!("/{target}"))
        }
    }
}

/// The selected entry, and the section it sits in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub section: SectionId,
    pub entry: EntryId,
}

impl Default for Selection {
    /// Defaults to the profile, the first entry in every valid buffer.
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
    /// Lowercased searchable text, computed once when the buffer is built.
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

    /// The lowercase names that address this entry: its own name, and the
    /// fragment a URL uses for it.
    fn names(&self) -> [String; 2] {
        [self.name.to_lowercase(), self.id.fragment()]
    }
}

/// A search result, and whether the scan passed the end of the buffer to
/// find it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchHit {
    pub selection: Selection,
    pub wrapped: bool,
}

/// All entries in display order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Buffer {
    entries: Vec<BufferEntry>,
}

impl Buffer {
    /// Flattens portfolio content into navigable lines.
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
            text.extend(project.technologies.iter().map(TechnologyName::as_str));
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
        self.index_of(id).and_then(|index| index.checked_add(1))
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

    /// Moves once through the same page order rendered by the sidebar.
    #[must_use]
    pub fn step(&self, from: &EntryId, step: PageStep) -> Option<Selection> {
        match step {
            PageStep::Next => self.next(from),
            PageStep::Previous => self.previous(from),
        }
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

    /// The entry `name` addresses, for `:edit`. Matches case insensitively
    /// against names and URL fragments. Exact matches take precedence over
    /// substring matches.
    #[must_use]
    pub fn by_name(&self, name: &str) -> Option<Selection> {
        let needle = name.trim().to_lowercase();
        if needle.is_empty() {
            return None;
        }

        self.entries
            .iter()
            .find(|entry| entry.names().contains(&needle))
            .or_else(|| {
                self.entries
                    .iter()
                    .find(|entry| entry.names().iter().any(|name| name.contains(&needle)))
            })
            .map(BufferEntry::selection)
    }

    /// The entry a `#fragment` addresses, comparing against generated
    /// fragments. Unknown fragments return `None`.
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
    /// checked last, after every other entry.
    #[must_use]
    pub fn search(&self, from: &EntryId, query: &str) -> Option<SearchHit> {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return None;
        }

        let start = self.index_of(from)?;
        let len = self.entries.len();

        (1..=len).find_map(|offset| {
            let index = start
                .wrapping_add(offset)
                .checked_rem(len)
                .unwrap_or_default();
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

#[cfg(test)]
mod page_tests {
    use super::*;
    use crate::app::content::portfolio_content;
    use claims::assert_some_eq;
    use rstest::rstest;

    #[rstest]
    #[case::bare_word("admin", Destination::Internal("/admin".to_owned()))]
    #[case::absolute_path("/admin/profile", Destination::Internal("/admin/profile".to_owned()))]
    #[case::external("https://github.com/kristofers", Destination::External("https://github.com/kristofers".to_owned()))]
    fn edit_routes_resolve_unmatched_targets(#[case] input: &str, #[case] expected: Destination) {
        assert_eq!(Destination::route(input), expected);
    }

    #[test]
    fn pages_have_canonical_locations() {
        let buffer = Buffer::from_content(&portfolio_content());
        let paths = buffer
            .entries()
            .iter()
            .map(|entry| entry.id.path())
            .collect::<Vec<_>>();

        assert_eq!(
            paths,
            [
                "/",
                "/work/guenther",
                "/work/traxor",
                "/work/cipher-workshop",
                "/#contact",
            ]
        );
    }

    #[test]
    fn project_pages_share_the_sidebar_sequence() {
        let content = portfolio_content();
        let buffer = Buffer::from_content(&content);
        let guenther = EntryId::Project(content.projects[0].slug.clone());

        assert_some_eq!(
            buffer
                .step(&guenther, PageStep::Previous)
                .map(|selection| selection.entry),
            EntryId::Profile
        );
        assert_some_eq!(
            buffer
                .step(&guenther, PageStep::Next)
                .map(|selection| selection.entry.path()),
            "/work/traxor".to_owned()
        );
    }
}
