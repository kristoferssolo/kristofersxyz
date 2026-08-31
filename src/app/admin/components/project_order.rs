//! The Owner's controls for the public project order.
//!
//! Order lives in the editor's project rail, where the Owner already switches
//! between Projects, and the open Project repeats the same two moves beside
//! its heading with its position. Both surfaces share one server action, so
//! there is one order and one refreshed portfolio rather than two.
//!
//! Every move is a submit button inside its own form, which keeps ordering
//! working with a pointer, with a keyboard, and before the page hydrates. A
//! Project that cannot move that way renders its button disabled, server-side
//! included. Dragging a row is an addition on top of that, available once the
//! page has hydrated and the Owner has turned arranging on.

use super::{EntryIcon, Icon, NAVIGATION_LINK, NewProjectLink};
use crate::{
    app::{
        admin::{admin_path_for_slug, server_functions::MoveProject},
        browser::focus_control,
        content::Portfolio,
    },
    domain::{ProjectMove, ProjectSlug},
};
use leptos::{form::ActionForm, prelude::*, web_sys::DragEvent};
use leptos_router::components::A;
use lucide_leptos::{ChevronDown, ChevronUp, GripVertical};

/// The move action the rail and the heading stepper share, and the control
/// that started the last move so focus can return to it.
#[derive(Clone, Copy)]
struct ProjectOrder {
    action: ServerAction<MoveProject>,
    pressed: RwSignal<Option<String>>,
}

impl ProjectOrder {
    /// Provides the shared action to the editor's ordering controls and keeps
    /// the portfolio in step with every move it completes.
    fn provide(portfolio: Portfolio) -> Self {
        let order = Self {
            action: ServerAction::new(),
            pressed: RwSignal::new(None),
        };

        Effect::new(move |_| {
            if let Some(Ok(content)) = order.action.value().get() {
                portfolio.replace(content);
                if let Some(id) = order.pressed.get_untracked() {
                    request_animation_frame(move || {
                        if !focus_control(&id) {
                            focus_control(&opposite(&id));
                        }
                    });
                }
            }
        });

        provide_context(order);
        order
    }

    fn expect() -> Self {
        expect_context::<Self>()
    }

    /// Applies a move the Owner made by dragging, and points focus at the
    /// dropped Project so the keyboard can carry it further.
    fn drag(self, slug: &ProjectSlug, onto: ProjectSlug) {
        self.pressed
            .set(Some(control_id(Place::Rail, &ProjectMove::Up, slug)));
        self.action.dispatch(MoveProject {
            slug: slug.to_string(),
            movement: ProjectMove::ToPlaceOf(onto).to_string(),
        });
    }
}

/// Which surface a control belongs to, so the rail and the heading stepper can
/// carry the same Project without sharing an element identity.
#[derive(Clone, Copy)]
enum Place {
    Rail,
    Heading,
}

impl Place {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Rail => "rail",
            Self::Heading => "heading",
        }
    }
}

/// Provides the editor's ordering state. The editor layout calls this once, so
/// every ordering control on the page shares one action.
pub fn provide_project_order(portfolio: Portfolio) {
    ProjectOrder::provide(portfolio);
}

