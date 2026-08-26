use super::view_model::HomeViewModel;
use crate::app::editor::SectionId;
use leptos::prelude::*;

#[component]
pub fn ActionLinks() -> impl IntoView {
    let view_model = expect_context::<HomeViewModel>();

    move || {
        view_model
            .current()
            .filter(|entry| entry.section == SectionId::Profile)
            .map(|_| {
                view! {
                    <div class="mt-8 flex flex-wrap gap-x-6 gap-y-2 text-[13px]">
                        {view_model
                            .actions()
                            .into_iter()
                            .map(|action| {
                                let hint = action
                                    .target
                                    .as_ref()
                                    .and_then(|entry| view_model.number_of(entry));
                                let target = action.target.clone();

                                view! {
                                    <a
                                        class="text-white underline decoration-[#3c424a] underline-offset-[5px] hover:decoration-[#e2a340]"
                                        href=action.href
                                        download=action.download
                                        on:click=move |_| {
                                            if let Some(entry) = target.as_ref() {
                                                view_model.pick(entry);
                                            }
                                        }
                                    >
                                        {action.label}
                                        {hint.map(|number| {
                                            view! {
                                                <span
                                                    aria-hidden="true"
                                                    class="ml-[1ch] hidden text-[#4c525a] no-underline md:inline"
                                                >
                                                    {format!("[{number}]")}
                                                </span>
                                            }
                                        })}
                                    </a>
                                }
                            })
                            .collect_view()}
                    </div>
                }
            })
    }
}
