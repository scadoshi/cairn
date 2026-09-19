//! Slide-up bottom sheet, zwiper's pattern.

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
) -> Element {
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
