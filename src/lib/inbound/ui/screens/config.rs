//! Config: everything the person can set, grouped, laid out like zwiper's
//! profile screen with its preferences sheet.

use crate::{
    domain::{
        counter::csv,
        preferences::{Celebration, Preferences, rest_mask_has, rest_mask_toggled},
    },
    inbound::ui::{
        TOAST_NORMAL, TOAST_QUICK,
        components::{
            bottom_sheet::BottomSheet,
            hint::{
                HintBullet, HintBullets, HintDialog, HintKey, HintLine, InfoButton, use_screen_hint,
            },
        },
        router::Route,
        use_date_format, use_prefs, use_store,
    },
    outbound::paths,
};
use chrono::Weekday;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::{fmt::Write as _, path::PathBuf};
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
    let mut hint = use_signal(|| None::<ConfigHint>);
    let mut celebration_open = use_signal(|| false);
    let mut hint_open = use_signal(|| false);
    let screen_hint_open = use_signal(|| false);
    use_screen_hint(screen_hint_open);
    let rest_count = (0..7).filter(|i| prefs().rest_days & (1 << i) != 0).count();
    let rest_label = if rest_count == 0 {
        "None".to_string()
    } else {
        rest_count.to_string()
    };

    // One CSV per counter, day and count, into the export folder. Every
    // attempt also appends a line to export.log beside the database: which
    // folder, whether it existed, and how it went. The simulator's sandbox
    // made the folder question hard to answer from the outside.
    let export = move |_| {
        let mut log = String::new();
        let result = store
            .list_counters()
            .map_err(|e| e.to_string())
            .and_then(|counters| {
                let dir = paths::exports().map_err(|e| format!("export folder: {e}"))?;
                let _ = writeln!(
                    log,
                    "home={:?} dir={} exists={}",
                    std::env::var_os("HOME"),
                    dir.display(),
                    dir.is_dir()
                );
                let mut written = 0usize;
                for c in counters {
                    let entries = store.entries(c.id).map_err(|e| e.to_string())?;
                    let path = dir.join(format!("count-{}.csv", c.name.slug()));
                    std::fs::write(&path, csv::render(&entries))
                        .map_err(|e| format!("{}: {e}", path.display()))?;
                    written += 1;
                }
                Ok((written, dir))
            });
        match &result {
            Ok((n, dir)) => {
                let _ = writeln!(log, "ok {n} files in {}", dir.display());
            }
            Err(e) => {
                let _ = writeln!(log, "failed: {e}");
            }
        }
        if let Some(d) = paths::database()
            .ok()
            .and_then(|db| db.parent().map(PathBuf::from))
        {
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(d.join("export.log"))
                .and_then(|mut f| std::io::Write::write_all(&mut f, log.as_bytes()));
        }
        match result {
            Ok((n, dir)) => {
                toast.success(
                    format!("Exported {n} files"),
                    ToastOptions::default().duration(TOAST_NORMAL),
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
    // Every row says what it did, in the same voice and for the same
    // length, because a setting that changes something you cannot see from
    // Config (when a day starts, how counters sort) is otherwise a button
    // that looks like it did nothing.
    let said = use_callback(move |what: String| {
        toast.success(what, ToastOptions::default().duration(TOAST_NORMAL));
    });

    let toggle_dark = move |_| {
        let prev = theme.read().clone();
        let on = !prev.is_dark;
        theme.set(ThemeConfig {
            name: prev.name,
            is_dark: on,
        });
        said.call(format!("Dark mode {}", if on { "on" } else { "off" }));
    };

    rsx! {
        div { class: "screen-content",
        div { class: "profile-sections content-enter",
            div { class: "profile-list",
                div { class: "card-header",
                    span { class: "card-title", "Appearance" }
                }
                div { class: "profile-row",
                    span { class: "row-label-with-hint",
                        span { class: "profile-row-label", "Theme" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::Theme)); hint_open.set(true); } }
                    }
                    div { class: "profile-row-value",
                        span { {display_theme_name(&theme.read().name)} }
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: move |_| {
                                hint.set(Some(ConfigHint::Theme));
                                preferences_open.set(true);
                            },
                            "Change"
                        }
                    }
                }
                div { class: "profile-row",
                    span { class: "row-label-with-hint",
                        span { class: "profile-row-label", "Mark" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::Mark)); hint_open.set(true); } }
                    }
                    div { class: "profile-row-value",
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: move |_| {
                                let next = prefs.peek().logo.next();
                                prefs.with_mut(|q| q.logo = next);
                                said.call(format!("{} mark", next.label()));
                            },
                            "{prefs().logo.label()}"
                        }
                    }
                }
                div { class: "profile-row",
                    span { class: "row-label-with-hint",
                        span { class: "profile-row-label", "Dark mode" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::DarkMode)); hint_open.set(true); } }
                    }
                    div { class: "profile-row-value",
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: toggle_dark,
                            if theme.read().is_dark { "On" } else { "Off" }
                        }
                    }
                }
                div { class: "profile-row",
                    span { class: "row-label-with-hint",
                        span { class: "profile-row-label", "Dates" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::Dates)); hint_open.set(true); } }
                    }
                    div { class: "profile-row-value",
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: move |_| {
                                let next = date_format().next();
                                date_format.set(next);
                                said.call(format!("Dates as {}", next.label()));
                            },
                            "{date_format().label()}"
                        }
                    }
                }
            }
            div { class: "profile-list",
                div { class: "card-header",
                    span { class: "card-title", "When to count" }
                }
                div { class: "profile-row",
                    span { class: "row-label-with-hint",
                        span { class: "profile-row-label", "Day starts" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::DayStarts)); hint_open.set(true); } }
                    }
                    div { class: "profile-row-value",
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: move |_| {
                                let i = Preferences::ROLLOVER_HOURS
                                    .iter()
                                    .position(|h| *h == prefs.peek().rollover_hour)
                                    .unwrap_or(0);
                                let next = Preferences::ROLLOVER_HOURS
                                    .get((i + 1) % Preferences::ROLLOVER_HOURS.len())
                                    .copied()
                                    .unwrap_or(0);
                                prefs.with_mut(|q| q.rollover_hour = next);
                                said.call(format!("Day starts at {}", rollover_label(next).to_lowercase()));
                            },
                            {rollover_label(prefs().rollover_hour)}
                        }
                    }
                }
                div { class: "profile-row",
                    span { class: "row-label-with-hint",
                        span { class: "profile-row-label", "Week starts" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::WeekStarts)); hint_open.set(true); } }
                    }
                    div { class: "profile-row-value",
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: move |_| {
                                let next = if prefs.peek().week_start == Weekday::Mon {
                                    Weekday::Sun
                                } else {
                                    Weekday::Mon
                                };
                                prefs.with_mut(|q| q.week_start = next);
                                said.call(format!("Week starts {next}"));
                            },
                            if prefs().week_start == Weekday::Mon { "Monday" } else { "Sunday" }
                        }
                    }
                }
                div { class: "profile-row",
                    span { class: "row-label-with-hint",
                        span { class: "profile-row-label", "Rest days" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::RestDays)); hint_open.set(true); } }
                    }
                    div { class: "profile-row-value",
                        span { "{rest_label}" }
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: move |_| {
                                hint.set(Some(ConfigHint::RestDays));
                                rest_open.set(true);
                            },
                            "Change"
                        }
                    }
                }
            }
            div { class: "profile-list",
                div { class: "card-header",
                    span { class: "card-title", "Counter behavior" }
                }
                div { class: "profile-row",
                    span { class: "row-label-with-hint",
                        span { class: "profile-row-label", "Sort counters" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::SortCounters)); hint_open.set(true); } }
                    }
                    div { class: "profile-row-value",
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: move |_| {
                                let next = prefs.peek().counter_order.next();
                                prefs.with_mut(|q| q.counter_order = next);
                                said.call(format!("Sorted by {}", next.label().to_lowercase()));
                            },
                            "{prefs().counter_order.label()}"
                        }
                    }
                }
                div { class: "profile-row",
                    span { class: "row-label-with-hint",
                        span { class: "profile-row-label", "Goal animation" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::Celebration)); hint_open.set(true); } }
                    }
                    div { class: "profile-row-value",
                        span { {prefs().celebration.label()} }
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: move |_| {
                                hint.set(Some(ConfigHint::Celebration));
                                celebration_open.set(true);
                            },
                            "Change"
                        }
                    }
                }
                div { class: "profile-row",
                    span { class: "row-label-with-hint",
                        span { class: "profile-row-label", "Confirm minus" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::ConfirmMinus)); hint_open.set(true); } }
                    }
                    div { class: "profile-row-value",
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: move |_| {
                                let on = !prefs.peek().confirm_minus;
                                prefs.with_mut(|q| q.confirm_minus = on);
                                said.call(format!("Confirm minus {}", if on { "on" } else { "off" }));
                            },
                            if prefs().confirm_minus { "On" } else { "Off" }
                        }
                    }
                }
            }
            div { class: "profile-list",
                div { class: "card-header",
                    span { class: "card-title", "Data" }
            }
                div { class: "profile-row",
                    span { class: "row-label-with-hint",
                        span { class: "profile-row-label", "Export" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::Export)); hint_open.set(true); } }
                    }
                    div { class: "profile-row-value",
                        Button { variant: ButtonVariant::Util, onclick: export, "CSV" }
                if let Some(n) = notice() {
                    p { class: "pref-note", style: "padding: 0 1rem 1rem;", "{n}" }
                }
            }
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
        PreferencesSheet { open: preferences_open, hint: hint_open }
        RestDaysSheet { open: rest_open, hint: hint_open }
        CelebrationSheet { open: celebration_open, hint: hint_open }
        ConfigHintDialog { open: hint_open, which: hint() }
        HintDialog { open: screen_hint_open, title: "Config",
            HintLine { "Settings only. Nothing here edits your counts" }
            HintLine { "Tap any row's " HintKey { color: "--accent-primary", "?" } " to learn what it does" }
        }
    }
}

