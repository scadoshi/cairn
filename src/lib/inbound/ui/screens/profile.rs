//! Profile: theme and dark mode, laid out like zwiper's profile screen with
//! its preferences sheet.

use crate::inbound::ui::components::bottom_sheet::BottomSheet;
use dioxus::prelude::*;
use zwipe_components::{ALLOWED_THEMES, Button, ButtonVariant, ThemeConfig};

/// Themes with adjusted palettes for color-vision deficiency, grouped at the
/// bottom of the picker. Same list zwiper and the site picker use.
const COLORBLIND_THEMES: &[&str] = &["protanopia", "deuteranopia", "tritanopia", "achromatopsia"];

/// Title-cased theme name, with the brand casings title-casing can't produce.
fn display_theme_name(slug: &str) -> String {
    match slug {
        "rose-pine" => return "Rosé Pine".to_string(),
        "vscode" => return "VS Code".to_string(),
        "github" => return "GitHub".to_string(),
        "synthwave-84" => return "Synthwave '84".to_string(),
        "powershell" => return "PowerShell".to_string(),
        "docs-rs" => return "docs.rs".to_string(),
        _ => {}
    }
    slug.split('-')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The profile screen.
#[component]
pub fn Profile() -> Element {
    let mut theme = use_context::<Signal<ThemeConfig>>();
    let mut preferences_open = use_signal(|| false);

    // Dark mode flips live; the App-level effect persists it.
    let toggle_dark = move |_| {
        let prev = theme.read().clone();
        theme.set(ThemeConfig {
            name: prev.name,
            is_dark: !prev.is_dark,
        });
    };

    rsx! {
        div { class: "profile-sections content-enter",
            div { class: "profile-list",
                div { class: "card-header",
                    span { class: "card-title", "Preferences" }
                }
                div { class: "profile-row",
                    span { class: "profile-row-label", "Theme" }
                    div { class: "profile-row-value",
                        span { {display_theme_name(&theme.read().name)} }
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: move |_| preferences_open.set(true),
                            "Change"
                        }
                    }
                }
                div { class: "profile-row",
                    span { class: "profile-row-label", "Dark mode" }
                    div { class: "profile-row-value",
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: toggle_dark,
                            if theme.read().is_dark { "On" } else { "Off" }
                        }
                    }
                }
            }
        }
        PreferencesSheet { open: preferences_open }
    }
}

/// One selectable theme row: name on the left, swatch dots on the right. The
/// dots take their colors from the theme's own class, so colors stay defined
/// only in themes.css.
#[component]
fn ThemeRow(
    theme: String,
    mode: String,
    mut selected: Signal<String>,
    mut live: Signal<ThemeConfig>,
    dark: Signal<bool>,
) -> Element {
    let is_selected = selected() == theme;
    let click_theme = theme.clone();
    rsx! {
        button {
            class: if is_selected { "pref-row selected" } else { "pref-row" },
            onclick: move |_| {
                selected.set(click_theme.clone());
                live.set(ThemeConfig { name: click_theme.clone(), is_dark: dark() });
            },
            div { class: "pref-row-inner",
                span { "{display_theme_name(&theme)}" }
                div { class: "theme-swatches theme-{theme}-{mode}",
                    span { class: "theme-dot", style: "background:var(--bg-primary)" }
                    span { class: "theme-dot", style: "background:var(--text-primary)" }
                    span { class: "theme-dot", style: "background:var(--accent-primary)" }
                    span { class: "theme-dot", style: "background:var(--accent-secondary)" }
                    span { class: "theme-dot", style: "background:var(--accent-tertiary)" }
                    span { class: "theme-dot", style: "background:var(--color-error)" }
                }
            }
        }
    }
}

/// Bottom sheet for picking a theme. Selections live-preview against the
/// whole app; Save keeps it, Back and the backdrop restore what was active
/// when the sheet opened.
#[component]
fn PreferencesSheet(mut open: Signal<bool>) -> Element {
    let mut live = use_context::<Signal<ThemeConfig>>();
    let mut original = use_signal(|| live.peek().clone());
    let mut selected = use_signal(|| live.peek().name.clone());
    let dark = use_signal(|| live.peek().is_dark);

    // Snapshot the active theme each time the sheet opens so every open
    // starts from the live theme.
    use_effect(move || {
        if open() {
            let current = live.peek().clone();
            original.set(current.clone());
            selected.set(current.name);
        }
    });

    let mode = if dark() { "dark" } else { "light" }.to_string();
    let regular = ALLOWED_THEMES
        .iter()
        .copied()
        .filter(|t| !COLORBLIND_THEMES.contains(t));
    let colorblind = ALLOWED_THEMES
        .iter()
        .copied()
        .filter(|t| COLORBLIND_THEMES.contains(t));

    rsx! {
        BottomSheet {
            open,
            title: "Themes",
            on_dismiss: move |()| live.set(original()),
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| {
                        live.set(original());
                        open.set(false);
                    },
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| open.set(false),
                    "Save"
                }
            },
            for t in regular {
                ThemeRow { theme: t.to_string(), mode: mode.clone(), selected, live, dark }
            }
            div { class: "pref-section-label", "Color blind" }
            for t in colorblind {
                ThemeRow { theme: t.to_string(), mode: mode.clone(), selected, live, dark }
            }
        }
    }
}