/// The Projects group of the editor navigation: the entry that creates a
/// Project, the Projects in their public order, and the moves that reorder
/// them.
#[component]
pub fn ProjectRail(active: String) -> impl IntoView {
    let portfolio = expect_context::<Portfolio>();
    let arranging = RwSignal::new(false);
    let dragged = RwSignal::new(None::<ProjectSlug>);

    view! {
        <div class="mt-8 mb-[.8rem] flex items-baseline justify-between gap-[1ch]">
            <p class="text-[10px] tracking-[.2em] text-[#767d87] uppercase">"Projects"</p>
            <button
                type="button"
                class="cursor-pointer border-0 bg-transparent p-0 font-[inherit] text-[10px] tracking-[.12em] text-[#767d87] uppercase hover:text-[#e2a340] aria-pressed:text-[#e2a340]"
                aria-pressed=move || arranging.get().then_some("true")
                on:click=move |_| arranging.update(|on| *on = !*on)
            >
                "arrange"
            </button>
        </div>
        <NewProjectLink full_width=true />
        <ul class="mt-2 mb-2 list-none p-0">
            {move || {
                portfolio
                    .current()
                    .projects
                    .into_iter()
                    .map(|project| {
                        view! {
                            <ProjectRailRow
                                active=active.clone()
                                slug=project.slug
                                title=project.title
                                arranging
                                dragged
                            />
                        }
                    })
                    .collect_view()
            }}
        </ul>
        <p class="mb-2 text-[11px] leading-[1.5] text-[#767d87]">
            {move || {
                if arranging.get() {
                    "Drag a project onto the place you want it, or use the move buttons."
                } else {
                    "Use the move buttons to change the order readers see."
                }
            }}
        </p>
    }
}

/// One rail row: the link that opens the Project, and the moves that place it.
#[component]
fn ProjectRailRow(
    active: String,
    slug: ProjectSlug,
    title: String,
    arranging: RwSignal<bool>,
    dragged: RwSignal<Option<ProjectSlug>>,
) -> impl IntoView {
    let order = ProjectOrder::expect();
    let href = admin_path_for_slug(&slug);
    let current = active == href;
    let row_slug = slug.clone();
    let start_slug = slug.clone();
    let drop_slug = slug.clone();

    view! {
        <li
            class="flex items-center justify-between gap-[1ch]"
            class=("cursor-grab", move || arranging.get())
            class=(
                "bg-[#0b0e11]",
                move || dragged.get().is_some_and(|held| held == row_slug),
            )
            draggable=move || arranging.get().then_some("true")
            on:dragstart=move |event: DragEvent| {
                if let Some(transfer) = event.data_transfer() {
                    transfer.set_effect_allowed("move");
                    let _ = transfer.set_data("text/plain", start_slug.as_str());
                }
                dragged.set(Some(start_slug.clone()));
            }
            on:dragover=move |event: DragEvent| {
                if arranging.get_untracked() {
                    event.prevent_default();
                }
            }
            on:drop=move |event: DragEvent| {
                event.prevent_default();
                if let Some(held) = dragged.get_untracked()
                    && held != drop_slug
                {
                    order.drag(&held, drop_slug.clone());
                }
                dragged.set(None);
            }
            on:dragend=move |_: DragEvent| dragged.set(None)
        >
            <Show when=move || arranging.get()>
                <span class="inline-flex text-[#4c525a]" aria-hidden="true">
                    <GripVertical size=14 />
                </span>
            </Show>
            <A
                href=href
                attr:class=format!("{NAVIGATION_LINK} min-w-0 flex-1")
                attr:aria-current=current.then_some("page")
            >
                <Icon kind=EntryIcon::Project />
                <span class="truncate">{title}</span>
            </A>
            <ProjectMoves place=Place::Rail slug />
        </li>
    }
}

/// The Project's place in the public order, beside the editor's heading.
#[component]
pub fn ProjectPositionStepper(slug: ProjectSlug) -> impl IntoView {
    let portfolio = expect_context::<Portfolio>();
    let placed = slug.clone();
    let position = move || place_in_order(portfolio, &placed);

    view! {
        <div class="flex items-center gap-[1.2ch] border border-[#1e2126] bg-[#050607] px-[.8rem] py-[.45rem] text-xs text-[#8b939d]">
            <span>"Position"</span>
            <b class="font-medium text-white">
                {move || {
                    position()
                        .map_or_else(
                            String::new,
                            |(place, total)| format!("{place} of {total}"),
                        )
                }}
            </b>
            <ProjectMoves place=Place::Heading slug />
        </div>
    }
}

