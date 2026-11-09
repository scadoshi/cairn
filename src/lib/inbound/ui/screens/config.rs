//! Config: everything the person can set, grouped, laid out like zwiper's
//! profile screen with its preferences sheet.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{Local, Weekday};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use orangui::icons::{self, DockIcon, EditIcon};
use orangui::{
    ActionDock, AlertDialog, BottomSheet, Button, ButtonVariant, Chip, DarkModeButton,
    DialogAction, DockAction, DockPlace, HintBullet, HintBullets, HintDialog, HintKey, HintLine,
    InfoButton, ModeIcon, PaletteIcon, ThemeConfig, use_screen_hint,
};

use crate::domain::counter::backup::{self, Backup, OnConflict, Restored};
use crate::domain::counter::{CounterName, CounterStore, csv};
use crate::domain::preferences::{self, AppIcon, Celebration, Logo, Preferences};
use crate::inbound::ui::components::celebration::{self, CelebrationHost};
use crate::inbound::ui::components::icon_sheet::IconSheet;
use crate::inbound::ui::components::theme_sheet::{self, AppThemeSheet};
use crate::inbound::ui::{
    self, TOAST_NORMAL, TOAST_QUICK, bump_store_version, sentence_case, use_date_format, use_prefs,
    use_store,
};
use crate::outbound::{app_icon, paths};
use crate::screens::draft::{ANIMATION_UNCHANGED, Draft, MARK_UNCHANGED, REST_DAYS_UNCHANGED};

