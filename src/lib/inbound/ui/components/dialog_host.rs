//! One place at the app root that draws dialogs.
//!
//! A `position: fixed` overlay inside a screen's scroll container gets
//! trapped by it in this WebView, so the dim stops at the content column.
//! zwiper escapes that by mounting its hint dialog beside the router. Same
//! here: dialogs describe themselves into a shared slot and [`DialogHostView`]
//! draws whatever is in it, above everything.

use dioxus::prelude::*;

/// What to draw: a title, a body, an optional destructive action, and how
/// to close.
#[derive(Clone)]
pub struct DialogSpec {
    /// Heading.
    pub title: String,
    /// Body content.
    pub body: Element,
    /// Label and handler for a destructive confirm; `None` for a plain
    /// Got it dialog.
    pub confirm: Option<(String, Callback<()>)>,
    /// Closes the dialog on the owner's side.
    pub close: Callback<()>,
    /// Which owner set it, so a stale owner can't clear a newer one.
    pub owner: u64,
}

/// The slot. Provided by the app root.
#[derive(Clone, Copy)]
pub struct DialogHost(pub Signal<Option<DialogSpec>>);

/// Draws the current dialog, if any.
#[component]
pub fn DialogHostView() -> Element {
    let host = use_context::<DialogHost>();
    let Some(spec) = host.0.read().clone() else {
        return rsx! {};
    };
    let close = spec.close;
    let mut slot = host.0;
    // Clearing here as well as through the owner's `open` signal: a confirm
    // that navigates away unmounts the owner before its effect can run, and
    // the dialog would otherwise stay on screen holding a callback into a
    // scope that no longer exists.
    let dismiss = use_callback(move |()| {
        slot.set(None);
        close.call(());
    });
    rsx! {
        div {
            class: "alert-dialog-overlay",
            "data-state": "open",
            onclick: move |_| dismiss.call(()),
        }
        div { class: "alert-dialog", role: "dialog", aria_modal: "true",
            h2 { class: "alert-dialog-title", "{spec.title}" }
            hr { class: "dialog-rule" }
            {spec.body}
            hr { class: "dialog-rule" }
            div { class: "alert-dialog-actions",
                button {
                    class: "alert-dialog-cancel",
                    onclick: move |_| dismiss.call(()),
                    if spec.confirm.is_some() { "Cancel" } else { "Got it" }
                }
                if let Some((label, confirm)) = spec.confirm {
                    button {
                        class: "alert-dialog-action-danger",
                        onclick: move |_| {
                            dismiss.call(());
                            confirm.call(());
                        },
                        "{label}"
                    }
                }
            }
        }
    }
}

static NEXT_OWNER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Keeps the host's slot in step with an `open` signal: sets the spec when
/// it opens, clears it when it closes, and never clears someone else's.
pub fn use_hosted_dialog(open: Signal<bool>, build: impl Fn(Callback<()>) -> DialogSpec + 'static) {
    let host = use_context::<DialogHost>();
    let owner = use_hook(|| NEXT_OWNER.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
    let close = use_callback(move |()| {
        let mut open = open;
        open.set(false);
    });
    use_drop(move || {
        let mut slot = host.0;
        if slot.peek().as_ref().is_some_and(|s| s.owner == owner) {
            slot.set(None);
        }
    });
    use_effect(move || {
        let mut slot = host.0;
        if open() {
            let mut spec = build(close);
            spec.owner = owner;
            slot.set(Some(spec));
        } else if slot.peek().as_ref().is_some_and(|s| s.owner == owner) {
            slot.set(None);
        }
    });
}
