use super::{
    EditorShell, StatusBarState,
    sidebar::{CollapsibleSidebar, PortfolioNavigation},
};
use crate::app::{editor::EntryId, editor_controller::EditorController};
use leptos::prelude::*;

/// Wraps page content in the shared sidebar and status bar, and routes global
/// keyboard input through the supplied editor session.
#[component]
pub fn BlankPage(
    editor: EditorController,
    #[prop(into)] status: Signal<StatusBarState>,
    #[prop(optional)] on_select: Option<Callback<EntryId>>,
    children: Children,
) -> impl IntoView {
    let active = Signal::derive(move || editor.active());
    let sidebar = Signal::derive(move || editor.sidebar());

    view! {
        <EditorShell editor status>
            <CollapsibleSidebar
                id="portfolio-navigation"
                label="portfolio navigation"
                width="340px"
                open=sidebar
                on_toggle=Callback::new(move |()| editor.toggle_sidebar())
                navigation=view! { <PortfolioNavigation active on_select /> }.into_any()
            >
                <div class="flex h-full min-h-0 min-w-0 flex-col">{children()}</div>
            </CollapsibleSidebar>
        </EditorShell>
    }
}
