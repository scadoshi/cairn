//! Slide-up bottom sheet, zwiper's pattern.

use super::navigation::overlay_stack::use_overlay_back_action;
use dioxus::prelude::*;
use std::time::Duration;
use zwipe_components::{ActionBar, Button, ButtonVariant};

/// A bottom sheet with backdrop, title, content slot, and footer.
///
/// `footer` overrides the default single "Close" button (e.g. a Back/Save
/// pair). `on_dismiss` fires when the backdrop is tapped, before the sheet
/// closes, so a sheet that live-previews something can revert it.
#[component]
pub fn BottomSheet(
    mut open: Signal<bool>,
    title: String,
    children: Element,
    footer: Option<Element>,
    on_dismiss: Option<EventHandler<()>>,
    /// A hint's open signal. Given one, the sheet's header carries the same
    /// "?" the screen headers do; without one, the corner stays empty.
    hint: Option<Signal<bool>>,
) -> Element {
    // The OS back gesture closes the sheet the way a backdrop tap does:
    // `on_dismiss` first (the theme sheet relies on it to revert), then close.
    let dismiss = use_callback(move |()| {
        let mut open = open;
        if let Some(h) = on_dismiss {
            h.call(());
        }
        open.set(false);
    });
    use_overlay_back_action(open.into(), dismiss);

    // First render carries `transition: none` via the premount class, dropped
    // once mounted. Without it iOS WebKit replays the transform transition on
    // insert and a freshly mounted sheet visibly slides away. The flag has to
    // flip after WebKit's first post-insert paint, so it waits a couple frames.
    let mut mounted = use_signal(|| false);
    use_effect(move || {
        spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            mounted.set(true);
        });
    });

    rsx! {
        div {
            class: if open() { "modal-backdrop show" } else { "modal-backdrop" },
            onclick: move |_| {
                if let Some(h) = on_dismiss { h.call(()); }
                open.set(false);
            },
        }
        div {
            class: if open() {
                "bottom-sheet show"
            } else if mounted() {
                "bottom-sheet"
            } else {
                "bottom-sheet bottom-sheet-premount"
            },
            div { class: "modal-header",
                span { class: "modal-title", "{title}" }
                if let Some(mut hint) = hint {
                    Button {
                        variant: ButtonVariant::Util,
                        class: "page-header-corner",
                        onclick: move |_| hint.set(true),
                        "?"
                    }
                }
            }
            div { class: "modal-content",
                div { class: "flex-col", style: "gap: 0.5rem;",
                    {children}
                }
            }
            ActionBar {
                if let Some(f) = footer {
                    {f}
                } else {
                    Button {
                        variant: ButtonVariant::Util,
                        onclick: move |_| open.set(false),
                        "Close"
                    }
                }
            }
        }
    }
}
