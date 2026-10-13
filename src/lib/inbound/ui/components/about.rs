//! The "!" in the header's left corner, zwiper's support button, opening a
//! note from the developer with a link to scottyfermo.com.

use dioxus::prelude::*;
use orangui::{AlertDialog, DialogAction, HintLine};

const SITE_URL: &str = "https://scottyfermo.com";

/// The corner button and the dialog it opens.
#[component]
pub fn AboutButton() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        AlertDialog {
            open,
            title: "Hello!",
            actions: vec![
                DialogAction::Dismiss("Got it".into()),
                DialogAction::Link {
                    label: "scottyfermo.com \u{2197}".into(),
                    href: SITE_URL.into(),
                },
            ],
            div { class: "hint-body",
                HintLine { "I made Cairn as a personal tool. My other projects are at the link below." }
            }
        }
        button {
            class: "btn-action page-header-support",
            r#type: "button",
            aria_label: "About the developer",
            onclick: move |_| open.set(true),
            "!"
        }
    }
}
