//! Config: everything the person can set, grouped, laid out like zwiper's
//! profile screen with its preferences sheet.

use crate::{
    domain::{counter::csv, preferences::Preferences},
    inbound::ui::{
        components::bottom_sheet::BottomSheet, router::Route, use_date_format, use_prefs, use_store,
    },
    outbound::paths,
};
use chrono::Weekday;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_components::{ALLOWED_THEMES, ActionBar, Button, ButtonVariant, Chip, ThemeConfig};

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

/// The config screen.
#[component]
pub fn Config() -> Element {
    let mut theme = use_context::<Signal<ThemeConfig>>();
    let nav = use_navigator();
    let store = use_store();
    let mut preferences_open = use_signal(|| false);
    let mut notice = use_signal(|| None::<String>);
    let toast = use_toast();
    let mut date_format = use_date_format();
    let mut prefs = use_prefs();
    let mut rest_open = use_signal(|| false);
    let rest_count = (0..7).filter(|i| prefs().rest_days & (1 << i) != 0).count();
    let rest_label = if rest_count == 0 {
        "None".to_string()
    } else {
        rest_count.to_string()
    };

    // One CSV per counter, day and count, into the platform Downloads folder.
    let export = move |_| {
        let result = store
            .list_counters()
            .map_err(|e| e.to_string())
            .and_then(|counters| {
                let dir = paths::exports().map_err(|e| e.to_string())?;
                let mut written = 0usize;
                for c in counters {
                    let entries = store.entries(c.id).map_err(|e| e.to_string())?;
                    let path = dir.join(format!("odo-{}.csv", c.name.slug()));
                    std::fs::write(&path, csv::render(&entries)).map_err(|e| e.to_string())?;
                    written += 1;
                }
                Ok((written, dir))
            });
        match result {
            Ok((n, dir)) => {
                toast.success(
                    format!("Exported {n} files"),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
                notice.set(Some(format!("Written to {}", dir.display())));
            }
            Err(e) => {
                toast.error("Export failed".to_string(), ToastOptions::default());
                notice.set(Some(e));
            }
        }
    };

    // Dark mode flips live; the App-level effect persists it.
    let toggle_dark = move |_| {
        let prev = theme.read().clone();
        theme.set(ThemeConfig {
            name: prev.name,
            is_dark: !prev.is_dark,
        });
    };

    rsx! {
        div { class: "screen-content",
        div { class: "profile-sections content-enter",
            div { class: "profile-list",
                div { class: "card-header",
                    span { class: "card-title", "Appearance" }
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
                div { class: "profile-row",
                    span { class: "profile-row-label", "Dates" }
                    div { class: "profile-row-value",
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: move |_| {
                                let next = date_format().next();
                                date_format.set(next);
                                toast.success(
                                    format!("Dates as {}", next.label()),
                                    ToastOptions::default().duration(Duration::from_millis(1200)),
                                );
                            },
                            "{date_format().label()}"
                        }
                    }
                }
            }
            div { class: "profile-list",
                div { class: "card-header",
                    span { class: "card-title", "Counting" }
                }
                div { class: "profile-row",
                    span { class: "profile-row-label", "Day starts" }
                    div { class: "profile-row-value",
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: move |_| prefs.with_mut(|q| {
                                let i = Preferences::ROLLOVER_HOURS.iter().position(|h| *h == q.rollover_hour).unwrap_or(0);
                                q.rollover_hour = Preferences::ROLLOVER_HOURS
                                    .get((i + 1) % Preferences::ROLLOVER_HOURS.len())
                                    .copied()
                                    .unwrap_or(0);
                            }),
                            {rollover_label(prefs().rollover_hour)}
                        }
                    }
                }
                div { class: "profile-row",
                    span { class: "profile-row-label", "Week starts" }
                    div { class: "profile-row-value",
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: move |_| prefs.with_mut(|q| q.week_start = if q.week_start == Weekday::Mon { Weekday::Sun } else { Weekday::Mon }),
                            if prefs().week_start == Weekday::Mon { "Monday" } else { "Sunday" }
                        }
                    }
                }
                div { class: "profile-row",
                    span { class: "profile-row-label", "Rest days" }
                    div { class: "profile-row-value",
                        span { "{rest_label}" }
                        Button { variant: ButtonVariant::Util, onclick: move |_| rest_open.set(true), "Change" }
                    }
                }
            }
            div { class: "profile-list",
                div { class: "card-header",
                    span { class: "card-title", "Counters" }
                }
                div { class: "profile-row",
                    span { class: "profile-row-label", "Sort counters" }
                    div { class: "profile-row-value",
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: move |_| prefs.with_mut(|q| q.counter_order = q.counter_order.next()),
                            "{prefs().counter_order.label()}"
                        }
                    }
                }
                div { class: "profile-row",
                    span { class: "profile-row-label", "Confirm minus" }
                    div { class: "profile-row-value",
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: move |_| prefs.with_mut(|q| q.confirm_minus = !q.confirm_minus),
                            if prefs().confirm_minus { "On" } else { "Off" }
                        }
                    }
                }
            }
            div { class: "profile-list",
                div { class: "card-header",
                    span { class: "card-title", "Data" }
                    if let Some(n) = notice() {
                    p { class: "pref-note", style: "padding: 0 1rem 1rem;", "{n}" }
                }
            }
                div { class: "profile-row",
                    span { class: "profile-row-label", "Export" }
                    div { class: "profile-row-value",
                        Button { variant: ButtonVariant::Util, onclick: export, "CSV" }
                        if let Some(n) = notice() {
                    p { class: "pref-note", style: "padding: 0 1rem 1rem;", "{n}" }
                }
            }
                    if let Some(n) = notice() {
                    p { class: "pref-note", style: "padding: 0 1rem 1rem;", "{n}" }
                }
            }
                if let Some(n) = notice() {
                    p { class: "pref-note", style: "padding: 0 1rem 1rem;", "{n}" }
                }
            }
        }
        }
        ActionBar {
            Button {
                variant: ButtonVariant::Util,
                onclick: move |_| {
                    if nav.can_go_back() {
                        nav.go_back();
                    } else {
                        nav.push(Route::Home {});
                    }
                },
                "Back"
            }
        }
        PreferencesSheet { open: preferences_open }
        RestDaysSheet { open: rest_open }
    }
}

