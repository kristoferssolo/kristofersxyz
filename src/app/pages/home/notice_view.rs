use super::view_model::HomeViewModel;
use leptos::prelude::*;

#[component]
pub(super) fn NoticeView() -> impl IntoView {
    let view_model = expect_context::<HomeViewModel>();

    view! {
        <div aria-live="polite" class="pointer-events-none fixed top-4 right-4 z-10 text-[12.5px]">
            {move || {
                view_model.notice().map(|notice| {
                    view! {
                        <p class="max-w-[46ch] rounded-lg border border-[#2b3037] bg-[#0b0e11] px-3.5 py-2 text-white">
                            {notice.message}
                        </p>
                    }
                })
            }}
        </div>
    }
}
