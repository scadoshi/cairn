//! Confirmation for destructive actions: orangui's `AlertDialog` with a
//! Cancel and a danger confirm, drawn by the app-root dialog host so the dim
//! covers the whole screen.

use dioxus::prelude::*;
use orangui::{AlertDialog, DialogAction};

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
    rsx! {
        AlertDialog {
            open,
            title,
            actions: vec![
                DialogAction::Dismiss("Cancel".into()),
                DialogAction::Confirm {
                    label: confirm_label,
                    danger: true,
                    on_confirm,
                },
            ],
            p { "{body}" }
        }
    }
}
