use leptos::{prelude::*, web_sys::DragEvent};

/// Tracks the dragged item and the insertion target for one ordered list.
pub struct DragState<T> {
    dragged: RwSignal<Option<T>>,
    target: RwSignal<Option<usize>>,
}

impl<T> Clone for DragState<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for DragState<T> {}

impl<T> DragState<T>
where
    T: Clone + Eq + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            dragged: RwSignal::new(None),
            target: RwSignal::new(None),
        }
    }

    pub fn is_dragging(&self, item: &T) -> bool {
        self.dragged.get().is_some_and(|dragged| dragged == *item)
    }

    pub fn is_drop_target(&self, target: usize) -> bool {
        self.dragged.get().is_some() && self.target.get().is_some_and(|held| held == target)
    }

    pub fn dragged(&self) -> Option<T> {
        self.dragged.get_untracked()
    }

    pub fn start(&self, item: T, data: &str, event: &DragEvent) {
        if let Some(transfer) = event.data_transfer() {
            transfer.set_effect_allowed("move");
            let _ = transfer.set_data("text/plain", data);
        }
        self.target.set(None);
        self.dragged.set(Some(item));
    }

    pub fn over(&self, target: usize, event: &DragEvent) {
        event.prevent_default();
        if self.dragged.get_untracked().is_some() {
            if let Some(transfer) = event.data_transfer() {
                transfer.set_drop_effect("move");
            }
            self.target.set(Some(target));
        }
    }

    pub fn clear(&self) {
        self.target.set(None);
        self.dragged.set(None);
    }
}

/// The one-pixel insertion line shared by every draggable ordered list.
#[component]
pub fn DropIndicator<T>(drag: DragState<T>, target: Signal<usize>) -> impl IntoView
where
    T: Clone + Eq + Send + Sync + 'static,
{
    view! {
        <span
            class="pointer-events-none absolute inset-x-0 top-[-1px] z-10 h-px bg-[#e2a340]"
            class=("hidden", move || !drag.is_drop_target(target.get()))
            aria-hidden="true"
        ></span>
    }
}