/// Every goal animation as chips, one selected, kept until Save.
///
/// A sheet rather than a button that cycles: nine values behind one button
/// meant eight taps to see the one you wanted, and no way to see what the
/// choices were without tapping through them.
///
/// The pick is held locally and only written on Save, the way the theme
/// sheet works, so tapping through the options to read them does not change
/// the setting.
#[component]
fn CelebrationSheet(mut open: Signal<bool>, hint: Signal<bool>) -> Element {
    let mut prefs = use_prefs();
    let toast = use_toast();
    let saved = prefs().celebration;
    let mut draft = use_signal(|| saved);

    // Every open starts from what is actually set, including one that
    // follows a Back.
    use_effect(move || {
        if open() {
            draft.set(prefs.peek().celebration);
        }
    });

    // Back and a tap outside are the same act, so they say the same thing.
    let discard = use_callback(move |()| {
        if draft.peek().to_owned() != saved {
            toast.info(
                "Animation changes discarded".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
    });

    rsx! {
        BottomSheet {
            open,
            title: "Goal animation",
            hint,
            on_dismiss: move |()| discard.call(()),
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| {
                        // Said out loud, because the pick is still on screen
                        // as the sheet slides away and it otherwise looks
                        // like it took.
                        discard.call(());
                        open.set(false);
                    },
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Util,
                    disabled: draft() == saved,
                    onclick: move |_| {
                        let picked = draft();
                        prefs.with_mut(|q| q.celebration = picked);
                        toast.success(
                            format!("Goal animation: {}", picked.label()),
                            ToastOptions::default().duration(TOAST_NORMAL),
                        );
                        open.set(false);
                    },
                    "Save"
                }
            },
            div { class: "chip-row chip-row-center", style: "flex-wrap: wrap;",
                for c in Celebration::ALL {
                    Chip {
                        selected: draft() == c,
                        onclick: move |_| draft.set(c),
                        "{c.label()}"
                    }
                }
            }
        }
    }
}

