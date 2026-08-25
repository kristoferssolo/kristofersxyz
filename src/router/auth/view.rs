//! HTML for the admin surface: the login, admin landing, and project-edit
//! pages, plus the shared document shell and its inline styles.
//!
//! The pages carry their own styles rather than the portfolio's compiled
//! Tailwind, so this surface renders independently of that build. Everything
//! drawn from content reads the live portfolio, so counts and lists stay true
//! as content changes.

use crate::{
    app::content::{Contact, Profile, Site, server_content},
    domain::Project,
};
use std::fmt::Write as _;

/// The styles for the admin surface. Inlined so the pages do not depend on the
/// portfolio's Tailwind build.
const ADMIN_STYLE: &str = r#"
:root{color-scheme:dark}
*{box-sizing:border-box}
body{margin:0;min-height:100dvh;background:#000;color:#d4d7db;
  font-family:"IBM Plex Mono",ui-monospace,monospace;text-rendering:optimizeLegibility}
.login{display:grid;grid-template-columns:360px 1fr;min-height:100dvh}
aside{border-right:1px solid #1e2126;padding:3rem 2.25rem;display:flex;flex-direction:column;min-height:0}
.eyebrow{font-size:10px;letter-spacing:.24em;text-transform:uppercase;color:#4c525a}
h1{margin:.7rem 0 0;font-family:"IBM Plex Sans",sans-serif;font-weight:600;font-size:1.5rem;color:#fff}
.lede{margin:.6rem 0 0;font-size:13px;line-height:1.6;color:#8b939d}
form{margin-top:2.2rem}
label{display:block;margin-top:1.3rem;font-size:12px;color:#8b939d}
input{margin-top:.4rem;display:block;width:100%;background:#0b0e11;color:#fff;
  border:1px solid #2b3037;padding:.5rem .65rem;font:inherit;font-size:13px}
input:focus{outline:none;border-color:#e2a340}
textarea{margin-top:.4rem;display:block;width:100%;min-height:55vh;background:#0b0e11;color:#fff;
  border:1px solid #2b3037;padding:.6rem .7rem;font:inherit;font-size:13px;line-height:1.65;resize:vertical}
textarea:focus{outline:none;border-color:#e2a340}
.err{margin-top:1.2rem;font-size:12px;color:#e2a340}
button{margin-top:1.6rem;width:100%;background:#080a0d;color:#fff;border:1px solid #30363d;
  padding:.55rem;font:inherit;font-size:13px;cursor:pointer}
button:hover{border-color:#e2a340}
.foot{margin-top:auto;padding-top:2rem;font-size:11px;color:#4c525a}
.stage{position:relative;overflow:hidden;display:flex;flex-direction:column;padding:3rem 3.25rem}
.tag{position:absolute;top:1.5rem;right:2rem;font-size:11px;letter-spacing:.18em;
  text-transform:uppercase;color:#2b3037}
.cols{display:grid;grid-template-columns:1fr 1fr;gap:0 3rem}
.grp{font-size:10px;letter-spacing:.2em;text-transform:uppercase;color:#59616a;margin:2rem 0 .8rem}
.grp:first-child{margin-top:0}
dl{display:grid;grid-template-columns:13ch 1fr;gap:.55rem 2ch;margin:0;font-size:13px}
dt{color:#8b939d}
dd{margin:0;color:#c3c9cf}
dd b{color:#e2a340;font-weight:500}
.pages{margin:.2rem 0 0;font-size:13px}
.pages p{margin:.35rem 0;color:#8b939d}
.pages .n{color:#3c424a;margin-right:1.5ch}
.mark{margin-top:auto;font-family:"IBM Plex Sans",sans-serif;font-weight:600;
  letter-spacing:-.04em;line-height:.9;font-size:clamp(2rem,4vw,3.2rem);color:#0e1116}
.eyebrow a{color:inherit;text-decoration:none}
.eyebrow a:hover{color:#8b939d}
.dash{display:grid;grid-template-columns:320px 1fr;min-height:100dvh}
.nav{list-style:none;margin:1rem 0 0;padding:0}
.nav a{display:flex;align-items:center;gap:1.1ch;padding:.45rem 0;font-size:13px;
  color:#8b939d;text-decoration:none}
.nav a:hover{color:#e2a340}
.nav a:hover .ico{color:#e2a340}
.nav a[aria-current]{color:#fff}
.nav a[aria-current] .ico{color:#8b939d}
.editor{max-width:640px}
.editor form{margin-top:1.6rem}
.editor button{width:auto;padding:.55rem 1.4rem}
.bottom{margin-top:auto;padding-top:2.5rem}
.bottom button{width:auto;padding:.55rem 1.4rem}
.projects{list-style:none;margin:1.4rem 0 0;padding:0;max-width:720px}
.projects li{border-bottom:1px solid #1e2126}
.projects a{display:block;padding:1.25rem 0;text-decoration:none;color:inherit}
.row{display:flex;justify-content:space-between;align-items:baseline;gap:2ch}
.name{font-size:15px;color:#fff}
.projects a:hover .name{color:#e2a340}
.edit{display:inline-flex;align-items:center;color:#3c424a}
.edit .i{width:14px;height:14px;fill:none;stroke:currentColor;stroke-width:2;
  stroke-linecap:round;stroke-linejoin:round}
.projects a:hover .edit{color:#e2a340}
.name-row{display:flex;align-items:center;gap:1.1ch}
.ico{width:16px;height:16px;flex:none;fill:none;stroke:currentColor;stroke-width:1.75;
  stroke-linecap:round;stroke-linejoin:round;color:#59616a}
.projects a:hover .ico{color:#e2a340}
.sum{margin:.5rem 0 0;font-size:13px;line-height:1.55;color:#8b939d}
.meta{margin:.6rem 0 0;font-size:11px;letter-spacing:.04em;color:#4c525a}
.meta b{color:#8b939d;font-weight:400}
.meta .path{color:#3c424a}
:focus-visible{outline:2px solid #e2a340;outline-offset:2px}
@media (max-width:720px){.login{grid-template-columns:1fr}.stage{display:none}aside{border-right:none}}
"#;

fn document(title: &str, body: &str) -> String {
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>{title}</title>\
         <link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">\
         <link href=\"https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&\
         family=IBM+Plex+Sans:wght@500;600&display=swap\" rel=\"stylesheet\">\
         <style>{ADMIN_STYLE}</style></head><body>{body}</body></html>"
    )
}

/// The login page. The right pane is a read-only status readout drawn from the
/// live portfolio, so the counts and page list stay true as content changes.
pub(super) fn login_page(error: Option<&str>) -> String {
    let content = server_content();
    let projects = content.projects.len();

    let mut names = Vec::with_capacity(projects.saturating_add(2));
    names.push(content.profile.name.clone());
    names.extend(content.projects.iter().map(|project| project.title.clone()));
    names.push(content.contact.name.clone());

    let pages = names
        .iter()
        .enumerate()
        .fold(String::new(), |mut pages, (index, name)| {
            let _ = write!(
                pages,
                "<p><span class=\"n\">{}</span>{}</p>",
                index.saturating_add(1),
                escape(name)
            );
            pages
        });

    let error = error.map_or_else(String::new, |message| {
        format!("<p class=\"err\">{}</p>", escape(message))
    });

    let body = format!(
        "<div class=\"login\">\
           <aside>\
             <p class=\"eyebrow\">Admin</p>\
             <h1>Sign in</h1>\
             <p class=\"lede\">The editing surface for the portfolio. Owner access only.</p>\
             <form method=\"post\" action=\"/login\">\
               <label>Username<input name=\"username\" autocomplete=\"username\"></label>\
               <label>Password<input name=\"password\" type=\"password\" \
                 autocomplete=\"current-password\"></label>\
               {error}\
               <button type=\"submit\">Sign in</button>\
             </form>\
             <p class=\"foot\">kristofers.xyz</p>\
           </aside>\
           <div class=\"stage\">\
             <span class=\"tag\">~/admin</span>\
             <div class=\"cols\">\
               <div>\
                 <p class=\"grp\">Session</p>\
                 <dl><dt>status</dt><dd>signed out</dd><dt>method</dt><dd>server-side</dd>\
                   <dt>idle limit</dt><dd>1 hour</dd></dl>\
                 <p class=\"grp\">Content</p>\
                 <dl><dt>store</dt><dd>SQLite</dd><dt>pages</dt><dd><b>{pages_count}</b></dd>\
                   <dt>projects</dt><dd><b>{projects}</b></dd></dl>\
               </div>\
               <div>\
                 <p class=\"grp\">Pages</p>\
                 <div class=\"pages\">{pages}</div>\
               </div>\
             </div>\
             <div class=\"mark\">kristofers.xyz</div>\
           </div>\
         </div>",
        pages_count = names.len(),
    );
    document("Sign in", &body)
}

/// The row affordance: Lucide `chevron-right`, shown on every entry link.
const AFFORDANCE: &str = "<span class=\"edit\"><svg class=\"i\" viewBox=\"0 0 24 24\">\
     <path d=\"m9 18 6-6-6-6\"/></svg></span>";
/// The entry markers: Lucide `box`, `user`, `mail`, and `globe`.
const ICON_PROJECT: &str = "<svg class=\"ico\" viewBox=\"0 0 24 24\">\
     <path d=\"M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z\"/>\
     <path d=\"m3.3 7 8.7 5 8.7-5\"/><path d=\"M12 22V12\"/></svg>";
const ICON_PROFILE: &str = "<svg class=\"ico\" viewBox=\"0 0 24 24\">\
     <path d=\"M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2\"/><circle cx=\"12\" cy=\"7\" r=\"4\"/></svg>";
const ICON_CONTACT: &str = "<svg class=\"ico\" viewBox=\"0 0 24 24\">\
     <rect width=\"20\" height=\"16\" x=\"2\" y=\"4\" rx=\"2\"/>\
     <path d=\"m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7\"/></svg>";
const ICON_SITE: &str = "<svg class=\"ico\" viewBox=\"0 0 24 24\">\
     <circle cx=\"12\" cy=\"12\" r=\"10\"/>\
     <path d=\"M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20\"/><path d=\"M2 12h20\"/></svg>";

/// The admin landing page: a session and content readout beside every project
/// as a link to its edit form. `name` is the signed-in user.
pub(super) fn admin_page(name: &str) -> String {
    let content = server_content();

    let rows = content
        .projects
        .iter()
        .fold(String::new(), |mut rows, project| {
            let words = project.description.as_str().split_whitespace().count();
            let links = project.links.len();
            let _ = write!(
                rows,
                "<li><a href=\"/admin/project/{slug}\">\
                   <div class=\"row\">\
                     <span class=\"name-row\">{ICON_PROJECT}<span class=\"name\">{title}</span></span>\
                     {AFFORDANCE}</div>\
                   <p class=\"sum\">{summary}</p>\
                   <p class=\"meta\"><b>{techs}</b> tech &middot; <b>{links}</b> {link_label} \
                     &middot; <b>{words}</b> words &middot; \
                     <span class=\"path\">{path}</span></p>\
                 </a></li>",
                slug = escape(project.slug.as_str()),
                title = escape(&project.title),
                summary = escape(&project.summary),
                techs = project.technologies.len(),
                link_label = if links == 1 { "link" } else { "links" },
                path = escape(&project.path()),
            );
            rows
        });

    let projects = content.projects.len();
    let pages = projects.saturating_add(2);

    let body = format!(
        "<div class=\"dash\">\
           <aside>\
             <p class=\"eyebrow\">Admin</p>\
             <h1>Signed in</h1>\
             <p class=\"lede\">Owner session. Pick a project to edit its description.</p>\
             <p class=\"grp\">Session</p>\
             <dl><dt>status</dt><dd>active</dd><dt>as</dt><dd>{name}</dd>\
               <dt>idle limit</dt><dd>1 hour</dd></dl>\
             <p class=\"grp\">Content</p>\
             <dl><dt>store</dt><dd>SQLite</dd><dt>projects</dt><dd><b>{projects}</b></dd>\
               <dt>pages</dt><dd><b>{pages}</b></dd></dl>\
             <div class=\"bottom\">\
               <form method=\"post\" action=\"/logout\">\
                 <button type=\"submit\">Sign out</button>\
               </form>\
               <p class=\"foot\">kristofers.xyz</p>\
             </div>\
           </aside>\
           <div class=\"stage\">\
             <p class=\"eyebrow\">Projects</p>\
             <ul class=\"projects\">{rows}</ul>\
             <p class=\"eyebrow\">Site</p>\
             <ul class=\"projects\">\
               <li><a href=\"/admin/profile\">\
                 <div class=\"row\">\
                   <span class=\"name-row\">{ICON_PROFILE}<span class=\"name\">Profile</span></span>\
                   {AFFORDANCE}</div>\
                 <p class=\"meta\">name, title, summary, about, email</p></a></li>\
               <li><a href=\"/admin/contact\">\
                 <div class=\"row\">\
                   <span class=\"name-row\">{ICON_CONTACT}<span class=\"name\">Contact</span></span>\
                   {AFFORDANCE}</div>\
                 <p class=\"meta\">name, body</p></a></li>\
               <li><a href=\"/admin/site\">\
                 <div class=\"row\">\
                   <span class=\"name-row\">{ICON_SITE}<span class=\"name\">Site metadata</span></span>\
                   {AFFORDANCE}</div>\
                 <p class=\"meta\">url, title, description, OpenGraph image</p></a></li>\
             </ul>\
           </div>\
         </div>",
        name = escape(name),
    );
    document("Admin", &body)
}

/// The edit sidebar: every editable entry as a link, with `active` (an href)
/// marked, so a switch is one click away while editing. Drawn from the live
/// portfolio, so the project list tracks content changes.
fn nav_aside(active: &str) -> String {
    let content = server_content();

    let link = |href: &str, label: &str, icon: &str| {
        let current = if href == active {
            " aria-current=\"page\""
        } else {
            ""
        };
        format!(
            "<li><a href=\"{href}\"{current}>{icon}<span>{label}</span></a></li>",
            href = escape(href),
            label = escape(label),
        )
    };

    let projects = content
        .projects
        .iter()
        .map(|project| link(&project_href(project), &project.title, ICON_PROJECT))
        .collect::<String>();

    format!(
        "<aside>\
           <p class=\"eyebrow\"><a href=\"/admin\">Admin</a></p>\
           <h1>Edit</h1>\
           <p class=\"grp\">Projects</p>\
           <ul class=\"nav\">{projects}</ul>\
           <p class=\"grp\">Site</p>\
           <ul class=\"nav\">{singletons}</ul>\
           <div class=\"bottom\">\
             <form method=\"post\" action=\"/logout\">\
               <button type=\"submit\">Sign out</button>\
             </form>\
             <p class=\"foot\">kristofers.xyz</p>\
           </div>\
         </aside>",
        singletons = [
            link("/admin/profile", "Profile", ICON_PROFILE),
            link("/admin/contact", "Contact", ICON_CONTACT),
            link("/admin/site", "Site metadata", ICON_SITE),
        ]
        .concat(),
    )
}

/// Wraps an edit form beside the entry sidebar, so the other entries stay one
/// click away. `active` is the current page's href, `breadcrumb` is trusted
/// markup, and `form` is the pre-rendered form.
fn edit_shell(active: &str, title: &str, breadcrumb: &str, form: &str) -> String {
    let body = format!(
        "<div class=\"dash\">{aside}\
           <div class=\"stage\">\
             <p class=\"eyebrow\">{breadcrumb}</p>\
             <h1>{title}</h1>\
             <div class=\"editor\">{form}</div>\
           </div>\
         </div>",
        aside = nav_aside(active),
        title = escape(title),
    );
    document(title, &body)
}

/// A project's edit form, prefilled with its current fields. The slug is the
/// route identity, so it is shown but not editable.
pub(super) fn project_page(project: &Project) -> String {
    let slug = escape(project.slug.as_str());
    let form = format!(
        "<form method=\"post\" action=\"/admin/project/{slug}\">\
           {title}{summary}{description}\
           <button type=\"submit\">Save</button>\
         </form>",
        title = field("Title", "title", &project.title),
        summary = field("Summary", "summary", &project.summary),
        description = area("Description (Markdown)", "markdown", project.description.as_str()),
    );
    let breadcrumb = format!("<a href=\"/admin\">Admin</a> / {slug}");
    edit_shell(&project_href(project), "Edit project", &breadcrumb, &form)
}

/// The admin edit route for a project.
fn project_href(project: &Project) -> String {
    format!("/admin/project/{}", project.slug.as_str())
}

/// The profile edit form, prefilled with its current scalar fields.
pub(super) fn profile_page(profile: &Profile) -> String {
    let form = format!(
        "<form method=\"post\" action=\"/admin/profile\">\
           {name}{title}{summary}{about}{email}\
           <button type=\"submit\">Save</button>\
         </form>",
        name = field("Name", "name", &profile.name),
        title = field("Title", "title", &profile.title),
        summary = field("Summary", "summary", &profile.summary),
        about = area("About", "about", &profile.about),
        email = field("Email", "email", &profile.email),
    );
    edit_shell(
        "/admin/profile",
        "Edit profile",
        "<a href=\"/admin\">Admin</a> / profile",
        &form,
    )
}

/// The contact edit form, prefilled with its current fields.
pub(super) fn contact_page(contact: &Contact) -> String {
    let form = format!(
        "<form method=\"post\" action=\"/admin/contact\">\
           {name}{body_area}\
           <button type=\"submit\">Save</button>\
         </form>",
        name = field("Name", "name", &contact.name),
        body_area = area("Body", "body", &contact.body),
    );
    edit_shell(
        "/admin/contact",
        "Edit contact",
        "<a href=\"/admin\">Admin</a> / contact",
        &form,
    )
}

/// The site metadata edit form, prefilled with its current fields.
pub(super) fn site_page(site: &Site) -> String {
    let form = format!(
        "<form method=\"post\" action=\"/admin/site\">\
           {url}{title}{description}{og_image}\
           <button type=\"submit\">Save</button>\
         </form>",
        url = field("URL", "url", &site.url),
        title = field("Title", "title", &site.title),
        description = field("Description", "description", &site.description),
        og_image = field("OpenGraph image", "og_image", &site.og_image),
    );
    edit_shell(
        "/admin/site",
        "Edit site metadata",
        "<a href=\"/admin\">Admin</a> / site",
        &form,
    )
}

/// A labeled single-line text input carrying its current value.
fn field(label: &str, name: &str, value: &str) -> String {
    format!(
        "<label>{label}<input name=\"{name}\" value=\"{value}\"></label>",
        label = escape(label),
        value = escape(value),
    )
}

/// A labeled multi-line text area carrying its current value.
fn area(label: &str, name: &str, value: &str) -> String {
    format!(
        "<label>{label}\
           <textarea name=\"{name}\" spellcheck=\"false\">{value}</textarea>\
         </label>",
        label = escape(label),
        value = escape(value),
    )
}

/// Escapes the handful of characters that would otherwise break out of the
/// surrounding HTML text. Content is author-supplied, but escaping keeps a
/// title with a `<` or `&` from corrupting the markup.
fn escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
