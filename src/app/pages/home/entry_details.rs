use super::view_model::HomeViewModel;
use leptos::prelude::*;

#[component]
pub fn EntryDetails() -> impl IntoView {
    let view_model = expect_context::<HomeViewModel>();

    view! {
        <p class="text-[11px] tracking-[0.24em] text-[#4c525a] uppercase">
            {move || view_model.current().map(|entry| entry.section.label())}
        </p>
        <h1 class="mt-3 font-sans text-[clamp(1.75rem,4.5vw,2.75rem)] leading-[1.1] font-semibold text-white">
            {move || view_model.current().map(|entry| entry.name)}
        </h1>
        {move || {
            view_model.current().and_then(|entry| entry.lead).map(|lead| {
                view! {
                    <p class="mt-4 font-sans text-[17px] leading-[1.5] text-white sm:text-[19px]">
                        {lead}
                    </p>
                }
            })
        }}
        <p class="mt-5 font-sans text-[16px] leading-[1.7] text-[#aab2bb] sm:text-[17px]">
            {move || view_model.current().map(|entry| entry.body)}
        </p>
        {move || {
            let focus = view_model.current().map(|entry| entry.focus).unwrap_or_default();
            (!focus.is_empty()).then(|| {
                view! {
                    <dl class="mt-5 grid grid-cols-[max-content_minmax(0,1fr)] gap-x-[2ch] gap-y-[5px] text-[13px]">
                        {focus
                            .into_iter()
                            .map(|area| {
                                view! {
                                    <div class="contents">
                                        <dt class="text-white">{area.label}</dt>
                                        <dd class="text-[#6c757f]">{area.detail}</dd>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </dl>
                }
            })
        }}
        <p class="mt-6 text-[12px] tracking-[0.08em] text-[#6c757f]">
            {move || view_model.current().map(|entry| entry.meta.join("  \u{b7}  "))}
        </p>
    }
}
