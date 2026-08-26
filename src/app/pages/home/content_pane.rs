use super::{
    action_links::ActionLinks, entry_details::EntryDetails, external_links::ExternalLinks,
    next_entry::NextEntry,
};
use leptos::prelude::*;

#[component]
pub fn ContentPane() -> impl IntoView {
    view! {
        <section
            id="buffer-content"
            class="min-h-0 flex-1 overflow-y-auto px-5 py-8 sm:px-10 md:px-14 md:py-14"
        >
            <div class="max-w-[62ch]">
                <EntryDetails />
                <ActionLinks />
                <ExternalLinks />
            </div>
            <NextEntry />
        </section>
    }
}
