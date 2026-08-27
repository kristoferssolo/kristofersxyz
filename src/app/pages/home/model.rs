use crate::{
    app::{
        content::{FocusArea, PortfolioContent, Profile, SocialLink},
        editor::{Buffer, EntryId, SectionId},
    },
    domain::ProjectLink,
};

/// A link as it is rendered: visible label, destination, relationship.
pub struct Link {
    pub label: String,
    pub href: String,
    pub rel: String,
}

/// Where an entry's links come from. `Contact` leads with the address itself
/// so the mail entry shows something you can read rather than the word "Email".
#[derive(Clone)]
pub enum Links {
    Social(Vec<SocialLink>),
    Project(Vec<ProjectLink>),
    Contact(Profile),
}

impl Links {
    pub fn resolve(self) -> Vec<Link> {
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
pub struct Action {
    pub label: String,
    pub href: String,
    pub target: Option<EntryId>,
    pub download: Option<&'static str>,
}

/// One navigable line in the buffer, with everything the content pane renders.
#[derive(Clone)]
pub struct Entry {
    pub id: EntryId,
    pub section: SectionId,
    pub name: String,
    pub lead: Option<String>,
    pub body: String,
    pub focus: Vec<FocusArea>,
    pub meta: Vec<String>,
    pub links: Links,
}

/// Builds the rendered entries in the same order and under the same ids as the
/// editor's [`Buffer`].
pub fn entries(content: &PortfolioContent) -> Vec<Entry> {
    Buffer::from_content(content)
        .entries()
        .iter()
        .filter_map(|buffer_entry| {
            let id = buffer_entry.id.clone();
            let section = buffer_entry.section;
            let name = buffer_entry.name.clone();

            match &id {
                EntryId::Profile => Some(Entry {
                    id,
                    section,
                    name,
                    lead: Some(content.profile.title.clone()),
                    body: content.profile.about.clone(),
                    focus: content.profile.working_style.clone(),
                    meta: content.profile.technologies.clone(),
                    links: Links::Social(content.profile.links.clone()),
                }),
                EntryId::Project(slug) => content
                    .projects
                    .iter()
                    .find(|project| &project.slug == slug)
                    .map(|project| Entry {
                        id,
                        section,
                        name,
                        lead: None,
                        body: project.summary.clone(),
                        focus: Vec::new(),
                        meta: project.technologies.clone(),
                        links: Links::Project(project.links.clone()),
                    }),
                EntryId::Contact => Some(Entry {
                    id,
                    section,
                    name,
                    lead: None,
                    body: content.contact.body.clone(),
                    focus: Vec::new(),
                    meta: Vec::new(),
                    links: Links::Contact(content.profile.clone()),
                }),
            }
        })
        .collect()
}

/// The profile pane's explicit next steps, ordered by what a hiring reader
/// wants first.
pub fn actions(entries: &[Entry]) -> Vec<Action> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{app::content::portfolio_content, domain::ProjectSlug};
    use claims::{assert_none, assert_some_eq};

    fn project_slug(value: &str) -> ProjectSlug {
        crate::test_support::parse(value)
    }

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
    fn a_project_uses_its_slug_for_identity_and_title_for_display() {
        let mut content = portfolio_content();
        content.projects[0].slug = project_slug("stable-slug");
        content.projects[0].title = "Reader-facing title".to_owned();

        let rendered = entries(&content)
            .into_iter()
            .find(|entry| entry.id == EntryId::Project(project_slug("stable-slug")))
            .map(|entry| entry.name);

        assert_some_eq!(rendered, "Reader-facing title".to_owned());
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
            EntryId::Project(project_slug("guenther"))
        );
        assert_none!(actions[1].target.clone(), "the CV leaves the page");
        assert_some_eq!(actions[2].target.clone(), EntryId::Contact);
    }
}
