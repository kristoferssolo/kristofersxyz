use super::super::{
    components::{EditorLayout, MarkdownEditor, TextArea, TextInput},
    error::AdminError,
    server_functions::{SaveContact, SaveProfile, SaveProject, SaveSite},
};
use crate::app::{content::Portfolio, content::PortfolioContent};
use leptos::{form::ActionForm, prelude::*};
use leptos_meta::Title;
use leptos_router::{NavigateOptions, components::A, hooks::use_navigate, hooks::use_params_map};

/// The project editor selected by the route slug.
#[component]
pub fn ProjectEditorPage() -> impl IntoView {
    let portfolio = expect_context::<Portfolio>();
    let params = use_params_map();

    move || {
        let slug = params.read().get("slug").unwrap_or_default();
        portfolio
            .current()
            .projects
            .into_iter()
            .find(|project| project.slug.as_str() == slug)
            .map_or_else(
                || {
                    set_not_found();
                    view! { <MissingProject /> }.into_any()
                },
                |project| {
                    let action = ServerAction::<SaveProject>::new();
                    follow_save(action, portfolio);
                    let active = format!("/admin/project/{}", project.slug);
                    let breadcrumb = format!("Admin / {}", project.slug);
                    let error = action_error(action);

                    view! {
                        <Title text="Edit project" />
                        <EditorLayout active heading="Edit project" breadcrumb wide=true>
                            <ActionForm action attr:class="admin-form">
                                <input type="hidden" name="slug" value=project.slug.to_string() />
                                <TextInput label="Title" name="title" value=project.title />
                                <TextInput label="Summary" name="summary" value=project.summary />
                                <MarkdownEditor value=project.description.as_str().to_owned() />
                                {error}
                                <button class="admin-button" type="submit">"Save"</button>
                            </ActionForm>
                        </EditorLayout>
                    }
                    .into_any()
                },
            )
    }
}

/// The profile singleton editor.
#[component]
pub fn ProfileEditorPage() -> impl IntoView {
    let portfolio = expect_context::<Portfolio>();
    let profile = portfolio.current().profile;
    let action = ServerAction::<SaveProfile>::new();
    follow_save(action, portfolio);
    let error = action_error(action);

    view! {
        <Title text="Edit profile" />
        <EditorLayout active="/admin/profile".to_owned() heading="Edit profile" breadcrumb="Admin / profile".to_owned() wide=false>
            <ActionForm action attr:class="admin-form">
                <TextInput label="Name" name="name" value=profile.name />
                <TextInput label="Title" name="title" value=profile.title />
                <TextInput label="Summary" name="summary" value=profile.summary />
                <TextArea label="About" name="about" value=profile.about />
                <TextInput label="Email" name="email" value=profile.email />
                {error}
                <button class="admin-button" type="submit">"Save"</button>
            </ActionForm>
        </EditorLayout>
    }
}

/// The contact singleton editor.
#[component]
pub fn ContactEditorPage() -> impl IntoView {
    let portfolio = expect_context::<Portfolio>();
    let contact = portfolio.current().contact;
    let action = ServerAction::<SaveContact>::new();
    follow_save(action, portfolio);
    let error = action_error(action);

    view! {
        <Title text="Edit contact" />
        <EditorLayout active="/admin/contact".to_owned() heading="Edit contact" breadcrumb="Admin / contact".to_owned() wide=false>
            <ActionForm action attr:class="admin-form">
                <TextInput label="Name" name="name" value=contact.name />
                <TextArea label="Body" name="body" value=contact.body />
                {error}
                <button class="admin-button" type="submit">"Save"</button>
            </ActionForm>
        </EditorLayout>
    }
}

/// The site metadata singleton editor.
#[component]
pub fn SiteEditorPage() -> impl IntoView {
    let portfolio = expect_context::<Portfolio>();
    let site = portfolio.current().site;
    let action = ServerAction::<SaveSite>::new();
    follow_save(action, portfolio);
    let error = action_error(action);

    view! {
        <Title text="Edit site metadata" />
        <EditorLayout active="/admin/site".to_owned() heading="Edit site metadata" breadcrumb="Admin / site".to_owned() wide=false>
            <ActionForm action attr:class="admin-form">
                <TextInput label="URL" name="url" value=site.url />
                <TextInput label="Title" name="title" value=site.title />
                <TextInput label="Description" name="description" value=site.description />
                <TextInput label="OpenGraph image" name="og_image" value=site.og_image />
                {error}
                <button class="admin-button" type="submit">"Save"</button>
            </ActionForm>
        </EditorLayout>
    }
}

fn follow_save<ServerFn>(action: ServerAction<ServerFn>, portfolio: Portfolio)
where
    ServerFn: leptos::server_fn::ServerFn<Output = PortfolioContent, Error = AdminError>
        + Clone
        + Send
        + Sync
        + 'static,
{
    let navigate = use_navigate();
    Effect::new(move |_| {
        if let Some(Ok(content)) = action.value().get() {
            portfolio.replace(content);
            navigate("/admin", NavigateOptions::default());
        }
    });
}

fn action_error<ServerFn>(action: ServerAction<ServerFn>) -> impl IntoView
where
    ServerFn: leptos::server_fn::ServerFn<Error = AdminError> + Clone + Send + Sync + 'static,
    ServerFn::Output: Clone + Send + Sync + 'static,
{
    move || {
        action
            .value()
            .get()
            .and_then(Result::err)
            .map(|error| view! { <p class="err">{error.to_string()}</p> })
    }
}

#[component]
fn MissingProject() -> impl IntoView {
    view! {
        <Title text="No such project" />
        <main class="stage">
            <div class="wrap narrow">
                <p class="eyebrow">"Admin"</p>
                <h1 class="admin-heading">"No such project"</h1>
                <p class="lede">"There is no editable project at this path."</p>
                <A href="/admin">"Back to admin"</A>
            </div>
        </main>
    }
}

#[cfg(feature = "ssr")]
fn set_not_found() {
    expect_context::<leptos_axum::ResponseOptions>().set_status(axum::http::StatusCode::NOT_FOUND);
}

#[cfg(not(feature = "ssr"))]
fn set_not_found() {}
