//! The Project Screenshot controls in the project editor.
//!
//! Screenshots get their own section rather than a line in the Technology and
//! link rail, because a thumbnail needs more width than a name does. Each one
//! is a row: the stored image beside the two fields that describe it, with its
//! place in the reader's order and the controls that change it.
//!
//! Ordering is by button, so a pointer, a keyboard, and assistive technology
//! all reach it the same way, and each button submits its own form so a move
//! works before the page hydrates. A move that would leave the order renders
//! disabled, server-side included. Deleting always asks first.
//!
//! Each row owns the signals behind its own fields, so a rejected save keeps
//! what the Owner typed instead of reverting to the stored text.

use super::{Eyebrow, FormMessage};
use crate::{
    app::{
        admin::{
            error::AdminError,
            server_functions::{
                DeleteProjectScreenshot, MoveProjectScreenshot, SaveScreenshotDetails,
                upload_project_screenshot,
            },
        },
        content::{Portfolio, PortfolioContent},
    },
    domain::{
        MAX_SCREENSHOT_BYTES, MAX_SCREENSHOT_EDGE, ProjectScreenshot, ProjectSlug, ScreenshotId,
        ScreenshotMove, ScreenshotSize,
    },
};
use leptos::{
    ev::SubmitEvent,
    form::ActionForm,
    prelude::*,
    wasm_bindgen::JsCast,
    web_sys::{FormData, HtmlFormElement},
};
use lucide_leptos::{ChevronDown, ChevronUp, Trash2, Upload};

/// The width every thumbnail is drawn at. Height follows the stored aspect
/// ratio, so a row reserves the space the image will occupy.
const THUMBNAIL_WIDTH: u32 = 200;

/// The upload limit as the hint states it.
const MAX_MEGABYTES: usize = MAX_SCREENSHOT_BYTES / (1_024 * 1_024);

/// The endpoint the upload form posts to. Naming it here keeps the form
/// working when the page has not hydrated, where there is no action to
/// dispatch.
const UPLOAD_ENDPOINT: &str = "/api/upload_project_screenshot";

/// The formats a browser should offer in its file picker. The server decides
/// what a file actually is; this only saves the Owner a wasted upload.
const ACCEPTED_FORMATS: &str = "image/png,image/jpeg,image/webp";

const FIELD: &str = "m-0 w-full border-0 border-b border-[#1e2126] bg-transparent px-[.25rem] \
    py-[.32rem] font-[inherit] text-[13px] text-white focus:border-b-[#e2a340] \
    focus:bg-[#0b0e11] focus:outline-none";
const FIELD_LABEL: &str = "mt-[.7rem] block text-[10px] tracking-[.12em] text-[#767d87] uppercase";
const ICON_BUTTON: &str = "inline-flex h-6 w-6 items-center justify-center border \
    border-transparent bg-transparent p-0 text-[#767d87] enabled:cursor-pointer \
    enabled:hover:border-[#2b3037] enabled:hover:text-white disabled:cursor-not-allowed \
    disabled:text-[#2f343a]";
const SMALL_BUTTON: &str = "cursor-pointer border border-[#2b3037] bg-[#080a0d] px-[.8rem] \
    py-[.3rem] font-[inherit] text-[12px] text-[#c3c9cf] hover:border-[#e2a340] hover:text-white";
const META: &str = "m-0 mt-[.5rem] text-[11px] break-all text-[#59616a]";

