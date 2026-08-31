//! The page that creates a Project.
//!
//! Creation and editing share the Markdown editor, the ordered collection
//! buffers, and the same validation, so the only field this page adds is the
//! slug. The slug is the Project's public route and its identity in the
//! editor, so it is set once here and never again.
//!
//! A rejected submission leaves the page mounted: every field keeps the value
//! the Owner typed, and the form states what to change.

use super::super::{
    admin_path_for_slug,
    components::{
        EditorLayout, FormMessage, MarkdownEditor, NEW_PROJECT_PATH, ProjectCollections,
        SaveRejection, TextInput,
    },
    error::AdminError,
    server_functions::CreateProject,
};
use crate::{
    app::content::Portfolio,
    domain::{ProjectLinks, ProjectTechnologies},
};
use leptos::{form::ActionForm, prelude::*};
use leptos_meta::Title;
use leptos_router::{NavigateOptions, hooks::use_navigate};

/// The creation form, and where a created Project sends the Owner.
#[component]
pub fn NewProjectPage() -> impl IntoView {
    let portfolio = expect_context::<Portfolio>();
    let action = ServerAction::<CreateProject>::new();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if let Some(Ok(content)) = action.value().get() {
            let created = content
                .projects
                .last()
                .map(|project| admin_path_for_slug(&project.slug));
            portfolio.replace(content);
            if let Some(editor) = created {
                navigate(&editor, NavigateOptions::default());
            }
        }
    });

    let error = Signal::derive(move || action.value().get().and_then(Result::err));
    let rejection = SaveRejection::new(error);

    view! {
        <Title text="New project" />
        <EditorLayout
            active=NEW_PROJECT_PATH.to_owned()
            heading="New project"
            breadcrumb="Admin / new project".to_owned()
            wide=true
        >
            <ActionForm action attr:class="mt-[2.2rem]">
                <TextInput label="Slug" name="slug" value=String::new() />
                <p class="mt-[.35rem] text-[11px] text-[#767d87]">
                    "The public path is /work/<slug>. It cannot change after the project is created."
                </p>
                <div class="mt-6 grid grid-cols-1 items-start gap-10 min-[961px]:grid-cols-[minmax(0,1fr)_360px]">
                    <div class="min-w-0">
                        <TextInput label="Title" name="title" value=String::new() />
                        <TextInput label="Summary" name="summary" value=String::new() />
                        <MarkdownEditor value=String::new() />
                    </div>
                    <ProjectCollections
                        technologies=ProjectTechnologies::default()
                        links=ProjectLinks::default()
                        rejection
                    />
                </div>
                {move || create_message(error, rejection)}
                <CreateButton />
                <p class="mt-[.8rem] text-[11px] text-[#767d87]">
                    "The project is published as soon as it is created, at the end of the project order."
                </p>
            </ActionForm>
        </EditorLayout>
    }
}

#[component]
fn CreateButton() -> impl IntoView {
    view! {
        <button
            class="mt-[1.6rem] w-auto cursor-pointer border border-[#30363d] bg-[#080a0d] px-[1.4rem] py-[.55rem] font-[inherit] text-[13px] text-white hover:border-[#e2a340]"
            type="submit"
        >
            "Create project"
        </button>
    }
}

/// What the form says when a submission was rejected. A rejected collection
/// line states its own reason beside the field, so the form only has to say
/// that nothing was created.
fn create_message(error: Signal<Option<AdminError>>, rejection: SaveRejection) -> impl IntoView {
    error.get().map(|error| {
        let message = if rejection.marks_a_line() {
            "Nothing was created. Fix the marked line and try again.".to_owned()
        } else {
            format!("{error} Nothing was created, and every field keeps what you typed.")
        };
        view! { <FormMessage>{message}</FormMessage> }
    })
}
