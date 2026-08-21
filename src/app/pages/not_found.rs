use super::AMBER;
use leptos::prelude::*;
use leptos_meta::Title;

/// Unknown paths, reported the way the editor would report a missing file.
#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        // Overrides the site title from `SiteMeta`, which would otherwise
        // announce this page as the homepage in the tab and to crawlers.
        <Title text="E484: Can't open file" />
        <main class="flex min-h-dvh flex-col bg-black font-mono text-[#d4d7db]">
            <div class="flex flex-1 items-center px-5 sm:px-10 md:px-14">
                <div class="max-w-[62ch]">
                    <p class="text-[11px] tracking-[0.24em] text-[#4c525a] uppercase">"E484"</p>
                    <h1 class="mt-3 font-sans text-[clamp(1.75rem,4.5vw,2.75rem)] leading-[1.1] font-semibold text-white">
                        "Can't open file"
                    </h1>
                    <p class="mt-5 font-sans text-[16px] leading-[1.7] text-[#aab2bb] sm:text-[17px]">
                        "There is nothing at this path. The portfolio lives at the root."
                    </p>
                    <div class="mt-6 text-[13px]">
                        <a
                            class="text-white underline decoration-[#3c424a] underline-offset-[5px] hover:decoration-[#e2a340]"
                            href="/"
                        >
                            "kristofers.xyz"
                        </a>
                    </div>
                </div>
            </div>

            <footer class="flex shrink-0 items-stretch text-[12px] text-[#8b939d]" style="background:#0d1013">
                <span
                    class="px-3 py-1 font-semibold text-black"
                    style=format!("background:{AMBER}")
                >
                    "NORMAL"
                </span>
                <span class="truncate px-3 py-1 text-white">"[No Name]"</span>
                <span class="ml-auto px-3 py-1 tabular-nums">"0:0"</span>
            </footer>
        </main>
    }
}
