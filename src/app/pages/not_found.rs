use crate::app::{
    content::Portfolio,
    editor::EntryId,
    editor_controller::EditorController,
    layout::{BlankPage, StatusLocation},
};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

/// Unknown paths, reported the way the editor would report a missing file.
#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <MissingPage
            title="E484: Can't open file"
            heading="Can't open file"
            description="There is nothing at this path. The portfolio lives at the root."
        />
    }
}

/// A missing public resource, presented using the portfolio editor shell.
#[component]
pub fn MissingPage(
    title: &'static str,
    heading: &'static str,
    description: &'static str,
) -> impl IntoView {
    let content = expect_context::<Portfolio>().snapshot();
    let editor = EditorController::routes(&content, &EntryId::Profile);
    let status = editor.status("[No Name]", || StatusLocation::Cursor {
        line: 0,
        column: 0,
    });

    view! {
        <Title text=title />
        <BlankPage editor status>
            <section class="flex min-h-0 flex-1 items-center overflow-y-auto px-5 py-14 sm:px-10 md:px-14">
                <div class="max-w-[62ch]">
                    <p class="text-[11px] tracking-[0.24em] text-[#4c525a] uppercase">"E484"</p>
                    <h1 class="mt-3 font-sans text-[clamp(1.75rem,4.5vw,2.75rem)] leading-[1.1] font-semibold text-white">
                        {heading}
                    </h1>
                    <p class="mt-5 font-sans text-[16px] leading-[1.7] text-[#aab2bb] sm:text-[17px]">
                        {description}
                    </p>
                    <A
                        href="/"
                        attr:class="mt-6 inline-block text-[13px] text-white underline decoration-[#3c424a] underline-offset-[5px] hover:decoration-[#e2a340]"
                    >
                        "Back to profile"
                    </A>
                </div>
            </section>
        </BlankPage>
    }
}
