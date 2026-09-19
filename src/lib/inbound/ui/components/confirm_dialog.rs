//! Confirmation for destructive actions, as a bottom sheet.
//!
//! Deleting a counter takes its entries with it and there is no undo, so it
//! goes through here. The backdrop and Back both cancel; only the danger
//! button resolves.

use super::bottom_sheet::BottomSheet;
use dioxus::prelude::*;
use zwipe_components::{Button, ButtonVariant};

/// A yes/no sheet. `open` is host-owned so the caller keeps the state that
/// drives it.
#[component]
pub fn ConfirmDialog(
    open: Signal<bool>,
    /// Sheet heading.
    title: String,
    /// What the action will do, in plain words.
    body: String,
    /// Label for the destructive button.
    confirm_label: String,
    on_confirm: EventHandler<()>,
) -> Element {
    let mut open = open;
    rsx! {
        BottomSheet {
            open,
            title,
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| open.set(false),
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Util,
                    danger: true,
                    onclick: move |_| {
                        open.set(false);
                        on_confirm.call(());
                    },
                    "{confirm_label}"
                }
            },
            p { class: "pref-note", "{body}" }
        }
    }
}
