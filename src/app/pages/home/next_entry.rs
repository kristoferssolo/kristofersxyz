use super::view_model::HomeViewModel;
use leptos::prelude::*;

#[component]
pub(super) fn NextEntry() -> impl IntoView {
    let view_model = expect_context::<HomeViewModel>();

    view! {
        <div class="mt-14 max-w-[62ch] border-t border-[#1e2126] pt-4">
            {move || {
                let (next, name) = view_model.next()?;
                Some(view! {
                    <button
                        type="button"
                        on:click=move |_| view_model.pick(&next)
                        class="flex w-full items-baseline gap-[2ch] text-left text-[12px] text-[#6c757f] hover:text-white"
                    >
                        <span class="text-[#e2a340]">
                            <span class="hidden md:inline">"j"</span>
                            <span aria-hidden="true" class="md:hidden">"\u{2193}"</span>
                        </span>
                        <span>{name}</span>
                    </button>
                })
            }}
        </div>
    }
}
