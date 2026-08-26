use super::{model::Links, view_model::HomeViewModel};
use leptos::prelude::*;

#[component]
pub fn ExternalLinks() -> impl IntoView {
    let view_model = expect_context::<HomeViewModel>();

    view! {
        <div class="mt-6 flex flex-wrap gap-x-6 gap-y-2 text-[13px]">
            {move || {
                view_model
                    .current()
                    .map_or(Links::Project(Vec::new()), |entry| entry.links)
                    .resolve()
                    .into_iter()
                    .map(|link| {
                        view! {
                            <a
                                class="text-white underline decoration-[#3c424a] underline-offset-[5px] hover:decoration-[#e2a340]"
                                href=link.href
                                rel=link.rel
                            >
                                {link.label}
                            </a>
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}