/// "Midnight", "2am", "4am", "6am".
fn rollover_label(hour: u32) -> String {
    match hour {
        0 => "Midnight".to_string(),
        h => format!("{h}am"),
    }
}

/// Pick the weekdays that don't count. Applies as you tap.
#[component]
fn RestDaysSheet(open: Signal<bool>) -> Element {
    const DAYS: [(Weekday, &str); 7] = [
        (Weekday::Mon, "Monday"),
        (Weekday::Tue, "Tuesday"),
        (Weekday::Wed, "Wednesday"),
        (Weekday::Thu, "Thursday"),
        (Weekday::Fri, "Friday"),
        (Weekday::Sat, "Saturday"),
        (Weekday::Sun, "Sunday"),
    ];
    let mut prefs = use_prefs();
    rsx! {
        BottomSheet {
            open,
            title: "Rest days",
            p { class: "pref-note", "Rest days don't break a streak and don't count against consistency." }
            div { class: "chip-row chip-row-center", style: "flex-wrap: wrap;",
                for (day, name) in DAYS {
                    Chip {
                        selected: prefs().rest_days & (1 << day.num_days_from_monday()) != 0,
                        onclick: move |_| prefs.with_mut(|q| *q = q.toggle_rest(day)),
                        "{name}"
                    }
                }
            }
        }
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
    let toast = use_toast();
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
                    onclick: move |_| {
                        open.set(false);
                        toast.success(
                            "Theme saved".to_string(),
                            ToastOptions::default().duration(Duration::from_millis(1500)),
                        );
                    },
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
