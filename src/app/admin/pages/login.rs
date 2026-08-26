use super::super::{server_functions::Login, style::ADMIN_STYLE};
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
            view! { <p><span class="n">{index.saturating_add(1)}</span>{name}</p> }
        })
        .collect_view();
    let error = move || {
        login
            .value()
            .get()
            .and_then(Result::err)
            .map(|error| view! { <p class="err">{error.to_string()}</p> })
    };

    view! {
        <Title text="Sign in" />
        <style inner_html=ADMIN_STYLE></style>
        <div class="login">
            <aside class="admin-aside">
                <p class="eyebrow">"Admin"</p>
                <h1 class="admin-heading">"Sign in"</h1>
                <p class="lede">"The editing surface for the portfolio. Owner access only."</p>
                <ActionForm action=login attr:class="admin-form">
                    <label class="admin-label">
                        "Username"
                        <input class="admin-input" name="username" autocomplete="username" />
                    </label>
                    <label class="admin-label">
                        "Password"
                        <input class="admin-input" type="password" name="password" autocomplete="current-password" />
                    </label>
                    {error}
                    <button class="admin-button" type="submit">"Sign in"</button>
                </ActionForm>
                <p class="foot">"kristofers.xyz"</p>
            </aside>
            <main class="stage">
                <span class="tag">"~/admin"</span>
                <div class="cols">
                    <div>
                        <p class="grp">"Session"</p>
                        <dl class="admin-dl">
                            <dt>"status"</dt><dd>"signed out"</dd>
                            <dt>"method"</dt><dd>"server-side"</dd>
                            <dt>"idle limit"</dt><dd>"1 hour"</dd>
                        </dl>
                        <p class="grp">"Content"</p>
                        <dl class="admin-dl">
                            <dt>"store"</dt><dd>"SQLite"</dd>
                            <dt>"pages"</dt><dd><b>{pages_count}</b></dd>
                            <dt>"projects"</dt><dd><b>{projects}</b></dd>
                        </dl>
                    </div>
                    <div>
                        <p class="grp">"Pages"</p>
                        <div class="pages">{pages}</div>
                    </div>
                </div>
                <div class="mark">"kristofers.xyz"</div>
            </main>
        </div>
    }
}
