use super::super::{
    server_functions::{SessionUser, current_user},
    style::ADMIN_STYLE,
};
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
                    Err(error) => view! { <p class="err">{error.to_string()}</p> }.into_any(),
                }
            })}
        </Suspense>
    }
}

#[component]
fn AuthenticatedOutlet(user: SessionUser) -> impl IntoView {
    provide_context(user);
    view! {
        <style inner_html=ADMIN_STYLE></style>
        <Outlet />
    }
}
