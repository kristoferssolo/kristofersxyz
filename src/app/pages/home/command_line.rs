use leptos::prelude::*;

#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props own dynamic text"
)]
pub(super) fn CommandLine(prompt: &'static str, text: String) -> impl IntoView {
    view! {
        <div class="flex h-full items-center gap-[1ch] rounded-md border border-[#2b3037] bg-[#0b0e11] px-2.5 text-[13px] text-white">
            <span aria-hidden="true" class="text-[#e2a340]">"\u{276f}"</span>
            <span>{format!("{prompt}{text}")}</span>
            <span aria-hidden="true" class="inline-block h-[15px] w-[1ch] bg-white"></span>
        </div>
    }
}