/// The Project Screenshot section of the project editor.
#[component]
pub fn ProjectScreenshotEditor(slug: ProjectSlug) -> impl IntoView {
    let portfolio = expect_context::<Portfolio>();
    let confirming = RwSignal::new(None::<ScreenshotId>);
    let listed = {
        let slug = slug.clone();
        Signal::derive(move || screenshots_of(portfolio, &slug))
    };
    let total = Signal::derive(move || listed.get().len());

    view! {
        <section class="mt-9 border-t border-[#1e2126]">
            <div class="flex flex-wrap items-baseline justify-between gap-[2ch] pt-[.8rem]">
                <Eyebrow>"Screenshots"</Eyebrow>
                <span class="text-[11px] text-[#59616a]">
                    {move || {
                        let count = total.get();
                        if count == 1 {
                            "1 in reader order".to_owned()
                        } else {
                            format!("{count} in reader order")
                        }
                    }}
                </span>
            </div>

            <Show
                when=move || total.get() != 0
                fallback=|| {
                    view! {
                        <p class="m-0 py-[.9rem] text-[12px] leading-[1.55] text-[#8b939d]">
                            <b class="font-medium text-white">"No screenshots yet."</b>
                            " A project can stay text only. Add one when a picture shows something the description cannot."
                        </p>
                    }
                }
            >
                <ForEnumerate
                    each=move || listed.get()
                    key=|screenshot: &ProjectScreenshot| screenshot.id.clone()
                    children=move |index, screenshot| {
                        view! { <ScreenshotRow index total screenshot confirming /> }
                    }
                />
            </Show>

            <ScreenshotUpload slug />
        </section>
    }
}

/// One screenshot: its stored image, the fields that describe it, and the
/// controls that place or remove it.
#[component]
fn ScreenshotRow(
    index: ReadSignal<usize>,
    total: Signal<usize>,
    screenshot: ProjectScreenshot,
    confirming: RwSignal<Option<ScreenshotId>>,
) -> impl IntoView {
    let portfolio = expect_context::<Portfolio>();
    let details = ServerAction::<SaveScreenshotDetails>::new();
    follow(details, portfolio);

    let id = screenshot.id.clone();
    let form_id = id.to_string();
    let alt = RwSignal::new(screenshot.alt.to_string());
    let caption = RwSignal::new(
        screenshot
            .caption
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default(),
    );
    let error = Signal::derive(move || details.value().get().and_then(Result::err));
    let place = move || position(index.get());
    let field_id = {
        let id = screenshot.id.clone();
        move |kind: &str| format!("{kind}-{id}")
    };
    let alt_id = field_id("screenshot-alt");
    let caption_id = field_id("screenshot-caption");
    let alt_label = alt_id.clone();
    let caption_label = caption_id.clone();

    view! {
        <div class="grid grid-cols-1 gap-6 border-b border-[#101317] py-5 min-[721px]:grid-cols-[200px_minmax(0,1fr)]">
            <div>
                <img
                    class="block max-w-full border border-[#1e2126] bg-[#0b0e11]"
                    src=screenshot.media_path()
                    width=THUMBNAIL_WIDTH
                    height=thumbnail_height(screenshot.size)
                    alt=""
                    loading="lazy"
                    decoding="async"
                />
                <p class=META>
                    {format!(
                        "{} · {} × {}",
                        screenshot.media_type,
                        screenshot.size.width(),
                        screenshot.size.height(),
                    )}
                </p>
                <p class=META>{screenshot.media_path()}</p>
            </div>

            <div class="min-w-0">
                <div class="flex flex-wrap items-center justify-between gap-[1ch]">
                    <span class="text-[11px] text-[#8b939d]">
                        {move || format!("Screenshot {} of {}", place(), position(total.get().saturating_sub(1)))}
                    </span>
                    <div class="flex items-center gap-[.1rem]">
                        <MoveControl
                            id=id.clone()
                            movement=ScreenshotMove::Up
                            direction="up"
                            place=Signal::derive(place)
                            disabled=Signal::derive(move || index.get() == 0)
                        >
                            <ChevronUp size=14 />
                        </MoveControl>
                        <MoveControl
                            id=id.clone()
                            movement=ScreenshotMove::Down
                            direction="down"
                            place=Signal::derive(place)
                            disabled=Signal::derive(move || {
                                index.get().saturating_add(1) >= total.get()
                            })
                        >
                            <ChevronDown size=14 />
                        </MoveControl>
                        <DeleteToggle id=id.clone() place=Signal::derive(place) confirming />
                    </div>
                </div>

                <ActionForm action=details attr:class="mt-[.4rem]">
                    <input type="hidden" name="id" value=form_id />
                    <label class=FIELD_LABEL for=alt_label>
                        "Alternative text (required)"
                    </label>
                    <input
                        id=alt_id
                        name="alt"
                        class=FIELD
                        class=("border-b-[#c2542f]", move || {
                            error.get() == Some(AdminError::InvalidAltText)
                        })
                        value=move || alt.get()
                        prop:value=move || alt.get()
                        on:input=move |event| alt.set(event_target_value(&event))
                        aria-invalid=move || {
                            (error.get() == Some(AdminError::InvalidAltText)).then_some("true")
                        }
                        spellcheck="false"
                    />
                    <label class=FIELD_LABEL for=caption_label>
                        "Caption (optional)"
                    </label>
                    <input
                        id=caption_id
                        name="caption"
                        class=format!("{FIELD} text-[12px] text-[#a7aeb6]")
                        class=("border-b-[#c2542f]", move || {
                            error.get() == Some(AdminError::InvalidCaption)
                        })
                        value=move || caption.get()
                        prop:value=move || caption.get()
                        on:input=move |event| caption.set(event_target_value(&event))
                        aria-invalid=move || {
                            (error.get() == Some(AdminError::InvalidCaption)).then_some("true")
                        }
                        placeholder="Leave empty for no caption"
                        spellcheck="false"
                    />
                    {move || {
                        error
                            .get()
                            .map(|error| {
                                view! {
                                    <p class="m-0 mt-[.4rem] text-[11px] leading-[1.45] text-[#e2a340]">
                                        {error.to_string()}
                                    </p>
                                }
                            })
                    }}
                    <button type="submit" class=format!("mt-[.8rem] {SMALL_BUTTON}")>
                        {move || format!("Save screenshot {} details", place())}
                    </button>
                </ActionForm>

                <DeleteConfirmation id=id place=Signal::derive(place) confirming />
            </div>
        </div>
    }
}

