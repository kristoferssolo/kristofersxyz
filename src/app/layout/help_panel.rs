use crate::app::editor_controller::EditorController;
use leptos::prelude::*;

/// Keyboard reference opened by `?` or `:help`.
#[component]
pub fn HelpPanel(editor: EditorController) -> impl IntoView {
    view! {
        {move || {
            editor.help().then(|| {
                view! {
                    <div class="fixed inset-0 z-20 grid place-items-center bg-black/85 px-4 py-8">
                        <section
                            role="dialog"
                            aria-modal="true"
                            aria-labelledby="keyboard-help-title"
                            class="w-full max-w-[720px] border border-[#2b3037] bg-black px-5 py-5 text-[12px] shadow-2xl sm:px-7 sm:py-6"
                        >
                            <div class="flex items-baseline justify-between gap-6 border-b border-[#1e2126] pb-3">
                                <h2
                                    id="keyboard-help-title"
                                    class="font-sans text-lg font-semibold text-white"
                                >
                                    "Keyboard help"
                                </h2>
                                <span class="shrink-0 text-[#6c757f]">"Esc closes"</span>
                            </div>

                            <div class="mt-5 grid gap-x-10 gap-y-6 sm:grid-cols-2">
                                <HelpGroup
                                    title="Navigation"
                                    bindings=vec![
                                        ("j / k", "next / previous page"),
                                        ("J / K", "next / previous section"),
                                        ("g / G", "first / last page"),
                                        ("1-9", "select page by number"),
                                        ("Enter", "open selected page"),
                                    ]
                                />
                                <HelpGroup
                                    title="Commands"
                                    bindings=vec![
                                        (":e[dit]", "open a page by name"),
                                        (":h[elp]", "open this reference"),
                                        (":w[ork]", "open the first Project"),
                                        (":c[ontact]", "open contact"),
                                        ("/text", "search portfolio content"),
                                        ("Ctrl+F", "start search"),
                                        ("Ctrl+B", "show or hide the sidebar"),
                                    ]
                                />
                            </div>
                        </section>
                    </div>
                }
            })
        }}
    }
}

#[component]
fn HelpGroup(title: &'static str, bindings: Vec<(&'static str, &'static str)>) -> impl IntoView {
    view! {
        <section>
            <h3 class="text-[10px] tracking-[0.2em] text-[#59616a] uppercase">{title}</h3>
            <dl class="mt-3 grid grid-cols-[10ch_minmax(0,1fr)] gap-x-3 gap-y-2">
                {bindings
                    .into_iter()
                    .map(|(keys, action)| {
                        view! {
                            <dt class="text-[#e2a340]">{keys}</dt>
                            <dd class="text-[#aab2bb]">{action}</dd>
                        }
                    })
                    .collect_view()}
            </dl>
        </section>
    }
}
