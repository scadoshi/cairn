//! The main menu behind the header's hamburger: one footer row with the
//! developer's "!", the icons for Home and Settings, the one on view lit,
//! and the About "i". The list above it is empty and hidden.
//!
//! The open state is app-wide ([`MenuOpen`], provided at the root), so a tap
//! on an entry only navigates: the header then folds the panel shut over
//! the new screen.

use dioxus::prelude::*;

use crate::inbound::ui::components::about::{AboutButton, VersionButton};
use crate::inbound::ui::components::action_dock::{COG, DockIcon, HOUSE};
use crate::inbound::ui::router::Route;

/// A screen the menu opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// Home, the counter list.
    Home,
    /// Settings (the `config` screen).
    Config,
}

impl Section {
    /// Every section, in footer order.
    pub const ALL: [Self; 2] = [Self::Home, Self::Config];

    /// The sections the list holds. Every section sits in the footer.
    pub const LISTED: [Self; 0] = [];

    /// The sections the footer holds as icons, left to right.
    pub const FOOTER: [Self; 2] = [Self::Home, Self::Config];

    /// The footer icon's path.
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Home => HOUSE,
            Self::Config => COG,
        }
    }

    /// The entry's label, the title of the screen it opens.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Config => "Settings",
        }
    }

    /// The screen the entry opens.
    pub const fn route(self) -> Route {
        match self {
            Self::Home => Route::Home {},
            Self::Config => Route::Config {},
        }
    }

    /// The section a route sits under, lit in the menu. A counter sits
    /// under Home.
    pub const fn of(route: &Route) -> Self {
        match route {
            Route::Home {} | Route::CounterScreen { .. } => Self::Home,
            Route::Config {} => Self::Config,
        }
    }
}

/// Whether the main menu is open, shared by the header on every screen.
/// Provided once at the app root so the panel outlives the screen it was
/// opened on and its close transition plays over the next one.
#[derive(Debug, Clone, Copy)]
pub struct MenuOpen(pub Signal<bool>);

/// The menu's link rows, for the nav's `links` slot. Each navigates and
/// leaves the menu to the header to close.
#[component]
pub fn MenuLinks(open: Signal<bool>, route: Route) -> Element {
    let current = Section::of(&route);
    rsx! {
        for section in Section::LISTED {
            li { key: "{section.label()}",
                SectionButton { section, current, open, class: "nav-link", icon: false }
            }
        }
    }
}

/// One section's button, lit when it is `current`: its label in the list,
/// its icon in the footer. A tap pushes the section's route; on the screen
/// it opens, it closes the menu instead.
#[component]
fn SectionButton(
    section: Section,
    current: Section,
    mut open: Signal<bool>,
    class: &'static str,
    icon: bool,
) -> Element {
    let route: Route = use_route();
    let nav = use_navigator();
    rsx! {
        button {
            class: if current == section { "{class} active" } else { "{class}" },
            r#type: "button",
            aria_label: section.label(),
            onclick: move |_| {
                if section.route() == route {
                    open.set(false);
                } else {
                    nav.push(section.route());
                }
            },
            if icon {
                DockIcon { path: section.icon() }
            } else {
                "{section.label()}"
            }
        }
    }
}

/// The menu's footer, for the nav's `trailing` slot: the "!" on the left,
/// the Home and Settings icons, then the About "i".
#[component]
pub fn MenuFooter(open: Signal<bool>, route: Route) -> Element {
    let current = Section::of(&route);
    rsx! {
        div { class: "app-nav-footer",
            div { class: "app-nav-footer-group",
                AboutButton {}
                for section in Section::FOOTER {
                    SectionButton {
                        key: "{section.label()}",
                        section,
                        current,
                        open,
                        class: "btn-action app-nav-icon nav-link",
                        icon: true,
                    }
                }
                VersionButton {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_section_lights_up_on_its_own_screen() {
        for section in Section::ALL {
            assert_eq!(Section::of(&section.route()), section, "{section:?}");
        }
    }

    #[test]
    fn the_listed_and_footer_sections_partition_all() {
        let mut split: Vec<Section> = Section::LISTED.into_iter().chain(Section::FOOTER).collect();
        let mut all: Vec<Section> = Section::ALL.to_vec();
        split.sort_by_key(|section| section.label());
        all.sort_by_key(|section| section.label());
        assert_eq!(split, all);
    }

    #[test]
    fn a_counter_sits_under_home() {
        assert_eq!(Section::of(&Route::CounterScreen { id: 1 }), Section::Home);
    }
}
