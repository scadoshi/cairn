//! The App icon sheet: Match theme first, then every theme as a row, each with
//! the icon drawn in that theme's colors in the mode the icon is in now
//! (Config's App icon row flips the mode), laid out like orangui's `ThemeSheet`.

use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use orangui::{BottomSheet, Button, ButtonVariant, THEME_GROUPS, ThemeConfig};

use crate::domain::preferences::AppIcon;
use crate::inbound::ui::screens::home::LOGO_CAIRN;
use crate::inbound::ui::{TOAST_NORMAL, TOAST_QUICK, use_prefs};
use crate::outbound::app_icon;

/// One selectable row: label on the left, the icon on the right. The tile
/// draws the mark as text in `tile`'s own class, so it is the same glyph and
/// colors `scripts/icon.py` renders, with nothing bundled to show it. `pick`
/// is the draft it stands for, `None` for Match theme.
#[component]
fn IconRow(
    label: String,
    pick: Option<String>,
    tile: ThemeConfig,
    mut draft: Signal<Option<String>>,
) -> Element {
    let is_selected = draft() == pick;
    let mode = app_icon::mode(tile.is_dark);
    rsx! {
        button {
            class: if is_selected { "pref-row selected" } else { "pref-row" },
            onclick: move |_| draft.set(pick.clone()),
            div { class: "pref-row-inner",
                span { "{label}" }
                div { class: "app-icon-tile theme-{tile.name}-{mode}",
                    pre { "{LOGO_CAIRN.trim_end()}" }
                }
            }
        }
    }
}

/// Bottom sheet for picking the home-screen icon's theme, or Match theme. The
/// pick is held until Save, which switches the icon, writes it to `icon`, and
/// stores whether it follows the theme.
#[component]
pub fn IconSheet(
    mut open: Signal<bool>,
    mut icon: Signal<ThemeConfig>,
    hint: Signal<bool>,
) -> Element {
    let toast = use_toast();
    let theme = use_context::<Signal<ThemeConfig>>();
    let mut prefs = use_prefs();
    // `None` is Match theme.
    let saved = move || (prefs().app_icon == AppIcon::Picked).then(|| icon().name);
    let mut draft = use_signal(saved);

    // Every open starts from the icon actually showing.
    use_effect(move || {
        if open() {
            icon.set(app_icon::current());
            draft.set(saved());
        }
    });

    let discard = use_callback(move |()| {
        if *draft.peek() != saved() {
            toast.info(
                "App icon unchanged".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
        open.set(false);
    });

    let save = move |_| {
        let picked = draft();
        let (next, message) = match &picked {
            None => (theme(), "App icon matches your theme".to_string()),
            Some(name) => (
                ThemeConfig {
                    name: name.clone(),
                    is_dark: icon.peek().is_dark,
                },
                format!("App icon: {}", orangui::display_theme_name(name)),
            ),
        };
        match app_icon::show(&next) {
            Ok(()) => {
                toast.success(message, ToastOptions::default().duration(TOAST_NORMAL));
                icon.set(next);
                prefs.write().app_icon = if picked.is_none() {
                    AppIcon::MatchesTheme
                } else {
                    AppIcon::Picked
                };
            }
            Err(e) => {
                toast.error(e, ToastOptions::default());
            }
        }
        open.set(false);
    };

    let icon_mode = icon().is_dark;

    rsx! {
        BottomSheet {
            open,
            title: "App icon",
            hint,
            on_dismiss: move |()| discard.call(()),
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| discard.call(()),
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Action,
                    disabled: draft() == saved(),
                    onclick: save,
                    "Save"
                }
            },
            IconRow { label: "Match theme", pick: None, tile: theme(), draft }
            for group in THEME_GROUPS {
                div { class: "pref-section-label", "{group.label}" }
                for t in group.themes.iter().copied() {
                    IconRow {
                        label: orangui::display_theme_name(t),
                        pick: Some(t.to_string()),
                        tile: ThemeConfig { name: t.to_string(), is_dark: icon_mode },
                        draft,
                    }
                }
            }
        }
    }
}
