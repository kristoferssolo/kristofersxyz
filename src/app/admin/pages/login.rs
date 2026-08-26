use super::super::server_functions::Login;
use crate::app::content::Portfolio;
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
    let error = move || {
        login.value().get().and_then(Result::err).map(
            |error| view! { <p class="mt-[1.2rem] text-xs text-[#e2a340]">{error.to_string()}</p> },
        )
    };

    view! {
        <Title text="Sign in" />
        <div class="grid min-h-dvh grid-cols-1 bg-black font-mono text-[#d4d7db] min-[721px]:grid-cols-[360px_1fr]">
            <aside class="flex min-h-0 flex-col px-9 py-12 min-[721px]:border-r min-[721px]:border-[#1e2126]">
                <p class="text-[10px] tracking-[.24em] text-[#767d87] uppercase">"Admin"</p>
                <h1 class="mt-[.7rem] font-sans text-2xl font-semibold text-white">"Sign in"</h1>
                <p class="mt-[.6rem] text-[13px] leading-[1.6] text-[#8b939d]">"The editing surface for the portfolio. Owner access only."</p>
                <ActionForm action=login attr:class="mt-[2.2rem]">
                    <label class="mt-[1.3rem] block text-xs text-[#8b939d]">
                        "Username"
                        <input
                            class="mt-[.4rem] block w-full border border-[#2b3037] bg-[#0b0e11] px-[.65rem] py-2 font-[inherit] text-[13px] text-white focus:border-[#e2a340] focus:outline-none"
                            name="username"
                            autocomplete="username"
                        />
                    </label>
                    <label class="mt-[1.3rem] block text-xs text-[#8b939d]">
                        "Password"
                        <input
                            class="mt-[.4rem] block w-full border border-[#2b3037] bg-[#0b0e11] px-[.65rem] py-2 font-[inherit] text-[13px] text-white focus:border-[#e2a340] focus:outline-none"
                            type="password"
                            name="password"
                            autocomplete="current-password"
                        />
                    </label>
                    {error}
                    <button
                        class="mt-[1.6rem] w-full cursor-pointer border border-[#30363d] bg-[#080a0d] p-[.55rem] font-[inherit] text-[13px] text-white hover:border-[#e2a340]"
                        type="submit"
                    >
                        "Sign in"
                    </button>
                </ActionForm>
                <p class="mt-auto pt-8 text-[11px] text-[#767d87]">"kristofers.xyz"</p>
            </aside>
            <main class="relative hidden flex-col overflow-hidden px-[3.25rem] py-12 min-[721px]:flex">
                <span class="absolute top-6 right-8 text-[11px] tracking-[.18em] text-[#2b3037] uppercase">"~/admin"</span>
                <div class="grid grid-cols-2 gap-x-12">
                    <div>
                        <p class="mt-0 mb-[.8rem] text-[10px] tracking-[.2em] text-[#767d87] uppercase">"Session"</p>
                        <dl class="m-0 grid grid-cols-[13ch_1fr] gap-x-[2ch] gap-y-[.55rem] text-[13px] [&_dt]:text-[#8b939d] [&_dd]:m-0 [&_dd]:text-[#c3c9cf]">
                            <dt>"status"</dt><dd>"signed out"</dd>
                            <dt>"method"</dt><dd>"server-side"</dd>
                            <dt>"idle limit"</dt><dd>"1 hour"</dd>
                        </dl>
                        <p class="mt-8 mb-[.8rem] text-[10px] tracking-[.2em] text-[#767d87] uppercase">"Content"</p>
                        <dl class="m-0 grid grid-cols-[13ch_1fr] gap-x-[2ch] gap-y-[.55rem] text-[13px] [&_dt]:text-[#8b939d] [&_dd]:m-0 [&_dd]:text-[#c3c9cf] [&_dd_b]:font-medium [&_dd_b]:text-[#e2a340]">
                            <dt>"store"</dt><dd>"SQLite"</dd>
                            <dt>"pages"</dt><dd><b>{pages_count}</b></dd>
                            <dt>"projects"</dt><dd><b>{projects}</b></dd>
                        </dl>
                    </div>
                    <div>
                        <p class="mt-0 mb-[.8rem] text-[10px] tracking-[.2em] text-[#767d87] uppercase">"Pages"</p>
                        <div class="mt-[.2rem] text-[13px]">{pages}</div>
                    </div>
                </div>
                <div class="mt-auto font-sans text-[clamp(2rem,4vw,3.2rem)] leading-[.9] font-semibold tracking-[-.04em] text-[#0e1116]">"kristofers.xyz"</div>
            </main>
        </div>
    }
}