/// The config screen.
#[component]
pub fn Config() -> Element {
    let theme = use_context::<Signal<ThemeConfig>>();
    let store = use_store();
    let backup_store = Arc::clone(&store);
    let restore_store = Arc::clone(&store);
    let import_store = Arc::clone(&store);
    let mut preferences_open = use_signal(|| false);
    let mut notice = use_signal(|| None::<String>);
    let toast = use_toast();
    let mut date_format = use_date_format();
    let mut prefs = use_prefs();
    let mut rest_open = use_signal(|| false);
    let mut hint = use_signal(|| None::<ConfigHint>);
    let mut celebration_open = use_signal(|| false);
    let mut mark_open = use_signal(|| false);
    let mut icon_open = use_signal(|| false);
    let mut icon = use_signal(app_icon::current);
    let mut hint_open = use_signal(|| false);
    let screen_hint_open = use_signal(|| false);
    use_screen_hint(screen_hint_open);
    let rest_count = (0..7).filter(|i| prefs().rest_days & (1 << i) != 0).count();
    let rest_label = if rest_count == 0 {
        "None".to_string()
    } else {
        rest_count.to_string()
    };

    // One CSV per counter, day and count, into the export folder.
    let export = move |_| {
        let result = store
            .list_counters()
            .map_err(|e| e.to_string())
            .and_then(|counters| {
                let dir = paths::exports().map_err(|e| format!("export folder: {e}"))?;
                let mut written = 0usize;
                for c in counters {
                    let entries = store.entries(c.id).map_err(|e| e.to_string())?;
                    let path = dir.join(format!("count-{}.csv", c.name.slug()));
                    std::fs::write(&path, csv::render(&entries))
                        .map_err(|e| format!("{}: {e}", path.display()))?;
                    written = written.saturating_add(1);
                }
                Ok((written, dir))
            });
        match result {
            Ok((n, dir)) => {
                toast.success(
                    exported_line(n),
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

    // The full backup: every counter with its settings, days and taps, in
    // one JSON file in the export folder.
    let save_backup = move |_| match write_backup(&*backup_store) {
        Ok(path) => {
            toast.success(
                "Backup saved".to_string(),
                ToastOptions::default().duration(TOAST_NORMAL),
            );
            notice.set(Some(format!("Written to {}", path.display())));
        }
        Err(e) => {
            toast.error("Backup failed".to_string(), ToastOptions::default());
            notice.set(Some(e));
        }
    };

    // A checked backup waiting on the Keep or Replace answer.
    let mut pending = use_signal(|| None::<PendingImport>);
    let mut conflict_open = use_signal(|| false);
    let restore = use_callback(move |on_conflict: OnConflict| {
        let Some(import) = pending.take() else {
            return;
        };
        match restore_store.restore(&import.backup, on_conflict) {
            Ok(done) => {
                bump_store_version();
                toast.success(
                    restored_line(done),
                    ToastOptions::default().duration(TOAST_NORMAL),
                );
                notice.set(Some(format!("Imported from {}", import.file.display())));
            }
            Err(e) => {
                toast.error("Import failed".to_string(), ToastOptions::default());
                notice.set(Some(format!(
                    "{}. Nothing was changed",
                    sentence_case(&e.to_string())
                )));
            }
        }
    });
    // Reads and checks the newest backup before anything is written. With
    // no name in common it imports at once; otherwise it asks.
    let import = move |_| match read_backup(&*import_store) {
        Ok(found) => {
            let ask = !found.conflicts.is_empty();
            pending.set(Some(found));
            if ask {
                conflict_open.set(true);
            } else {
                restore.call(OnConflict::Keep);
            }
        }
        Err(e) => {
            toast.error("Import failed".to_string(), ToastOptions::default());
            notice.set(Some(e));
        }
    };

    // Dark mode flips live; the App-level effect persists it. A row whose
    // button names its new value says nothing more; the home-screen icon,
    // out of view, says what it became.

    let flip_dark =
        theme_sheet::use_toggle_dark(Some(EventHandler::new(move |kept| icon.set(kept))));
    let toggle_dark = move |_| flip_dark.call(());

    // The home-screen icon's own mode, apart from the app's.
    let toggle_icon_dark = move |_| {
        let prev = icon.read().clone();
        let next = ThemeConfig {
            name: prev.name,
            is_dark: !prev.is_dark,
        };
        match app_icon::set(&next) {
            Ok(()) => {
                toast.info(
                    format!("App icon {}", app_icon::mode(next.is_dark)),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
                icon.set(next);
            }
            Err(e) => toast.error(e, ToastOptions::default()),
        }
    };

    rsx! {
        div { class: "screen-content",
        div { class: "settings-sections content-enter",
            div { class: "settings-list",
                div { class: "settings-list-header",
                    span { class: "settings-list-title", "Appearance" }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Theme" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::Theme)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        span { {orangui::display_theme_name(&theme.read().name)} }
                        DarkModeButton { is_dark: theme.read().is_dark, onclick: toggle_dark }
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| {
                                hint.set(Some(ConfigHint::Theme));
                                preferences_open.set(true);
                            },
                            PaletteIcon {}
                            " Change"
                        }
                    }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "App icon" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::AppIcon)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        if prefs().app_icon == AppIcon::MatchesTheme {
                            span { "Matches theme" }
                        } else {
                            span { {orangui::display_theme_name(&icon().name)} }
                            DarkModeButton { is_dark: icon().is_dark, onclick: toggle_icon_dark }
                        }
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| {
                                hint.set(Some(ConfigHint::AppIcon));
                                icon_open.set(true);
                            },
                            "Change"
                        }
                    }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Mark" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::Mark)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        span { {prefs().logo.label()} }
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| {
                                hint.set(Some(ConfigHint::Mark));
                                mark_open.set(true);
                            },
                            "Change"
                        }
                    }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Dates" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::Dates)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| {
                                let next = date_format().next();
                                date_format.set(next);
                            },
                            "{date_format().label()}"
                        }
                    }
                }
            }
            div { class: "settings-list",
                div { class: "settings-list-header",
                    span { class: "settings-list-title", "When to count" }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Day starts" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::DayStarts)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        Button {
                            variant: ButtonVariant::Action,
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
                            },
                            {rollover_label(prefs().rollover_hour)}
                        }
                    }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Week starts" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::WeekStarts)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| {
                                let next = if prefs.peek().week_start == Weekday::Mon {
                                    Weekday::Sun
                                } else {
                                    Weekday::Mon
                                };
                                prefs.with_mut(|q| q.week_start = next);
                            },
                            if prefs().week_start == Weekday::Mon { "Monday" } else { "Sunday" }
                        }
                    }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Rest days" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::RestDays)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        span { "{rest_label}" }
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| {
                                hint.set(Some(ConfigHint::RestDays));
                                rest_open.set(true);
                            },
                            "Change"
                        }
                    }
                }
            }
            div { class: "settings-list",
                div { class: "settings-list-header",
                    span { class: "settings-list-title", "Counter behavior" }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Sort counters" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::SortCounters)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| {
                                let next = prefs.peek().counter_order.next();
                                prefs.with_mut(|q| q.counter_order = next);
                            },
                            "{prefs().counter_order.label()}"
                        }
                    }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Goal animation" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::Celebration)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        span { {prefs().celebration.label()} }
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| {
                                hint.set(Some(ConfigHint::Celebration));
                                celebration_open.set(true);
                            },
                            "Change"
                        }
                    }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Confirm minus" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::ConfirmMinus)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| {
                                let on = !prefs.peek().confirm_minus;
                                prefs.with_mut(|q| q.confirm_minus = on);
                            },
                            if prefs().confirm_minus { "On" } else { "Off" }
                        }
                    }
                }
            }
            div { class: "settings-list",
                div { class: "settings-list-header",
                    span { class: "settings-list-title", "Data" }
            }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Backup" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::Backup)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        Button { variant: ButtonVariant::Action, onclick: save_backup, "Export" }
                        Button { variant: ButtonVariant::Action, onclick: import, "Import" }
                    }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Export" }
                        InfoButton { onclick: move |_| { hint.set(Some(ConfigHint::Export)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        Button { variant: ButtonVariant::Action, onclick: export, "CSV" }
                    }
                }
                if let Some(n) = notice() {
                    p { class: "pref-note export-note", "{n}" }
                }
            }
        }
        }
        AppThemeSheet {
            open: preferences_open,
            hint: Some(hint_open),
            on_kept: move |kept: ThemeConfig| icon.set(kept),
        }
        RestDaysSheet { open: rest_open, hint: hint_open }
        CelebrationSheet { open: celebration_open, hint: hint_open }
        MarkSheet { open: mark_open, hint: hint_open }
        IconSheet { open: icon_open, icon, hint: hint_open }
        ConfigHintDialog { open: hint_open, which: hint() }
        AlertDialog {
            open: conflict_open,
            title: conflict_title(pending.read().as_ref().map_or(0, |p| p.conflicts.len())),
            actions: vec![
                DialogAction::Dismiss("Cancel".into()),
                DialogAction::Confirm {
                    label: "Keep mine".into(),
                    danger: false,
                    on_confirm: Callback::new(move |()| restore.call(OnConflict::Keep)),
                },
                DialogAction::Confirm {
                    label: "Replace".into(),
                    danger: true,
                    on_confirm: Callback::new(move |()| restore.call(OnConflict::Replace)),
                },
            ],
            p { {conflict_names(pending.read().as_ref().map(|p| p.conflicts.as_slice()).unwrap_or_default())} }
            p { "Replace swaps yours, and their history, for the backup's. Keep mine leaves yours and imports the rest." }
        }
        HintDialog { open: screen_hint_open, title: "Settings",
            HintLine { "Settings only. Nothing here edits your counts" }
            HintLine { "Tap any row's " HintKey { color: "--palette-6", "?" } " to learn what it does" }
        }
    }
}

