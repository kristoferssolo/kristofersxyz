pub struct ProfileRow {
    pub name: String,
    pub title: String,
    pub summary: String,
    pub about: String,
    pub email: String,
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

pub struct ProjectScreenshotRow {
    pub project_id: i64,
    pub screenshot_id: String,
    pub media_type: String,
    pub width: i64,
    pub height: i64,
    pub alt_text: String,
    pub caption: Option<String>,
}