/// Which config row's hint is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigHint {
    Theme,
    Mark,
    Celebration,
    DarkMode,
    Dates,
    DayStarts,
    WeekStarts,
    RestDays,
    SortCounters,
    ConfirmMinus,
    Export,
}

/// One dialog for the whole screen, its content picked by `which`.
#[component]
fn ConfigHintDialog(open: Signal<bool>, which: Option<ConfigHint>) -> Element {
    let Some(which) = which else {
        return rsx! {};
    };
    match which {
        ConfigHint::Theme => rsx! {
            HintDialog { open, title: "Theme",
                HintLine { "The app's palette. It previews as you tap" }
                HintLine { "The last four are color blind modes" }
            }
        },
        ConfigHint::Mark => rsx! {
            HintDialog { open, title: "Mark",
                HintLine { "Which letter sits at the top of the home screen" }
                HintBullets {
                    HintBullet { HintKey { color: "--accent-primary", "Cairn" } " is the app's own mark, a C" }
                    HintBullet { HintKey { color: "--accent-primary", "scadoshi" } " is the dev mark this app was built under, an S" }
                    HintBullet { "It changes nothing but the drawing" }
                }
            }
        },
        ConfigHint::Celebration => rsx! {
            HintDialog { open, title: "Goal animation",
                HintLine { "What plays when a counter finishes its day" }
                HintLine { "A counter can pick its own in its " HintKey { color: "--accent-primary", "Edit" } " sheet" }
            }
        },
        ConfigHint::DarkMode => rsx! {
            HintDialog { open, title: "Dark mode",
                HintLine { "Every theme has a light and a dark side. This flips between them" }
            }
        },
        ConfigHint::Dates => rsx! {
            HintDialog { open, title: "Dates",
                HintLine { "How every date is written" }
                HintBullets {
                    HintBullet { HintKey { "MM/DD/YY" } " month first" }
                    HintBullet { HintKey { "DD/MM/YY" } " day first" }
                }
            }
        },
        ConfigHint::DayStarts => rsx! {
            HintDialog { open, title: "Day starts",
                HintLine { "When a new day begins. Taps after midnight but before this hour count for the day before" }
                HintLine { "Changing it re-sorts past taps. Totals don't move" }
            }
        },
        ConfigHint::WeekStarts => rsx! {
            HintDialog { open, title: "Week starts",
                HintLine { "Which day opens the week: this week's chart, weekly totals, and the week number" }
                HintBullets {
                    HintBullet { HintKey { "Monday" } " is ISO" }
                    HintBullet { HintKey { "Sunday" } " is the US calendar" }
                }
            }
        },
        ConfigHint::RestDays => rsx! {
            HintDialog { open, title: "Rest days",
                HintLine { "Days you don't train. They don't count against consistency, and a streak steps over them" }
                HintLine { "Logging on one still counts" }
            }
        },
        ConfigHint::SortCounters => rsx! {
            HintDialog { open, title: "Sort counters",
                HintBullets {
                    HintBullet { HintKey { "Created" } " oldest first" }
                    HintBullet { HintKey { "Name" } " A to Z" }
                    HintBullet { HintKey { "Most active" } " this year's total" }
                    HintBullet { HintKey { "Lifetime" } " all-time total" }
                }
            }
        },
        ConfigHint::ConfirmMinus => rsx! {
            HintDialog { open, title: "Confirm minus",
                HintLine { "Every minus button asks first. Worth it when the step is big" }
                HintLine { "Minus only touches today and never goes below zero" }
            }
        },
        ConfigHint::Export => rsx! {
            HintDialog { open, title: "Export",
                HintLine { HintKey { color: "--accent-primary", "CSV" } " writes one file per counter: day and count" }
                HintBullets {
                    HintBullet { "iPhone: Files app, On My iPhone, Cairn" }
                    HintBullet { "Desktop: Downloads" }
                    HintBullet { "Same names overwrite" }
                }
            }
        },
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
fn RestDaysSheet(mut open: Signal<bool>, hint: Signal<bool>) -> Element {
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
    let toast = use_toast();
    let saved = prefs().rest_days;
    // Held until Save, so toggling a day to see what it does costs nothing.
    // Rest days re-sort consistency and streaks across the whole history,
    // which is too much to happen under a finger that is still deciding.
    let mut draft = use_signal(|| saved);

    use_effect(move || {
        if open() {
            draft.set(prefs.peek().rest_days);
        }
    });

    let discard = use_callback(move |()| {
        if draft.peek().to_owned() != saved {
            toast.info(
                "Rest day changes discarded".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
    });

    rsx! {
        BottomSheet {
            open,
            title: "Rest days",
            hint,
            on_dismiss: move |()| discard.call(()),
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| {
                        discard.call(());
                        open.set(false);
                    },
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Util,
                    disabled: draft() == saved,
                    onclick: move |_| {
                        let picked = draft();
                        prefs.with_mut(|q| q.rest_days = picked);
                        let count = picked.count_ones();
                        toast.success(
                            match count {
                                0 => "No rest days".to_string(),
                                1 => "1 rest day".to_string(),
                                n => format!("{n} rest days"),
                            },
                            ToastOptions::default().duration(TOAST_NORMAL),
                        );
                        open.set(false);
                    },
                    "Save"
                }
            },
            div { class: "chip-row chip-row-center", style: "flex-wrap: wrap;",
                for (day, name) in DAYS {
                    Chip {
                        selected: rest_mask_has(draft(), day),
                        onclick: move |_| draft.with_mut(|d| *d = rest_mask_toggled(*d, day)),
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
fn PreferencesSheet(mut open: Signal<bool>, hint: Signal<bool>) -> Element {
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
            hint,
            on_dismiss: move |()| {
                let changed = selected.peek().to_owned() != original.peek().name
                    || dark.peek().to_owned() != original.peek().is_dark;
                live.set(original());
                if changed {
                    toast.info(
                        "Theme changes discarded".to_string(),
                        ToastOptions::default().duration(TOAST_QUICK),
                    );
                }
            },
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| {
                        let changed = selected() != original().name || dark() != original().is_dark;
                        live.set(original());
                        if changed {
                            toast.info(
                                "Theme changes discarded".to_string(),
                                ToastOptions::default().duration(TOAST_QUICK),
                            );
                        }
                        open.set(false);
                    },
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Util,
                    // Nothing to save until the pick differs from the theme
                    // the sheet opened on. The palette is already live, so
                    // Save is only confirming that it stays.
                    disabled: selected() == original().name && dark() == original().is_dark,
                    onclick: move |_| {
                        open.set(false);
                        toast.success(
                            "Theme saved".to_string(),
                            ToastOptions::default().duration(TOAST_NORMAL),
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
