//! Centered confirmation dialog for destructive actions, zwiper's look.
//!
//! zwiper builds this on dioxus-primitives; here it is a plain overlay and box
//! carrying the same classes, so the styling is shared verbatim without the
//! extra dependency. Tapping the backdrop or Cancel closes it; only the danger
//! button confirms.

use dioxus::prelude::*;

/// A yes/no dialog. `open` is host-owned so the caller keeps the state that
/// drives it.
#[component]
pub fn ConfirmDialog(
    open: Signal<bool>,
    /// Dialog heading.
    title: String,
    /// What the action will do, in plain words.
    body: String,
    /// Label for the destructive button.
    confirm_label: String,
    on_confirm: EventHandler<()>,
) -> Element {
    let mut open = open;
    if !open() {
        return rsx! {};
    }
    rsx! {
        div {
            class: "alert-dialog-overlay",
            "data-state": "open",
            onclick: move |_| open.set(false),
        }
        div { class: "alert-dialog", role: "alertdialog", aria_modal: "true",
            h2 { class: "alert-dialog-title", "{title}" }
            hr { class: "dialog-rule" }
            p { class: "alert-dialog-description", "{body}" }
            hr { class: "dialog-rule" }
            div { class: "alert-dialog-actions",
                button {
                    class: "alert-dialog-cancel",
                    onclick: move |_| open.set(false),
                    "Cancel"
                }
                button {
                    class: "alert-dialog-action-danger",
                    onclick: move |_| {
                        open.set(false);
                        on_confirm.call(());
                    },
                    "{confirm_label}"
                }
            }
        }
    }
}
