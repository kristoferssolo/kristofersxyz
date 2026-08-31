use super::super::{
    components::provide_project_order,
    server_functions::{SessionUser, current_user},
};
use crate::app::content::Portfolio;
use leptos::prelude::*;
use leptos_router::components::{Outlet, Redirect};

/// Resolves the session before any nested admin route renders.
#[component]
pub fn AuthenticatedAdmin() -> impl IntoView {
    let user = Resource::new_blocking(|| (), |()| current_user());

    view! {
        <Suspense>
            {move || Suspend::new(async move {
                match user.await {
                    Ok(Some(user)) => view! { <AuthenticatedOutlet user /> }.into_any(),
                    Ok(None) => view! { <Redirect path="/login" /> }.into_any(),
                    Err(error) => view! {
                        <p class="mt-[1.2rem] bg-black font-mono text-xs text-[#e2a340]">
                            {error.to_string()}
                        </p>
                    }.into_any(),
                }
            })}
        </Suspense>
    }
}

#[component]
fn AuthenticatedOutlet(user: SessionUser) -> impl IntoView {
    provide_context(user);
    // Every admin page that shows ordering controls shares one move action, so
    // the rail and the open project never hold two different orders.
    provide_project_order(expect_context::<Portfolio>());
    view! { <Outlet /> }
}