/// One step through the reader's order. Each button is its own form, so it
/// works before the page hydrates, and it names the screenshot it moves.
#[component]
fn MoveControl(
    id: ScreenshotId,
    movement: ScreenshotMove,
    direction: &'static str,
    place: Signal<String>,
    disabled: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let portfolio = expect_context::<Portfolio>();
    let action = ServerAction::<MoveProjectScreenshot>::new();
    follow(action, portfolio);

    view! {
        <ActionForm action attr:class="contents">
            <input type="hidden" name="id" value=id.to_string() />
            <input type="hidden" name="movement" value=movement.to_string() />
            <button
                type="submit"
                class=ICON_BUTTON
                disabled=move || disabled.get()
                aria-label=move || format!("Move screenshot {} {direction}", place.get())
            >
                {children()}
            </button>
        </ActionForm>
    }
}

/// Opens the confirmation for one screenshot. Nothing is deleted here.
#[component]
fn DeleteToggle(
    id: ScreenshotId,
    place: Signal<String>,
    confirming: RwSignal<Option<ScreenshotId>>,
) -> impl IntoView {
    let open = {
        let id = id.clone();
        move || confirming.get().is_some_and(|current| current == id)
    };

    view! {
        <button
            type="button"
            class=ICON_BUTTON
            aria-expanded=move || if open() { "true" } else { "false" }
            aria-label=move || format!("Delete screenshot {}", place.get())
            on:click=move |_| {
                let id = id.clone();
                confirming.update(|current| {
                    *current = if current.as_ref() == Some(&id) { None } else { Some(id) };
                });
            }
        >
            <Trash2 size=14 />
        </button>
    }
}