/// The letters to pick a mark from, one selected, kept until Save.
#[component]
fn MarkSheet(mut open: Signal<bool>, hint: Signal<bool>) -> Element {
    let mut prefs = use_prefs();
    let toast = use_toast();
    let mut model = use_signal(|| Draft::new(prefs.peek().logo, MARK_UNCHANGED));

    use_effect(move || {
        if open() {
            model.write().open(prefs.peek().logo);
        }
    });

    let discard = use_callback(move |()| {
        let said = model.peek().back();
        if let Some(said) = said {
            ui::raise(toast, &said);
        }
        open.set(false);
    });

    rsx! {
        BottomSheet {
            open,
            title: "Mark",
            hint,
            on_dismiss: move |()| discard.call(()),
            closable: true,
            div { class: "chip-row chip-row-center", style: "flex-wrap: wrap;",
                for l in Logo::ALL {
                    Chip {
                        selected: *model.read().shown() == l,
                        onclick: move |_| model.write().pick(l),
                        "{l.label()}"
                    }
                }
            }
            // Save as the action dock's checkmark, pinned bottom right.
            ActionDock { place: DockPlace::Sheet,
                DockAction {
                    label: "Save",
                    icon: icons::CHECK,
                    disabled: !model.read().can_save(),
                    onclick: move |()| {
                        let picked = model.peek().save();
                        if let Some(picked) = picked {
                            prefs.with_mut(|q| q.logo = picked);
                            model.write().open(picked);
                        }
                        open.set(false);
                    },
                }
            }
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
    let host = use_context::<CelebrationHost>();
    let mut model = use_signal(|| Draft::new(prefs.peek().celebration, ANIMATION_UNCHANGED));

    // Every open starts from what is actually set, including one that
    // follows a Back.
    use_effect(move || {
        if open() {
            model.write().open(prefs.peek().celebration);
        }
    });

    // Back and a tap outside are the same act, so they say the same thing.
    let discard = use_callback(move |()| {
        let said = model.peek().back();
        if let Some(said) = said {
            ui::raise(toast, &said);
        }
        open.set(false);
    });

    rsx! {
        BottomSheet {
            open,
            title: "Goal animation",
            hint,
            on_dismiss: move |()| discard.call(()),
            closable: true,
            div { class: "chip-row chip-row-center", style: "flex-wrap: wrap;",
                for c in Celebration::ALL {
                    Chip {
                        selected: *model.read().shown() == c,
                        onclick: move |_| model.write().pick(c),
                        "{c.label()}"
                    }
                }
            }
            // Test and Save in the action dock, pinned bottom right.
            ActionDock { place: DockPlace::Sheet,
                // Plays the pick as a goal would, toast and all, before
                // it is saved. Random steps through its rotation on
                // each tap.
                DockAction {
                    label: "Test",
                    icon: icons::PLAY,
                    disabled: *model.read().shown() == Celebration::Off,
                    onclick: move |()| {
                        let picked = *model.peek().shown();
                        if let Some(line) = celebration::celebrate(host, picked) {
                            toast.success(line, ToastOptions::default().duration(TOAST_NORMAL));
                        }
                    },
                }
                span { class: "action-dock-divider" }
                DockAction {
                    label: "Save",
                    icon: icons::CHECK,
                    disabled: !model.read().can_save(),
                    onclick: move |()| {
                        let picked = model.peek().save();
                        if let Some(picked) = picked {
                            prefs.with_mut(|q| q.celebration = picked);
                            model.write().open(picked);
                        }
                        open.set(false);
                    },
                }
            }
        }
    }
}

/// Which config row's hint is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigHint {
    Theme,
    AppIcon,
    Mark,
    Celebration,
    Dates,
    DayStarts,
    WeekStarts,
    RestDays,
    SortCounters,
    ConfirmMinus,
    Backup,
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
                HintLine { HintKey { ModeIcon { dark: false } " light" } " " HintKey { ModeIcon { dark: true } " dark" } " switches between the theme's two palettes" }
            }
        },
        ConfigHint::AppIcon => rsx! {
            HintDialog { open, title: "App icon",
                HintLine { "The icon on your home screen, in any theme's colors" }
                HintLine { HintKey { "Match theme" } " keeps it in your theme as you change it" }
            }
        },
        ConfigHint::Mark => rsx! {
            HintDialog { open, title: "Mark",
                HintLine { "Which letter sits at the top of the home screen" }
                HintLine { "It changes nothing but the drawing" }
            }
        },
        ConfigHint::Celebration => rsx! {
            HintDialog { open, title: "Goal animation",
                HintLine { "What plays when a counter finishes its day, clears the whole day, or passes another tenth of its year" }
                HintLine { "A counter can pick its own from its " HintKey { EditIcon {} } " sheet" }
                HintLine { HintKey { DockIcon { path: icons::PLAY } } " plays the one picked before " HintKey { DockIcon { path: icons::CHECK } } " saves it" }
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
        ConfigHint::Backup => rsx! {
            HintDialog { open, title: "Backup",
                HintLine { HintKey { "Export" } " saves every counter, its settings and every tap in one file" }
                HintLine { HintKey { "Import" } " reads the newest backup there. If a counter's name is already here, it asks to keep yours or replace it" }
                HintBullets {
                    HintBullet { "iPhone: Files app, On My iPhone, Cairn" }
                    HintBullet { "Desktop: Downloads" }
                    HintBullet { "One file a day; the same day overwrites" }
                }
            }
        },
        ConfigHint::Export => rsx! {
            HintDialog { open, title: "Export",
                HintLine { HintKey { "CSV" } " writes one file per counter: day and count" }
                HintBullets {
                    HintBullet { "iPhone: Files app, On My iPhone, Cairn" }
                    HintBullet { "Desktop: Downloads" }
                    HintBullet { "Same names overwrite" }
                }
            }
        },
    }
}

