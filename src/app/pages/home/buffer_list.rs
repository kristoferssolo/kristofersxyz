use super::{browser::row_id, view_model::HomeViewModel};
use crate::app::pages::AMBER;
use leptos::prelude::*;

#[component]
pub(super) fn BufferList() -> impl IntoView {
    let view_model = expect_context::<HomeViewModel>();

    view! {
        <div
            role="listbox"
            aria-label="Buffers"
            class="min-h-0 overflow-y-auto border-b border-[#1e2126] py-3 text-[13px] md:border-r md:border-b-0"
        >
            {view_model
                .groups()
                .into_iter()
                .map(|(section, entries)| {
                    view! {
                        <div role="group" aria-label=section.label() class="mt-4 first:mt-0">
                            <p
                                aria-hidden="true"
                                class="mb-1 pl-[7ch] text-[10px] tracking-[0.24em] text-[#4c525a] uppercase"
                            >
                                {section.label()}
                            </p>
                            {entries
                                .into_iter()
                                .map(|(id, name)| {
                                    let number = view_model.number_of(&id).unwrap_or_default();
                                    let is_active = move || view_model.position() == number;
                                    let clicked = id.clone();
                                    let tint = move || {
                                        if is_active() {
                                            format!("color:{AMBER}")
                                        } else {
                                            "color:transparent".to_owned()
                                        }
                                    };
                                    let counter = move || {
                                        if is_active() {
                                            format!("color:{AMBER}")
                                        } else {
                                            "color:#3c424a".to_owned()
                                        }
                                    };

                                    view! {
                                        <button
                                            type="button"
                                            role="option"
                                            id=row_id(&id)
                                            aria-selected=move || {
                                                if is_active() { "true" } else { "false" }
                                            }
                                            tabindex=move || if is_active() { 0 } else { -1 }
                                            on:click=move |_| view_model.pick(&clicked)
                                            class="flex w-full items-baseline gap-[1ch] px-3 py-[3px] text-left hover:bg-[#101317] focus-visible:outline-none"
                                            class=("bg-[#14181d]", is_active)
                                        >
                                            <span
                                                aria-hidden="true"
                                                class="w-[1ch] shrink-0"
                                                style=tint
                                            >
                                                "\u{258e}"
                                            </span>
                                            <span
                                                aria-hidden="true"
                                                class="w-[3ch] shrink-0 text-right tabular-nums"
                                                style=counter
                                            >
                                                {move || {
                                                    if is_active() {
                                                        number.to_string()
                                                    } else {
                                                        view_model.position().abs_diff(number).to_string()
                                                    }
                                                }}
                                            </span>
                                            <span
                                                class="truncate"
                                                class=("text-white", is_active)
                                                class=("text-[#8b939d]", move || !is_active())
                                            >
                                                {name}
                                            </span>
                                        </button>
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
}