/// The explicit confirmation a deletion needs. It states what goes and offers
/// the way out beside it.
#[component]
fn DeleteConfirmation(
    id: ScreenshotId,
    place: Signal<String>,
    confirming: RwSignal<Option<ScreenshotId>>,
) -> impl IntoView {
    let portfolio = expect_context::<Portfolio>();
    let action = ServerAction::<DeleteProjectScreenshot>::new();
    follow(action, portfolio);
    Effect::new(move |_| {
        if matches!(action.value().get(), Some(Ok(_))) {
            confirming.set(None);
        }
    });

    let shown = {
        let id = id.clone();
        move || confirming.get().is_some_and(|current| current == id)
    };
    let field = StoredValue::new(id.to_string());

    view! {
        <Show when=shown fallback=|| ()>
            <div class="mt-[.7rem] flex flex-wrap items-center gap-[1ch] border border-[#2b3037] bg-[#0b0e11] px-[.8rem] py-[.6rem]">
                <p class="m-0 flex-1 text-[12px] text-[#e2a340]">
                    {move || {
                        format!(
                            "Delete screenshot {}? The image is removed from the project detail.",
                            place.get(),
                        )
                    }}
                </p>
                <ActionForm action attr:class="contents">
                    <input type="hidden" name="id" value=move || field.get_value() />
                    <button
                        type="submit"
                        class=format!("{SMALL_BUTTON} hover:border-[#c2542f]")
                    >
                        {move || format!("Delete screenshot {}", place.get())}
                    </button>
                </ActionForm>
                <button
                    type="button"
                    class=SMALL_BUTTON
                    on:click=move |_| confirming.set(None)
                >
                    "Keep it"
                </button>
            </div>
        </Show>
    }
}

/// The one control that adds a screenshot.
///
/// The form posts to the upload endpoint on its own, so it still works without
/// the client bundle. Once the page has hydrated, the same submit is sent as a
/// multipart request whose answer refreshes the portfolio in place.
#[component]
fn ScreenshotUpload(slug: ProjectSlug) -> impl IntoView {
    let portfolio = expect_context::<Portfolio>();
    let upload = Action::new_local(|data: &FormData| {
        let data = data.clone();
        upload_project_screenshot(data.into())
    });
    Effect::new(move |_| {
        if let Some(Ok(content)) = upload.value().get() {
            portfolio.replace(content);
        }
    });

    let pending = upload.pending();
    let error = Signal::derive(move || upload.value().get().and_then(Result::err));

    view! {
        <form
            class="mt-5 border border-dashed border-[#2b3037] px-[.8rem] py-[.7rem]"
            action=UPLOAD_ENDPOINT
            method="post"
            enctype="multipart/form-data"
            on:submit=move |event: SubmitEvent| {
                let Some(form) = event
                    .target()
                    .and_then(|target| target.dyn_into::<HtmlFormElement>().ok())
                else {
                    return;
                };
                let Ok(data) = FormData::new_with_form(&form) else {
                    return;
                };
                event.prevent_default();
                upload.dispatch_local(data);
                form.reset();
            }
        >
            <input type="hidden" name="slug" value=slug.to_string() />
            <div class="flex flex-wrap items-end gap-x-[1.5ch] gap-y-[.6rem]">
                <span class="inline-flex pb-[.4rem] text-[#767d87]" aria-hidden="true">
                    <Upload size=14 />
                </span>
                <label class="min-w-[24ch] flex-1">
                    <span class="block text-[10px] tracking-[.12em] text-[#767d87] uppercase">
                        "Alternative text (required)"
                    </span>
                    <input
                        name="alt"
                        class=FIELD
                        placeholder="What the screenshot shows"
                        required
                        spellcheck="false"
                    />
                </label>
                <label class="min-w-[24ch] flex-1">
                    <span class="block text-[10px] tracking-[.12em] text-[#767d87] uppercase">
                        "Image"
                    </span>
                    <input
                        type="file"
                        name="image"
                        class="mt-[.35rem] block w-full font-[inherit] text-[12px] text-[#c3c9cf] file:mr-[1ch] file:cursor-pointer file:border file:border-[#2b3037] file:bg-[#080a0d] file:px-[.7rem] file:py-[.25rem] file:font-[inherit] file:text-[12px] file:text-[#c3c9cf] hover:file:border-[#e2a340]"
                        accept=ACCEPTED_FORMATS
                        required
                    />
                </label>
                <button
                    type="submit"
                    class="cursor-pointer border border-[#30363d] bg-[#080a0d] px-[1.2rem] py-[.4rem] font-[inherit] text-[13px] text-white hover:border-[#e2a340] disabled:cursor-not-allowed disabled:text-[#2f343a]"
                    disabled=move || pending.get()
                >
                    {move || if pending.get() { "Uploading" } else { "Upload" }}
                </button>
            </div>
            <p class="m-0 mt-[.6rem] text-[11px] text-[#767d87]">
                {format!(
                    "PNG, JPEG, or WebP. Up to {MAX_MEGABYTES} MB and {MAX_SCREENSHOT_EDGE} by {MAX_SCREENSHOT_EDGE} pixels.",
                )}
            </p>
            <Show when=move || pending.get() fallback=|| ()>
                <p class="m-0 mt-[.4rem] text-[11px] text-[#8b939d]">
                    "Sending the screenshot. The row appears here once it is stored."
                </p>
            </Show>
            {move || error.get().map(|error| view! { <FormMessage>{error.to_string()}</FormMessage> })}
        </form>
    }
}

