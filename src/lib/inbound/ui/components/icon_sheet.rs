//! The App icon sheet: every theme as a row, each with the icon drawn in that
//! theme's colors in the mode the icon is in now (Config's App icon row flips
//! the mode), laid out like orangui's ThemeSheet.

use crate::{
    inbound::ui::{TOAST_NORMAL, TOAST_QUICK, screens::home::LOGO_CAIRN},
    outbound::app_icon,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use orangui::{BottomSheet, Button, ButtonVariant, THEME_GROUPS, ThemeConfig, display_theme_name};

/// One selectable icon row: name on the left, the icon on the right. The tile
/// draws the mark as text in the theme's own class, so it is the same glyph
/// and colors `scripts/icon.py` renders, with nothing bundled to show it.
#[component]
fn IconRow(theme: String, mode: String, mut draft: Signal<String>) -> Element {
    let is_selected = draft() == theme;
    let click_theme = theme.clone();
    rsx! {
        button {
            class: if is_selected { "pref-row selected" } else { "pref-row" },
            onclick: move |_| draft.set(click_theme.clone()),
            div { class: "pref-row-inner",
                span { "{display_theme_name(&theme)}" }
                div { class: "app-icon-tile theme-{theme}-{mode}",
                    pre { "{LOGO_CAIRN.trim_end()}" }
                }
            }
        }
    }
}

/// Bottom sheet for picking the home-screen icon's theme. The pick is held
/// until Save, which switches the icon in its current mode and writes it to
/// `icon`.
#[component]
pub fn IconSheet(
    mut open: Signal<bool>,
    mut icon: Signal<ThemeConfig>,
    hint: Signal<bool>,
) -> Element {
    let toast = use_toast();
    let mut draft = use_signal(|| icon.peek().name.clone());

    // Every open starts from the icon actually showing.
    use_effect(move || {
        if open() {
            let now = app_icon::current();
            draft.set(now.name.clone());
            icon.set(now);
        }
    });

    let discard = use_callback(move |()| {
        if *draft.peek() != icon.peek().name {
            toast.info(
                "App icon unchanged".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
        open.set(false);
    });

    let mode = app_icon::mode(icon().is_dark).to_string();

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
                    disabled: draft() == icon().name,
                    onclick: move |_| {
                        let picked = ThemeConfig {
                            name: draft(),
                            is_dark: icon.peek().is_dark,
                        };
                        match app_icon::set(&picked) {
                            Ok(()) => {
                                toast.success(
                                    format!("App icon: {}", display_theme_name(&picked.name)),
                                    ToastOptions::default().duration(TOAST_NORMAL),
                                );
                                icon.set(picked);
                            }
                            Err(e) => {
                                toast.error(e, ToastOptions::default());
                            }
                        }
                        open.set(false);
                    },
                    "Save"
                }
            },
            for group in THEME_GROUPS {
                div { class: "pref-section-label", "{group.label}" }
                for t in group.themes.iter().copied() {
                    IconRow { theme: t.to_string(), mode: mode.clone(), draft }
                }
            }
        }
    }
}
