use crate::app::editor_controller::EditorController;
use leptos::prelude::*;

#[component]
pub fn NoticeView(editor: EditorController) -> impl IntoView {
    view! {
        <div aria-live="polite" class="pointer-events-none fixed top-4 right-4 z-30 text-[12.5px]">
            {move || {
                editor.notice().map(|message| {
                    view! {
                        <p class="max-w-[46ch] border border-[#2b3037] bg-[#0b0e11] px-3.5 py-2 text-white">
                            {message}
                        </p>
                    }
                })
            }}
        </div>
    }
}
