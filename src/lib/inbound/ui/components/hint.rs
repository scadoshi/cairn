//! Hint dialogs and the "?" that opens them, zwiper's teaching-moment
//! pattern. A hint is a centered dialog with a title, a few lines or
//! bullets, and a single Got it. Body text names on-screen buttons with
//! [`HintKey`] so the reader recognizes what to press.

use super::navigation::overlay_stack::use_overlay_back;
use dioxus::prelude::*;

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

/// The dialog: title, body, Got it. Same overlay and box as the delete
/// dialog so the two read as one family; the OS back gesture closes it.
#[component]
pub fn HintDialog(open: Signal<bool>, title: String, children: Element) -> Element {
    let mut open = open;
    use_overlay_back(open);
    if !open() {
        return rsx! {};
    }
    rsx! {
        div {
            class: "alert-dialog-overlay",
            "data-state": "open",
            onclick: move |_| open.set(false),
        }
        div { class: "alert-dialog", role: "dialog", aria_modal: "true",
            h2 { class: "alert-dialog-title", "{title}" }
            hr { class: "dialog-rule" }
            div { class: "alert-dialog-description hint-body", {children} }
            hr { class: "dialog-rule" }
            div { class: "alert-dialog-actions",
                button {
                    class: "alert-dialog-cancel",
                    onclick: move |_| open.set(false),
                    "Got it"
                }
            }
        }
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
