use crate::{
    app::{
        content::Portfolio,
        editor::EntryId,
        editor_controller::EditorController,
        layout::{BlankPage, StatusBarState, StatusLocation},
        markdown,
    },
    domain::{Project, ProjectSlug},
};
use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_params_map};

#[component]
pub fn ProjectPage() -> impl IntoView {
    let portfolio = expect_context::<Portfolio>();
    let params = use_params_map();

    move || {
        let content = portfolio.current();
        let slug = params.read().get("slug").unwrap_or_default();
        content
            .projects
            .iter()
            .find(|project| project.slug.as_str() == slug)
            .cloned()
            .map_or_else(
                || view! { <MissingProject /> }.into_any(),
                |project| view! { <ProjectReader project /> }.into_any(),
            )
    }
}

#[component]
fn ProjectReader(project: Project) -> impl IntoView {
    let content = expect_context::<Portfolio>().current();
    let active_id = EntryId::Project(project.slug.clone());
    let editor = EditorController::routes(&content, &active_id);
    let description = markdown::render(&project.description);
    let repository = project.links.first().cloned();
    let filename = format!("work/{}.md", project.slug);
    let status = Signal::derive(move || {
        StatusBarState::from_editor_mode(
            editor.mode(),
            filename.clone(),
            StatusLocation::Page {
                current: editor.position(),
                total: editor.total(),
            },
        )
        .with_help()
    });

    view! {
        <BlankPage editor status>
            <article class="min-h-0 flex-1 overflow-y-auto px-5 py-9 sm:px-10 md:px-14 md:py-14 xl:px-20">
                <div class="mx-auto grid max-w-[1080px] gap-14 xl:grid-cols-[minmax(0,76ch)_220px]">
                    <div class="min-w-0">
                        <p class="text-[10px] tracking-[0.22em] text-[#59616a] uppercase">
                            {format!("~/work/{}", project.slug)}
                        </p>
                        <h1 class="mt-4 font-sans text-[clamp(2.5rem,7vw,5.5rem)] leading-[0.95] font-semibold tracking-[-0.045em] text-white">
                            {project.title.clone()}
                        </h1>
                        <p class="mt-6 max-w-[62ch] font-sans text-[17px] leading-[1.65] text-[#b8bec5] sm:text-[19px]">
                            {project.summary.clone()}
                        </p>
                        {repository.map(|link| {
                            view! {
                                <div class="mt-7 text-[13px]">
                                    <a
                                        href=link.href
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="text-white underline decoration-[#3c424a] underline-offset-[5px] hover:decoration-[#e2a340]"
                                    >
                                        {link.label}
                                        <span aria-hidden="true" class="ml-[1ch] text-[#e2a340]">
                                            "↗"
                                        </span>
                                    </a>
                                </div>
                            }
                        })}

                        <div
                            class="project-description mt-14"
                            inner_html=description
                        ></div>

                        <ProjectSequence projects=content.projects current=project.slug.clone() />
                    </div>

                    <aside class="border-t border-[#1e2126] pt-6 xl:border-t-0 xl:pt-1">
                        <p class="text-[10px] tracking-[0.2em] text-[#59616a] uppercase">
                            "Technologies"
                        </p>
                        <ul class="mt-3 space-y-1 text-[12px] text-[#aab2bb]">
                            {project
                                .technologies
                                .into_iter()
                                .map(|technology| view! { <li>{technology}</li> })
                                .collect_view()}
                        </ul>

                        <div class="mt-9 border-t border-[#1e2126] pt-5">
                            <p class="text-[10px] tracking-[0.2em] text-[#59616a] uppercase">
                                "Evidence"
                            </p>
                            <p class="mt-3 font-sans text-[13px] leading-[1.6] text-[#7f8892]">
                                "Description, system shape, implementation decisions, and source."
                            </p>
                        </div>
                    </aside>
                </div>
            </article>
        </BlankPage>
    }
}

#[component]
fn ProjectSequence(projects: Vec<Project>, current: ProjectSlug) -> impl IntoView {
    let (previous, next) = project_neighbours(&projects, &current);

    view! {
        <nav
            aria-label="Project sequence"
            class="mt-16 grid gap-6 border-t border-[#1e2126] pt-5 text-[12px] sm:grid-cols-2"
        >
            {previous.map(|project| {
                view! {
                    <A href=project.path() attr:class="group text-[#747d87] hover:text-white">
                        <span class="block text-[10px] tracking-[0.16em] text-[#4c525a] uppercase">
                            "Previous project"
                        </span>
                        <span class="mt-2 block group-hover:text-white">{project.title}</span>
                    </A>
                }
            })}
            {next.map(|project| {
                view! {
                    <A
                        href=project.path()
                        attr:class="group text-[#747d87] hover:text-white sm:text-right"
                    >
                        <span class="block text-[10px] tracking-[0.16em] text-[#4c525a] uppercase">
                            "Next project"
                        </span>
                        <span class="mt-2 block group-hover:text-white">{project.title}</span>
                    </A>
                }
            })}
        </nav>
    }
}

#[component]
fn MissingProject() -> impl IntoView {
    let content = expect_context::<Portfolio>().current();
    let editor = EditorController::routes(&content, &EntryId::Profile);
    let status = Signal::derive(move || {
        StatusBarState::from_editor_mode(
            editor.mode(),
            "[No Name]",
            StatusLocation::Cursor { line: 0, column: 0 },
        )
        .with_help()
    });

    view! {
        <BlankPage editor status>
            <section class="flex min-h-0 flex-1 items-center overflow-y-auto px-5 py-14 sm:px-10 md:px-14">
                <div class="max-w-[62ch]">
                    <p class="text-[11px] tracking-[0.24em] text-[#4c525a] uppercase">"E484"</p>
                    <h1 class="mt-3 font-sans text-[clamp(1.75rem,4.5vw,2.75rem)] leading-[1.1] font-semibold text-white">
                        "Can't open project"
                    </h1>
                    <p class="mt-5 font-sans text-[16px] leading-[1.7] text-[#aab2bb] sm:text-[17px]">
                        "There is no published project at this path."
                    </p>
                    <A href="/" attr:class="mt-6 inline-block text-[13px] text-white underline decoration-[#3c424a] underline-offset-[5px] hover:decoration-[#e2a340]">
                        "Back to profile"
                    </A>
                </div>
            </section>
        </BlankPage>
    }
}

fn project_neighbours(
    projects: &[Project],
    current: &ProjectSlug,
) -> (Option<Project>, Option<Project>) {
    let Some(index) = projects.iter().position(|project| &project.slug == current) else {
        return (None, None);
    };

    let previous = index
        .checked_sub(1)
        .and_then(|previous| projects.get(previous))
        .cloned();
    let next = index
        .checked_add(1)
        .and_then(|next| projects.get(next))
        .cloned();
    (previous, next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::content::portfolio_content;
    use claims::{assert_none, assert_some_eq};

    #[test]
    fn project_sequence_does_not_wrap() {
        let projects = portfolio_content().projects;
        let first = &projects[0].slug;
        let last = &projects[2].slug;

        let (before_first, after_first) = project_neighbours(&projects, first);
        assert_none!(before_first);
        assert_some_eq!(
            after_first.map(|project| project.title),
            "traxor".to_owned()
        );

        let (before_last, after_last) = project_neighbours(&projects, last);
        assert_some_eq!(
            before_last.map(|project| project.title),
            "traxor".to_owned()
        );
        assert_none!(after_last);
    }
}
