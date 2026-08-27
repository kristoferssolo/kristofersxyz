use super::super::{
    components::{FormButton, InfoList, InfoRow, TextInput, action_error},
    server_functions::Login,
};
use crate::app::{content::Portfolio, layout::CommandShell};
use leptos::{form::ActionForm, prelude::*};
use leptos_meta::Title;

/// The credentials form and a read-only summary of the portfolio.
#[component]
pub fn LoginPage() -> impl IntoView {
    let login = ServerAction::<Login>::new();
    let content = expect_context::<Portfolio>().current();
    let projects = content.projects.len();
    let names = std::iter::once(content.profile.name)
        .chain(content.projects.into_iter().map(|project| project.title))
        .chain(std::iter::once(content.contact.name))
        .collect::<Vec<_>>();
    let pages_count = names.len();
    let pages = names
        .into_iter()
        .enumerate()
        .map(|(index, name)| {
            view! {
                <p class="my-[.35rem] text-[#8b939d]">
                    <span class="mr-[1.5ch] text-[#6b7280]">{index.saturating_add(1)}</span>
                    {name}
                </p>
            }
        })
        .collect_view();
    let error = action_error(login);

    view! {
        <Title text="Sign in" />
        <CommandShell filename="login">
        <div class="grid h-full grid-cols-1 overflow-hidden bg-black font-mono text-[#d4d7db] min-[721px]:grid-cols-[360px_1fr]">
            <aside class="flex min-h-0 flex-col overflow-y-auto px-9 py-12 min-[721px]:border-r min-[721px]:border-[#1e2126]">
                <p class="text-[10px] tracking-[.24em] text-[#767d87] uppercase">"Admin"</p>
                <h1 class="mt-[.7rem] font-sans text-2xl font-semibold text-white">"Sign in"</h1>
                <p class="mt-[.6rem] text-[13px] leading-[1.6] text-[#8b939d]">"The editing surface for the portfolio. Owner access only."</p>
                <ActionForm action=login attr:class="mt-[2.2rem]">
                    <TextInput
                        label="Username"
                        name="username"
                        value=String::new()
                        autocomplete="username"
                    />
                    <TextInput
                        label="Password"
                        name="password"
                        value=String::new()
                        input_type="password"
                        autocomplete="current-password"
                    />
                    {error}
                    <FormButton label="Sign in" full_width=true />
                </ActionForm>
                <p class="mt-auto pt-8 text-[11px] text-[#767d87]">"kristofers.xyz"</p>
            </aside>
            <main class="relative hidden flex-col overflow-hidden px-[3.25rem] py-12 min-[721px]:flex">
                <span class="absolute top-6 right-8 text-[11px] tracking-[.18em] text-[#2b3037] uppercase">"~/admin"</span>
                <div class="grid grid-cols-2 gap-x-12">
                    <div>
                        <p class="mt-0 mb-[.8rem] text-[10px] tracking-[.2em] text-[#767d87] uppercase">"Session"</p>
                        <InfoList
                            rows=vec![
                                InfoRow { label: "status", value: "signed out".to_owned(), emphasized: false },
                                InfoRow { label: "method", value: "server-side".to_owned(), emphasized: false },
                                InfoRow { label: "idle limit", value: "1 hour".to_owned(), emphasized: false },
                            ]
                        />
                        <p class="mt-8 mb-[.8rem] text-[10px] tracking-[.2em] text-[#767d87] uppercase">"Content"</p>
                        <InfoList
                            rows=vec![
                                InfoRow { label: "store", value: "SQLite".to_owned(), emphasized: false },
                                InfoRow { label: "pages", value: pages_count.to_string(), emphasized: true },
                                InfoRow { label: "projects", value: projects.to_string(), emphasized: true },
                            ]
                        />
                    </div>
                    <div>
                        <p class="mt-0 mb-[.8rem] text-[10px] tracking-[.2em] text-[#767d87] uppercase">"Pages"</p>
                        <div class="mt-[.2rem] text-[13px]">{pages}</div>
                    </div>
                </div>
                <div class="mt-auto font-sans text-[clamp(2rem,4vw,3.2rem)] leading-[.9] font-semibold tracking-[-.04em] text-[#0e1116]">"kristofers.xyz"</div>
            </main>
        </div>
        </CommandShell>
    }
}
