//! The screen header, zwiper's: the kit's `NavBar` folded behind its
//! hamburger. A back arrow on the left on every screen but Home, whose left
//! corner is empty, the screen's title and, on screens with a hint dialog,
//! its `?` in the middle, and the hamburger on the right opening the
//! [main menu](super::menu). `header_hide.js` slides the whole bar up as the
//! content scrolls down and back as it scrolls up.

use dioxus::prelude::*;
use orangui::{InfoButton, NavBar, use_overlay_back};

use super::menu::{MenuFooter, MenuLinks, MenuOpen};
use crate::inbound::ui::router::Route;

/// The header bar with the main menu, titled `title`.
///
/// `hint` is the screen's hint opener, from the `ScreenHint` slot; `None`
/// leaves the `?` out. `back` is the screen's own Back, on a screen below
/// Home; without it the left corner is empty.
#[component]
pub fn ScreenHeader(
    title: String,
    hint: Option<Callback<()>>,
    back: Option<Callback<()>>,
) -> Element {
    let route: Route = use_route();
    let mut open = use_context::<MenuOpen>().0;
    // While the menu is open, the OS back intent closes it instead of
    // leaving the screen.
    use_overlay_back(open);

    // A new route closes the menu. The menu's open state is app-wide, so
    // the panel is still open here and the kit's close transition plays
    // over the new screen.
    use_effect(use_reactive!(|route| {
        let _ = route;
        open.set(false);
    }));

    rsx! {
        div { class: "app-nav",
            NavBar {
                open,
                brand: rsx! {
                    if let Some(back) = back {
                        button {
                            class: "nav-brand app-nav-back",
                            aria_label: "Back",
                            onclick: move |_| {
                                open.set(false);
                                back.call(());
                            },
                            BackArrow {}
                        }
                    }
                },
                persistent: rsx! {
                    div { class: "app-nav-title",
                        h2 { class: "app-nav-title-text", "{title}" }
                        if let Some(hint) = hint {
                            InfoButton { onclick: move |_| hint.call(()) }
                        }
                    }
                },
                links: rsx! {
                    MenuLinks { open, route: route.clone() }
                },
                trailing: rsx! {
                    MenuFooter { open, route }
                },
            }
        }
    }
}

/// The back arrow: a left chevron drawn in the text color.
#[component]
fn BackArrow() -> Element {
    rsx! {
        svg {
            class: "app-nav-back-icon",
            view_box: "0 0 24 24",
            xmlns: "http://www.w3.org/2000/svg",
            "aria-hidden": "true",
            path { d: "M15 5 8 12l7 7" }
        }
    }
}
