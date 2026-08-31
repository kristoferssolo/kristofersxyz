use super::super::{
    admin_path_for_slug,
    components::{
        EditorLayout, Eyebrow, FormMessage, MarkdownEditor, PageHeading, ProjectCollections,
        ProjectPositionStepper, SaveButton, SaveRejection, TextArea, TextInput, action_error,
    },
    error::AdminError,
    server_functions::{SaveContact, SaveProfile, SaveProject, SaveSite},
};
use crate::app::{
    content::Portfolio, content::PortfolioContent, layout::CommandShell, pages::set_not_found,
};
use leptos::{form::ActionForm, prelude::*};
use leptos_meta::Title;
use leptos_router::{NavigateOptions, components::A, hooks::use_navigate, hooks::use_params_map};

macro_rules! editor_form {
    ($action:expr, $($field:tt)*) => {{
        let error = action_error($action);
        view! {
            <ActionForm action=$action attr:class="mt-[2.2rem]">
                $($field)*
                {error}
                <SaveButton />
            </ActionForm>
        }
    }};
}

/// The project editor selected by the route slug.
#[component]
pub fn ProjectEditorPage() -> impl IntoView {
    let portfolio = expect_context::<Portfolio>();
    let params = use_params_map();

    // The editor reads a snapshot, so reordering the Projects around this one
    // leaves the open form and everything typed into it alone.
    move || {
        let slug = params.read().get("slug").unwrap_or_default();
        portfolio
            .snapshot()
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
                    let active = admin_path_for_slug(&project.slug);
                    let breadcrumb = format!("Admin / {}", project.slug);
                    let stepper = view! { <ProjectPositionStepper slug=project.slug.clone() /> };
                    let error = Signal::derive(move || action.value().get().and_then(Result::err));
                    let rejection = SaveRejection::new(error);
                    view! {
                        <Title text="Edit project" />
                        <EditorLayout
                            active
                            heading="Edit project"
                            breadcrumb
                            wide=true
                            aside=stepper.into_any()
                        >
                            <ActionForm action attr:class="mt-[2.2rem]">
                                <input type="hidden" name="slug" value=project.slug.to_string() />
                                <div class="grid grid-cols-1 items-start gap-10 min-[961px]:grid-cols-[minmax(0,1fr)_360px]">
                                    <div class="min-w-0">
                                        <TextInput label="Title" name="title" value=project.title />
                                        <TextInput
                                            label="Summary"
                                            name="summary"
                                            value=project.summary
                                        />
                                        <MarkdownEditor value=project
                                            .description
                                            .as_str()
                                            .to_owned() />
                                    </div>
                                    <ProjectCollections
                                        technologies=project.technologies
                                        links=project.links
                                        rejection
                                    />
                                </div>
                                {move || save_message(error, rejection)}
                                <SaveButton />
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
    let profile = portfolio.snapshot().profile;
    let action = ServerAction::<SaveProfile>::new();
    follow_save(action, portfolio);
    view! {
        <Title text="Edit profile" />
        <EditorLayout active="/admin/profile".to_owned() heading="Edit profile" breadcrumb="Admin / profile".to_owned() wide=false>
            {editor_form!(
                action,
                <TextInput label="Name" name="name" value=profile.name />
                <TextInput label="Title" name="title" value=profile.title />
                <TextInput label="Summary" name="summary" value=profile.summary />
                <TextArea label="About" name="about" value=profile.about />
                <TextInput label="Email" name="email" value=profile.email />
            )}
        </EditorLayout>
    }
}

/// The contact singleton editor.
#[component]
pub fn ContactEditorPage() -> impl IntoView {
    let portfolio = expect_context::<Portfolio>();
    let contact = portfolio.snapshot().contact;
    let action = ServerAction::<SaveContact>::new();
    follow_save(action, portfolio);
    view! {
        <Title text="Edit contact" />
        <EditorLayout active="/admin/contact".to_owned() heading="Edit contact" breadcrumb="Admin / contact".to_owned() wide=false>
            {editor_form!(
                action,
                <TextInput label="Name" name="name" value=contact.name />
                <TextArea label="Body" name="body" value=contact.body />
            )}
        </EditorLayout>
    }
}

/// The site metadata singleton editor.
#[component]
pub fn SiteEditorPage() -> impl IntoView {
    let portfolio = expect_context::<Portfolio>();
    let site = portfolio.snapshot().site;
    let action = ServerAction::<SaveSite>::new();
    follow_save(action, portfolio);
    view! {
        <Title text="Edit site metadata" />
        <EditorLayout active="/admin/site".to_owned() heading="Edit site metadata" breadcrumb="Admin / site".to_owned() wide=false>
            {editor_form!(
                action,
                <TextInput label="URL" name="url" value=site.url />
                <TextInput label="Title" name="title" value=site.title />
                <TextInput label="Description" name="description" value=site.description />
                <TextInput label="OpenGraph image" name="og_image" value=site.og_image />
            )}
        </EditorLayout>
    }
}

/// What the project editor says at the foot of the form. A rejected line
/// states its own reason beside the field, so the form only has to say that
/// nothing was written.
fn save_message(error: Signal<Option<AdminError>>, rejection: SaveRejection) -> impl IntoView {
    error.get().map(|error| {
        let message = if rejection.marks_a_line() {
            "Nothing was saved. Fix the marked line and save again.".to_owned()
        } else {
            error.to_string()
        };
        view! { <FormMessage>{message}</FormMessage> }
    })
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

#[component]
fn MissingProject() -> impl IntoView {
    view! {
        <Title text="No such project" />
        <CommandShell filename="admin/project">
        <main class="relative flex h-full flex-col overflow-x-hidden overflow-y-auto bg-black px-[3.25rem] py-12 font-mono text-[#d4d7db]">
            <div class="mx-auto w-full max-w-[760px]">
                <Eyebrow>"Admin"</Eyebrow>
                <PageHeading>"No such project"</PageHeading>
                <p class="mt-[.6rem] text-[13px] leading-[1.6] text-[#8b939d]">"There is no editable project at this path."</p>
                <A href="/admin">"Back to admin"</A>
            </div>
        </main>
        </CommandShell>
    }
}
