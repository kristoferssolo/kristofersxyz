use super::{command_line::CommandLine, view_model::HomeViewModel};
use crate::app::{editor::Mode, pages::AMBER};
use leptos::prelude::*;

#[component]
pub(super) fn StatusLine() -> impl IntoView {
    let view_model = expect_context::<HomeViewModel>();

    view! {
        <footer class="h-7 shrink-0">
            {move || match view_model.state.get().mode {
                Mode::Normal => {
                    view! {
                        <div
                            class="flex h-full items-stretch text-[12px] text-[#8b939d]"
                            style="background:#0d1013"
                        >
                            <span
                                class="flex items-center px-3 font-semibold text-black"
                                style=format!("background:{AMBER}")
                            >
                                "NORMAL"
                            </span>
                            <span class="flex items-center px-3 text-white">"kristofers.xyz"</span>
                            <span class="hidden items-center px-3 text-[#6c757f] md:flex">
                                ":help"
                            </span>
                            <span class="ml-auto flex items-stretch">
                                <span class="flex items-center px-3 tabular-nums">
                                    {move || {
                                        format!("[{}/{}]", view_model.position(), view_model.total())
                                    }}
                                </span>
                                <span class="flex items-center px-3 tabular-nums">
                                    {move || {
                                        format!(
                                            "{}%",
                                            view_model.position() * 100 / view_model.total()
                                        )
                                    }}
                                </span>
                            </span>
                        </div>
                    }
                    .into_any()
                }
                Mode::Command(text) => view! { <CommandLine prompt=":" text /> }.into_any(),
                Mode::Search(text) => view! { <CommandLine prompt="/" text /> }.into_any(),
            }}
        </footer>
    }
}
