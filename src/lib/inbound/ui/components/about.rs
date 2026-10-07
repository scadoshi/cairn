//! The "!" in the header's left corner, zwiper's support button, opening a
//! dialog about who makes Cairn with a link to their site.

use super::{
    dialog_host::{DialogSpec, use_hosted_dialog},
    hint::HintLine,
    navigation::overlay_stack::use_overlay_back,
};
use dioxus::prelude::*;

const SITE_URL: &str = "https://scottyfermo.com";

/// The corner button and the dialog it opens.
#[component]
pub fn AboutButton() -> Element {
    let mut open = use_signal(|| false);
    use_overlay_back(open);
    use_hosted_dialog(open, move |close| DialogSpec {
        title: "About the developer".to_string(),
        body: rsx! {
            div { class: "alert-dialog-description hint-body",
                HintLine { "Cairn is made by one developer." }
                HintLine { "No account and no server: every count stays on this device." }
                HintLine { "Their other projects, and how to get in touch, are at scottyfermo.com." }
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