/// The move up and move down buttons for one Project. Each submits on its own,
/// so the pair works before hydration, and each names the Project it moves.
#[component]
fn ProjectMoves(place: Place, slug: ProjectSlug) -> impl IntoView {
    let portfolio = expect_context::<Portfolio>();
    let first = {
        let slug = slug.clone();
        Signal::derive(move || place_in_order(portfolio, &slug).is_none_or(|(place, _)| place <= 1))
    };
    let last = {
        let slug = slug.clone();
        Signal::derive(move || {
            place_in_order(portfolio, &slug).is_none_or(|(place, total)| place >= total)
        })
    };
    let down_slug = slug.clone();

    view! {
        <div class="flex items-center gap-[.1rem]">
            <MoveButton
                place
                slug=slug
                movement=ProjectMove::Up
                disabled=first
                direction="up"
            >
                <ChevronUp size=14 />
            </MoveButton>
            <MoveButton
                place
                slug=down_slug
                movement=ProjectMove::Down
                disabled=last
                direction="down"
            >
                <ChevronDown size=14 />
            </MoveButton>
        </div>
    }
}

#[component]
fn MoveButton(
    place: Place,
    slug: ProjectSlug,
    movement: ProjectMove,
    disabled: Signal<bool>,
    direction: &'static str,
    children: Children,
) -> impl IntoView {
    let order = ProjectOrder::expect();
    let id = control_id(place, &movement, &slug);
    let pressed_id = id.clone();
    let label = format!("Move {slug} {direction}");

    view! {
        <ActionForm action=order.action attr:class="contents">
            <input type="hidden" name="slug" value=slug.to_string() />
            <input type="hidden" name="movement" value=movement.to_string() />
            <button
                type="submit"
                id=id
                class="inline-flex h-6 w-6 items-center justify-center border border-transparent bg-transparent p-0 text-[#767d87] enabled:cursor-pointer enabled:hover:border-[#2b3037] enabled:hover:text-white disabled:cursor-not-allowed disabled:text-[#2f343a]"
                disabled=move || disabled.get()
                aria-label=label
                on:click=move |_| order.pressed.set(Some(pressed_id.clone()))
            >
                {children()}
            </button>
        </ActionForm>
    }
}

/// Where `slug` sits in the public order, one-based, with the number of
/// Projects. Reading the shared portfolio is what makes a move update the
/// position and the disabled ends everywhere at once.
fn place_in_order(portfolio: Portfolio, slug: &ProjectSlug) -> Option<(usize, usize)> {
    let content = portfolio.current();
    let total = content.projects.len();
    content
        .projects
        .iter()
        .position(|project| project.slug == *slug)
        .map(|index| (index.saturating_add(1), total))
}

/// The element identity of one move control.
fn control_id(place: Place, movement: &ProjectMove, slug: &ProjectSlug) -> String {
    let direction = match movement {
        ProjectMove::Up => "up",
        ProjectMove::Down => "down",
        ProjectMove::ToPlaceOf(_) => "place",
    };
    format!("{}-move-{direction}-{slug}", place.as_str())
}

/// The control at the other end of a pair, which is where focus goes when a
/// move leaves the pressed button disabled at an end of the order.
fn opposite(id: &str) -> String {
    if id.contains("-move-up-") {
        id.replace("-move-up-", "-move-down-")
    } else {
        id.replace("-move-down-", "-move-up-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_control_names_its_place_project_and_direction() {
        let slug = crate::test_support::parse::<ProjectSlug>("traxor");

        assert_eq!(
            control_id(Place::Rail, &ProjectMove::Up, &slug),
            "rail-move-up-traxor"
        );
        assert_eq!(
            control_id(Place::Heading, &ProjectMove::Down, &slug),
            "heading-move-down-traxor"
        );
    }

    #[test]
    fn focus_falls_back_to_the_other_end_of_the_pair() {
        assert_eq!(opposite("rail-move-up-traxor"), "rail-move-down-traxor");
        assert_eq!(
            opposite("heading-move-down-traxor"),
            "heading-move-up-traxor"
        );
    }
}
