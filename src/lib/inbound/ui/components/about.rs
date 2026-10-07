//! The "!" in the header's left corner, zwiper's support button, opening a
//! note from the developer with a link to scottyfermo.com.

use super::dialog_host::{DialogSpec, use_hosted_dialog};
use dioxus::prelude::*;
use zwipe_components::{HintLine, use_overlay_back};

const SITE_URL: &str = "https://scottyfermo.com";

/// The corner button and the dialog it opens.
#[component]
pub fn AboutButton() -> Element {
    let mut open = use_signal(|| false);
    use_overlay_back(open);
    use_hosted_dialog(open, move |close| DialogSpec {
        title: "Hello!".to_string(),
        body: rsx! {
            div { class: "alert-dialog-description hint-body",
                HintLine { "I made Cairn as a personal tool. My other projects are at the link below." }
            }
        },
        confirm: None,
        link: Some(("scottyfermo.com \u{2197}", SITE_URL)),
        close,
        owner: 0,
    });

    rsx! {
        button {
            class: "util-btn page-header-support",
            r#type: "button",
            aria_label: "About the developer",
            onclick: move |_| open.set(true),
            "!"
        }
    }
}
