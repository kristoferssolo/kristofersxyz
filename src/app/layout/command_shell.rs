use super::{EditorShell, StatusLocation};
use crate::app::{content::Portfolio, editor::EntryId, editor_controller::EditorController};
use leptos::prelude::*;

#[component]
pub fn CommandShell(#[prop(into)] filename: String, children: Children) -> impl IntoView {
    let content = expect_context::<Portfolio>().current();
    let editor = EditorController::restricted(&content, &EntryId::Profile);
    let status = editor.status(filename, || StatusLocation::Cursor { line: 0, column: 0 });

    view! {
        <EditorShell editor status>
            {children()}
        </EditorShell>
    }
}
