#[cfg(test)]
use crate::app::editor::Buffer;
use crate::app::{
    content::{FocusArea, PortfolioContent, Profile, ProjectLink, SocialLink},
    editor::{EntryId, SectionId},
};

/// A link as it is rendered: visible label, destination, relationship.
pub(super) struct Link {
    pub(super) label: String,
    pub(super) href: String,
    pub(super) rel: String,
}

/// Where an entry's links come from. `Contact` leads with the address itself
/// so the mail entry shows something you can read rather than the word "Email".
#[derive(Clone)]
pub(super) enum Links {
    Social(Vec<SocialLink>),
    Project(Vec<ProjectLink>),
    Contact(Profile),
}

impl Links {
    pub(super) fn resolve(self) -> Vec<Link> {
        let social = |link: SocialLink| Link {
            label: link.label,
            href: link.href,
            rel: link.rel,
        };

        match self {
            Self::Social(links) => links.into_iter().map(social).collect(),
            Self::Project(links) => links
                .into_iter()
                .map(|link| Link {
                    label: link.label,
                    href: link.href,
                    rel: "noopener noreferrer".to_owned(),
                })
                .collect(),
            Self::Contact(profile) => {
                let address = Link {
                    label: profile.email.trim_start_matches("mailto:").to_owned(),
                    href: profile.email.clone(),
                    rel: "noopener noreferrer".to_owned(),
                };
                std::iter::once(address)
                    .chain(
                        profile
                            .links
                            .into_iter()
                            .filter(|link| link.label != "Email")
                            .map(social),
                    )
                    .collect()
            }
        }
    }
}

/// One item in the profile pane's action row.
#[derive(Clone)]
pub(super) struct Action {
    pub(super) label: String,
    pub(super) href: String,
    pub(super) target: Option<EntryId>,
    pub(super) download: Option<&'static str>,
}

/// One navigable line in the buffer, with everything the content pane renders.
#[derive(Clone)]
pub(super) struct Entry {
    pub(super) id: EntryId,
    pub(super) section: SectionId,
    pub(super) name: String,
    pub(super) lead: Option<String>,
    pub(super) body: String,
    pub(super) focus: Vec<FocusArea>,
    pub(super) meta: Vec<String>,
    pub(super) links: Links,
}

/// A notification with the id that owns its timer, so an older timer cannot
/// erase a newer message.
#[derive(Clone)]
pub(super) struct Notice {
    pub(super) id: u64,
    pub(super) message: String,
}

pub(super) type EntryGroup = (SectionId, Vec<(EntryId, String)>);

/// Flattens the portfolio into the buffer's line list, in the same order and
/// under the same ids as the editor's own [`Buffer`].
pub(super) fn entries(content: &PortfolioContent) -> Vec<Entry> {
    let profile = &content.profile;
    let mut entries = vec![Entry {
        id: EntryId::Profile,
        section: SectionId::Profile,
        name: profile.name.clone(),
        lead: Some(profile.title.clone()),
        body: profile.about.clone(),
        focus: profile.working_style.clone(),
        meta: profile.stack.clone(),
        links: Links::Social(profile.links.clone()),
    }];

    entries.extend(content.projects.iter().map(|project| Entry {
        id: EntryId::Project(project.name.clone()),
        section: SectionId::Work,
        name: project.name.clone(),
        lead: None,
        body: project.summary.clone(),
        focus: Vec::new(),
        meta: project.stack.clone(),
        links: Links::Project(project.links.clone()),
    }));

    entries.push(Entry {
        id: EntryId::Contact,
        section: SectionId::Contact,
        name: content.contact.name.clone(),
        lead: None,
        body: content.contact.body.clone(),
        focus: Vec::new(),
        meta: Vec::new(),
        links: Links::Contact(profile.clone()),
    });

    entries
}

/// The profile pane's explicit next steps, ordered by what a hiring reader
/// wants first.
pub(super) fn actions(entries: &[Entry]) -> Vec<Action> {
    let find = |section| {
        entries
            .iter()
            .find(|entry| entry.section == section)
            .map(|entry| entry.id.clone())
    };
    let go = |label: &str, entry: EntryId| Action {
        label: label.to_owned(),
        href: format!("#{}", entry.fragment()),
        target: Some(entry),
        download: None,
    };

    find(SectionId::Work)
        .map(|entry| go("View work", entry))
        .into_iter()
        .chain(std::iter::once(Action {
            label: "Download CV".to_owned(),
            href: "/cv.pdf".to_owned(),
            target: None,
            download: Some("kristofers-solo-cv.pdf"),
        }))
        .chain(find(SectionId::Contact).map(|entry| go("Contact", entry)))
        .collect()
}

/// Groups entry ids under their section, preserving order.
pub(super) fn group_by_section(entries: &[Entry]) -> Vec<EntryGroup> {
    let mut groups: Vec<EntryGroup> = Vec::new();

    for entry in entries {
        let row = (entry.id.clone(), entry.name.clone());
        match groups.last_mut() {
            Some((section, rows)) if *section == entry.section => rows.push(row),
            _ => groups.push((entry.section, vec![row])),
        }
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::content::portfolio_content;
    use claims::{assert_none, assert_some_eq};

    #[test]
    fn the_rendered_entries_match_the_editor_buffer() {
        let content = portfolio_content();
        let rendered = entries(&content)
            .into_iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>();
        let editor = Buffer::from_content(&content)
            .entries()
            .iter()
            .map(|entry| entry.id.clone())
            .collect::<Vec<_>>();

        assert_eq!(rendered, editor);
    }

    #[test]
    fn sections_group_in_order() {
        let sections = group_by_section(&entries(&portfolio_content()))
            .into_iter()
            .map(|(section, rows)| (section, rows.len()))
            .collect::<Vec<_>>();

        assert_eq!(
            sections,
            [
                (SectionId::Profile, 1),
                (SectionId::Work, 3),
                (SectionId::Contact, 1),
            ]
        );
    }

    #[test]
    fn the_action_row_points_at_work_the_cv_and_contact() {
        let actions = actions(&entries(&portfolio_content()));
        let hrefs = actions
            .iter()
            .map(|action| action.href.as_str())
            .collect::<Vec<_>>();

        assert_eq!(hrefs, ["#work-guenther", "/cv.pdf", "#contact"]);
        assert_some_eq!(
            actions[0].target.clone(),
            EntryId::Project("guenther".to_owned())
        );
        assert_none!(actions[1].target.clone(), "the CV leaves the page");
        assert_some_eq!(actions[2].target.clone(), EntryId::Contact);
    }
}
