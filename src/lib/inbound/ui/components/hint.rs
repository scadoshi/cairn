//! Hint dialogs and the "?" that opens them, zwiper's teaching-moment
//! pattern. A hint is a centered dialog with a title, a few lines or
//! bullets, and a single Got it. The "?" and the lines inside come from
//! zwipe-components (`InfoButton`, `HintLine`, `HintBullets`, `HintBullet`,
//! `HintKey`); the dialog, the screen's header hint and [`HintChip`] are
//! cairn's.

use super::dialog_host::{DialogSpec, use_hosted_dialog};
use dioxus::prelude::*;
use zwipe_components::use_overlay_back;

/// The screen's own hint, published upward so the shell's header can offer
/// it. The header is drawn above the router, so it cannot reach into the
/// screen for this; the screen hands it up instead, the same way dialogs
/// describe themselves into the dialog host.
#[derive(Clone, Copy)]
pub struct ScreenHint(pub Signal<Option<Callback<()>>>);

/// Offer this screen's hint in the header, for as long as the screen is on
/// it.
pub fn use_screen_hint(open: Signal<bool>) {
    let slot = use_context::<ScreenHint>();
    let opener = use_callback(move |()| {
        let mut open = open;
        open.set(true);
    });
    use_drop(move || {
        let mut slot = slot.0;
        slot.set(None);
    });
    use_effect(move || {
        let mut slot = slot.0;
        slot.set(Some(opener));
    });
}

/// The dialog: title, body, Got it. Drawn by the app-root host so the dim
/// covers the whole screen; the OS back gesture closes it.
#[component]
pub fn HintDialog(open: Signal<bool>, title: String, children: Element) -> Element {
    use_overlay_back(open);
    use_hosted_dialog(open, move |close| DialogSpec {
        title: title.clone(),
        body: rsx! { div { class: "alert-dialog-description hint-body", {children.clone()} } },
        confirm: None,
        close,
        link: None,
        owner: 0,
    });
    rsx! {}
}

/// A real tag, dropped into hint text as its own example.
///
/// The same markup the screen uses, so what the hint shows and what the
/// counter shows cannot drift: pass the modifier class ("stat-chip-short"
/// for the red one) rather than restating its colors here.
#[component]
pub fn HintChip(#[props(default = String::new())] class: String, children: Element) -> Element {
    rsx! {
        span { class: "stat-chip hint-chip {class}", {children} }
    }
}
