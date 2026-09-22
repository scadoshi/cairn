//! Hint dialogs and the "?" that opens them, zwiper's teaching-moment
//! pattern. A hint is a centered dialog with a title, a few lines or
//! bullets, and a single Got it. Body text names on-screen buttons with
//! [`HintKey`] so the reader recognizes what to press.

use super::{
    dialog_host::{DialogSpec, use_hosted_dialog},
    navigation::overlay_stack::use_overlay_back,
};
use dioxus::prelude::*;

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

/// The small "?" that sits beside a label and opens a hint.
#[component]
pub fn InfoButton(onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "info-button",
            r#type: "button",
            aria_label: "What is this?",
            onclick: move |evt| {
                evt.stop_propagation();
                onclick.call(evt);
            },
            "?"
        }
    }
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

/// One body line.
#[component]
pub fn HintLine(children: Element) -> Element {
    rsx! {
        p { class: "hint-line", {children} }
    }
}

/// A bulleted list of lines.
#[component]
pub fn HintBullets(children: Element) -> Element {
    rsx! {
        ul { class: "hint-bullets", {children} }
    }
}

/// One bullet.
#[component]
pub fn HintBullet(children: Element) -> Element {
    rsx! {
        li { {children} }
    }
}

/// An inert reference to an on-screen button, styled like one so the
/// reader recognizes it, deliberately not tappable. `color` is a CSS
/// variable name.
#[component]
pub fn HintKey(
    #[props(default = "--accent-tertiary".to_string())] color: String,
    children: Element,
) -> Element {
    rsx! {
        span { class: "hint-key", style: "border-color: var({color}); color: var({color});", {children} }
    }
}
