//! Confirmation for destructive actions, zwiper's look, drawn by the
//! app-root dialog host so the dim covers the whole screen.

use super::{
    dialog_host::{DialogSpec, use_hosted_dialog},
    navigation::overlay_stack::use_overlay_back,
};
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
    use_overlay_back(open);
    let confirm = use_callback(move |()| on_confirm.call(()));
    use_hosted_dialog(open, move |close| DialogSpec {
        title: title.clone(),
        body: rsx! { p { class: "alert-dialog-description", "{body}" } },
        confirm: Some((confirm_label.clone(), confirm)),
        close,
        owner: 0,
    });
    rsx! {}
}
