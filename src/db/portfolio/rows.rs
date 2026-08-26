pub struct SiteRow {
    pub url: String,
    pub title: String,
    pub description: String,
    pub og_image: String,
}

pub struct ProfileRow {
    pub name: String,
    pub title: String,
    pub summary: String,
    pub about: String,
    pub email: String,
}

pub struct FocusRow {
    pub label: String,
    pub detail: String,
}

pub struct SocialRow {
    pub label: String,
    pub href: String,
    pub rel: String,
}

pub struct ContactRow {
    pub name: String,
    pub body: String,
}

pub struct ProjectRow {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub description_markdown: String,
}

pub struct ProjectItemRow {
    pub project_id: i64,
    pub item: String,
}

pub struct ProjectLinkRow {
    pub project_id: i64,
    pub label: String,
    pub href: String,
}
