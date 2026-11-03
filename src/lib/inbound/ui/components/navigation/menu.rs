//! The main menu behind the header's hamburger: Home and Settings, the one
//! on view lit, then a footer row with the About "!".

use dioxus::prelude::*;
use orangui::GridCells;

use crate::inbound::ui::components::about::AboutButton;
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
    /// Every section, in menu order.
    pub const ALL: [Self; 2] = [Self::Home, Self::Config];

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

/// The menu's link rows, for the nav's `links` slot. Each closes the menu as
/// it navigates.
#[component]
pub fn MenuLinks(open: Signal<bool>, route: Route) -> Element {
    let current = Section::of(&route);
    rsx! {
        for section in Section::ALL {
            SectionLink { key: "{section.label()}", section, current, open }
        }
    }
}

/// One section's row, lit when it is `current`.
#[component]
fn SectionLink(section: Section, current: Section, mut open: Signal<bool>) -> Element {
    rsx! {
        li {
            Link {
                class: if current == section { "nav-link active" } else { "nav-link" },
                to: section.route(),
                onclick: move |_| open.set(false),
                "{section.label()}"
            }
        }
    }
}

/// The menu's footer, for the nav's `trailing` slot: the About "!" on the
/// left. It also carries the panel's still copy of the grid backdrop, which
/// sits under the whole menu as the bottom sheet's does.
#[component]
pub fn MenuFooter() -> Element {
    rsx! {
        GridCells { still: true }
        div { class: "app-nav-footer",
            AboutButton {}
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
    fn a_counter_sits_under_home() {
        assert_eq!(Section::of(&Route::CounterScreen { id: 1 }), Section::Home);
    }
}
