//! The main menu behind the header's hamburger: the counters as rows in
//! Home's order, the one on view lit, over a footer row with the developer's
//! "!", the icons for Home and Settings, the one on view lit, the About
//! "i", the theme's palette and the mode's sun or moon. With no counters
//! the list is hidden and the menu is the footer alone.
//!
//! The open state is app-wide ([`MenuOpen`], provided at the root), so a tap
//! on an entry only navigates: the header then folds the panel shut over
//! the new screen.

use dioxus::prelude::*;
use orangui::{ModeIcon, PaletteIcon, ThemeConfig};

use crate::domain::counter::CounterId;
use crate::inbound::ui::components::about::{AboutButton, VersionButton};
use crate::inbound::ui::components::action_dock::{COG, DockIcon, HOUSE};
use crate::inbound::ui::components::theme_sheet::{self, ThemeSheetOpen};
use crate::inbound::ui::router::Route;
use crate::inbound::ui::{self, StoreVersion, use_prefs, use_store};

/// A screen the menu opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// Home, the counter list.
    Home,
    /// Settings (the `config` screen).
    Config,
}

impl Section {
    /// Every section, in the footer's order, left to right.
    pub const ALL: [Self; 2] = [Self::Home, Self::Config];

    /// The footer icon's path.
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Home => HOUSE,
            Self::Config => COG,
        }
    }

    /// The icon's label, the title of the screen it opens.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Config => "Settings",
        }
    }

    /// The screen the icon opens.
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

/// The menu's link rows, for the nav's `links` slot: every counter in
/// Home's order, or one error row when the store cannot list them. Each
/// navigates and leaves the menu to the header to close.
#[component]
pub fn MenuLinks(open: Signal<bool>, route: Route) -> Element {
    let store = use_store();
    let prefs = use_prefs();
    // Read so a new, renamed or deleted counter re-renders the rows.
    let _ = use_context::<StoreVersion>().0.read();
    let counters = match store.list_counters() {
        Ok(counters) => ui::ordered(&counters, &store, prefs().counter_order),
        Err(e) => {
            return rsx! {
                li { class: "message-error", "{e}" }
            };
        }
    };
    let open_id = match route {
        Route::CounterScreen { id } => Some(CounterId(id)),
        Route::Home {} | Route::Config {} => None,
    };
    rsx! {
        for counter in counters {
            li { key: "counter-{counter.id}",
                CounterRow { id: counter.id, name: counter.name.to_string(), lit: open_id == Some(counter.id), open }
            }
        }
    }
}

/// One counter's row, lit while its screen is on view. A tap pushes the
/// counter's screen; on that screen it closes the menu instead.
#[component]
fn CounterRow(id: CounterId, name: String, lit: bool, mut open: Signal<bool>) -> Element {
    let route: Route = use_route();
    let nav = use_navigator();
    rsx! {
        button {
            class: if lit { "nav-link app-nav-counter active" } else { "nav-link app-nav-counter" },
            r#type: "button",
            onclick: move |_| {
                let target = Route::CounterScreen { id: id.0 };
                if target == route {
                    open.set(false);
                } else {
                    nav.push(target);
                }
            },
            span { class: "app-nav-counter-name", "{name}" }
        }
    }
}

/// One section's footer icon, lit when it is `current`. A tap pushes the
/// section's route; on the screen it opens, it closes the menu instead.
#[component]
fn SectionButton(section: Section, current: Section, mut open: Signal<bool>) -> Element {
    let route: Route = use_route();
    let nav = use_navigator();
    rsx! {
        button {
            class: if current == section { "btn-action nav-icon nav-link active" } else { "btn-action nav-icon nav-link" },
            r#type: "button",
            aria_label: section.label(),
            onclick: move |_| {
                if section.route() == route {
                    open.set(false);
                } else {
                    nav.push(section.route());
                }
            },
            DockIcon { path: section.icon() }
        }
    }
}

/// The menu's footer, for the nav's `trailing` slot: the "!" on the left,
/// the Home and Settings icons, the About "i", then the palette, which
/// folds the menu and raises the theme sheet, and the sun or moon naming
/// the mode in use, which flips Dark/Light and saves at once.
#[component]
pub fn MenuFooter(open: Signal<bool>, route: Route) -> Element {
    let current = Section::of(&route);
    let theme = use_context::<Signal<ThemeConfig>>();
    let mut sheet_open = use_context::<ThemeSheetOpen>().0;
    let toggle_dark = theme_sheet::use_toggle_dark(None);
    let mut open = open;
    let is_dark = theme.read().is_dark;
    rsx! {
        div { class: "nav-trailing-row",
            div { class: "nav-trailing-group",
                AboutButton {}
                for section in Section::ALL {
                    SectionButton { key: "{section.label()}", section, current, open }
                }
                VersionButton {}
                button {
                    class: "btn-action nav-icon",
                    r#type: "button",
                    aria_label: "Theme",
                    onclick: move |_| {
                        open.set(false);
                        sheet_open.set(true);
                    },
                    PaletteIcon {}
                }
                button {
                    class: "btn-action nav-icon",
                    r#type: "button",
                    aria_label: if is_dark { "Switch to light" } else { "Switch to dark" },
                    onclick: move |_| toggle_dark.call(()),
                    ModeIcon { dark: is_dark }
                }
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
    fn every_section_has_its_own_label() {
        let mut labels: Vec<&str> = Section::ALL.iter().map(|s| s.label()).collect();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), Section::ALL.len());
    }

    #[test]
    fn a_counter_sits_under_home() {
        assert_eq!(Section::of(&Route::CounterScreen { id: 1 }), Section::Home);
    }
}