/// Keeps the shared portfolio in step with a completed change, so the rows,
/// the public pages, and the page metadata all read the same content.
fn follow<ServerFn>(action: ServerAction<ServerFn>, portfolio: Portfolio)
where
    ServerFn: leptos::server_fn::ServerFn<Output = PortfolioContent, Error = AdminError>
        + Clone
        + Send
        + Sync
        + 'static,
{
    Effect::new(move |_| {
        if let Some(Ok(content)) = action.value().get() {
            portfolio.replace(content);
        }
    });
}

/// The screenshots of the Project being edited, in the reader's order.
fn screenshots_of(portfolio: Portfolio, slug: &ProjectSlug) -> Vec<ProjectScreenshot> {
    portfolio
        .current()
        .projects
        .into_iter()
        .find(|project| project.slug == *slug)
        .map(|project| project.screenshots.as_slice().to_vec())
        .unwrap_or_default()
}

/// The one-based place the Owner sees for a zero-based row index.
fn position(index: usize) -> String {
    format!("{:02}", index.saturating_add(1))
}

/// The height a thumbnail occupies at [`THUMBNAIL_WIDTH`], so the row reserves
/// the right space before the image arrives.
fn thumbnail_height(size: ScreenshotSize) -> u32 {
    u64::from(size.height())
        .saturating_mul(u64::from(THUMBNAIL_WIDTH))
        .checked_div(u64::from(size.width()))
        .and_then(|height| u32::try_from(height).ok())
        .unwrap_or(THUMBNAIL_WIDTH)
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_ok;

    fn size(width: u32, height: u32) -> ScreenshotSize {
        assert_ok!(ScreenshotSize::try_from((width, height)))
    }

    #[test]
    fn a_thumbnail_keeps_the_stored_aspect_ratio() {
        assert_eq!(thumbnail_height(size(1600, 1000)), 125);
        assert_eq!(thumbnail_height(size(1000, 1600)), 320);
    }

    /// A very wide screenshot still reserves a visible row rather than
    /// collapsing to nothing.
    #[test]
    fn a_thumbnail_never_reserves_zero_height() {
        assert_eq!(thumbnail_height(size(4096, 1)), 1);
    }

    #[test]
    fn the_hint_states_the_upload_limit() {
        assert_eq!(MAX_MEGABYTES, 5);
    }
}
