pub(super) struct SiteRow {
    pub(super) url: String,
    pub(super) title: String,
    pub(super) description: String,
    pub(super) og_image: String,
}

pub(super) struct ProfileRow {
    pub(super) name: String,
    pub(super) title: String,
    pub(super) summary: String,
    pub(super) about: String,
    pub(super) email: String,
}

pub(super) struct FocusRow {
    pub(super) label: String,
    pub(super) detail: String,
}

pub(super) struct SocialRow {
    pub(super) label: String,
    pub(super) href: String,
    pub(super) rel: String,
}

pub(super) struct ContactRow {
    pub(super) name: String,
    pub(super) body: String,
}

pub(super) struct ProjectRow {
    pub(super) id: i64,
    pub(super) slug: String,
    pub(super) title: String,
    pub(super) summary: String,
    pub(super) description_markdown: String,
}

pub(super) struct ProjectItemRow {
    pub(super) project_id: i64,
    pub(super) item: String,
}

pub(super) struct ProjectLinkRow {
    pub(super) project_id: i64,
    pub(super) label: String,
    pub(super) href: String,
}