/// A backup read from the export folder and checked, with the names it
/// shares with the store.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingImport {
    backup: Backup,
    conflicts: Vec<CounterName>,
    file: PathBuf,
}

/// Writes the full backup into the export folder and returns its path, or
/// says what failed.
fn write_backup(store: &(impl CounterStore + ?Sized)) -> Result<PathBuf, String> {
    let text = Backup::from_store(store)
        .map_err(|e| e.to_string())?
        .to_json()
        .map_err(|e| e.to_string())?;
    let dir = paths::exports().map_err(|e| format!("export folder: {e}"))?;
    let path = paths::backup_file(&dir, Local::now().date_naive());
    std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

/// Finds the newest backup in the export folder, reads and checks it, and
/// lists the names it shares with the store. Writes nothing.
fn read_backup(store: &(impl CounterStore + ?Sized)) -> Result<PendingImport, String> {
    let dir = paths::exports().map_err(|e| format!("export folder: {e}"))?;
    let file = paths::latest_backup(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .ok_or_else(|| format!("No backup in {}", dir.display()))?;
    let text = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    let backup = backup::parse(&text).map_err(|e| sentence_case(&e.to_string()))?;
    let existing = store.list_counters().map_err(|e| e.to_string())?;
    let conflicts = backup.conflicts(&existing);
    Ok(PendingImport {
        backup,
        conflicts,
        file,
    })
}

/// "Exported 1 file", "Exported 3 files".
fn exported_line(n: usize) -> String {
    if n == 1 {
        "Exported 1 file".to_string()
    } else {
        format!("Exported {n} files")
    }
}

/// "1 counter is already here", "3 counters are already here".
fn conflict_title(n: usize) -> String {
    if n == 1 {
        "1 counter is already here".to_string()
    } else {
        format!("{n} counters are already here")
    }
}

/// The shared names, comma separated.
fn conflict_names(names: &[CounterName]) -> String {
    names
        .iter()
        .map(CounterName::as_str)
        .collect::<Vec<&str>>()
        .join(", ")
}

/// What an import did, as its toast says it.
fn restored_line(done: Restored) -> String {
    let imported = done.added.saturating_add(done.replaced);
    let mut line = match imported {
        1 => "Imported 1 counter".to_string(),
        n => format!("Imported {n} counters"),
    };
    if done.replaced > 0 {
        let _ = write!(line, ", {} replaced", done.replaced);
    }
    if done.kept > 0 {
        let _ = write!(line, ", kept {} of yours", done.kept);
    }
    line
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
    // Held until Save, so toggling a day to see what it does costs nothing.
    // Rest days re-sort consistency and streaks across the whole history,
    // which is too much to happen under a finger that is still deciding.
    let mut model = use_signal(|| Draft::new(prefs.peek().rest_days, REST_DAYS_UNCHANGED));

    use_effect(move || {
        if open() {
            model.write().open(prefs.peek().rest_days);
        }
    });

    let discard = use_callback(move |()| {
        let said = model.peek().back();
        if let Some(said) = said {
            ui::raise(toast, &said);
        }
        open.set(false);
    });

    rsx! {
        BottomSheet {
            open,
            title: "Rest days",
            hint,
            on_dismiss: move |()| discard.call(()),
            closable: true,
            div { class: "chip-row chip-row-center", style: "flex-wrap: wrap;",
                for (day, name) in DAYS {
                    Chip {
                        selected: preferences::rest_mask_has(*model.read().shown(), day),
                        onclick: move |_| {
                            let toggled = preferences::rest_mask_toggled(*model.peek().shown(), day);
                            model.write().pick(toggled);
                        },
                        "{name}"
                    }
                }
            }
            // Save as the action dock's checkmark, pinned bottom right.
            ActionDock { place: DockPlace::Sheet,
                DockAction {
                    label: "Save",
                    icon: icons::CHECK,
                    disabled: !model.read().can_save(),
                    onclick: move |()| {
                        let picked = model.peek().save();
                        if let Some(picked) = picked {
                            prefs.with_mut(|q| q.rest_days = picked);
                            model.write().open(picked);
                        }
                        open.set(false);
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_export_toast_counts_files_in_the_right_number() {
        assert_eq!(exported_line(0), "Exported 0 files");
        assert_eq!(exported_line(1), "Exported 1 file");
        assert_eq!(exported_line(3), "Exported 3 files");
    }
}
