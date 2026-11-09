//! Config: everything the person can set, grouped, laid out like zwiper's
//! profile screen with its preferences sheet.

use std::collections::VecDeque;
use std::path::PathBuf;

use chrono::{Local, Weekday};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, Toasts, use_toast};
use orangui::icons::{self, DockIcon, EditIcon};
use orangui::{
    ActionDock, AlertDialog, BottomSheet, Button, ButtonVariant, Chip, DarkModeButton,
    DialogAction, DockAction, DockPlace, HintBullet, HintBullets, HintDialog, HintKey, HintLine,
    InfoButton, ModeIcon, PaletteIcon, ThemeConfig, use_screen_hint,
};

use crate::domain::counter::backup::{self, Backup, OnConflict};
use crate::domain::counter::{CounterStore, csv};
use crate::domain::preferences::{self, AppIcon, Celebration, Logo, Preferences};
use crate::inbound::ui::components::celebration::{self, CelebrationHost};
use crate::inbound::ui::components::icon_sheet::IconSheet;
use crate::inbound::ui::components::theme_sheet::{self, AppThemeSheet};
use crate::inbound::ui::{
    self, SharedStore, TOAST_NORMAL, TOAST_QUICK, bump_store_version, sentence_case,
    use_date_format, use_prefs, use_store,
};
use crate::outbound::{app_icon, paths};
use crate::screens::config::{
    Action, CONFLICT_BODY, Effect, PendingImport, State, Step, Topic, conflict_names,
    conflict_title, rest_label, rollover_label,
};
use crate::screens::draft::{ANIMATION_UNCHANGED, Draft, MARK_UNCHANGED, REST_DAYS_UNCHANGED};

/// The config screen.
#[component]
pub fn Config() -> Element {
    let theme = use_context::<Signal<ThemeConfig>>();
    let store = use_store();
    let mut preferences_open = use_signal(|| false);
    let toast = use_toast();
    let mut date_format = use_date_format();
    let prefs = use_prefs();
    let mut rest_open = use_signal(|| false);
    let mut celebration_open = use_signal(|| false);
    let mut mark_open = use_signal(|| false);
    let mut icon_open = use_signal(|| false);
    let mut icon = use_signal(app_icon::current);
    let mut hint_open = use_signal(|| false);
    let screen_hint_open = use_signal(|| false);
    use_screen_hint(screen_hint_open);
    let conflict_open = use_signal(|| false);
    let driver = Driver {
        state: use_signal(State::default),
        store: use_hook(|| CopyValue::new(store)),
        toasts: toast,
        prefs,
        conflict_open,
    };
    let step = move |which: Step| step_pref(driver, which);
    let (notice, topic, conflicts) = {
        let model = driver.state.read();
        (
            model.notice().map(str::to_owned),
            model.topic(),
            model
                .pending()
                .map(|p| p.conflicts.clone())
                .unwrap_or_default(),
        )
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
                        InfoButton { onclick: move |_| { driver.dispatch(Action::Topic(Topic::Theme)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        span { {orangui::display_theme_name(&theme.read().name)} }
                        DarkModeButton { is_dark: theme.read().is_dark, onclick: toggle_dark }
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| {
                                driver.dispatch(Action::Topic(Topic::Theme));
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
                        InfoButton { onclick: move |_| { driver.dispatch(Action::Topic(Topic::AppIcon)); hint_open.set(true); } }
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
                                driver.dispatch(Action::Topic(Topic::AppIcon));
                                icon_open.set(true);
                            },
                            "Change"
                        }
                    }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Mark" }
                        InfoButton { onclick: move |_| { driver.dispatch(Action::Topic(Topic::Mark)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        span { {prefs().logo.label()} }
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| {
                                driver.dispatch(Action::Topic(Topic::Mark));
                                mark_open.set(true);
                            },
                            "Change"
                        }
                    }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Dates" }
                        InfoButton { onclick: move |_| { driver.dispatch(Action::Topic(Topic::Dates)); hint_open.set(true); } }
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
                        InfoButton { onclick: move |_| { driver.dispatch(Action::Topic(Topic::DayStarts)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| {
                                step(Step::DayStarts);
                            },
                            {rollover_label(prefs().rollover_hour)}
                        }
                    }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Week starts" }
                        InfoButton { onclick: move |_| { driver.dispatch(Action::Topic(Topic::WeekStarts)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| {
                                step(Step::WeekStarts);
                            },
                            if prefs().week_start == Weekday::Mon { "Monday" } else { "Sunday" }
                        }
                    }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Rest days" }
                        InfoButton { onclick: move |_| { driver.dispatch(Action::Topic(Topic::RestDays)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        span { {rest_label(prefs().rest_days)} }
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| {
                                driver.dispatch(Action::Topic(Topic::RestDays));
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
                        InfoButton { onclick: move |_| { driver.dispatch(Action::Topic(Topic::SortCounters)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| {
                                step(Step::SortCounters);
                            },
                            "{prefs().counter_order.label()}"
                        }
                    }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Goal animation" }
                        InfoButton { onclick: move |_| { driver.dispatch(Action::Topic(Topic::Celebration)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        span { {prefs().celebration.label()} }
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| {
                                driver.dispatch(Action::Topic(Topic::Celebration));
                                celebration_open.set(true);
                            },
                            "Change"
                        }
                    }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Confirm minus" }
                        InfoButton { onclick: move |_| { driver.dispatch(Action::Topic(Topic::ConfirmMinus)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        Button {
                            variant: ButtonVariant::Action,
                            onclick: move |_| {
                                step(Step::ConfirmMinus);
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
                        InfoButton { onclick: move |_| { driver.dispatch(Action::Topic(Topic::Backup)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        Button { variant: ButtonVariant::Action, onclick: move |_| driver.dispatch(Action::SaveBackup), "Export" }
                        Button { variant: ButtonVariant::Action, onclick: move |_| driver.dispatch(Action::Import), "Import" }
                    }
                }
                div { class: "settings-row",
                    span { class: "row-label-with-hint",
                        span { class: "settings-row-label", "Export" }
                        InfoButton { onclick: move |_| { driver.dispatch(Action::Topic(Topic::Export)); hint_open.set(true); } }
                    }
                    div { class: "settings-row-value",
                        Button { variant: ButtonVariant::Action, onclick: move |_| driver.dispatch(Action::ExportCsv), "CSV" }
                    }
                }
                if let Some(n) = notice {
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
        ConfigHintDialog { open: hint_open, which: topic }
        AlertDialog {
            open: conflict_open,
            title: conflict_title(conflicts.len()),
            actions: vec![
                DialogAction::Dismiss("Cancel".into()),
                DialogAction::Confirm {
                    label: "Keep mine".into(),
                    danger: false,
                    on_confirm: Callback::new(move |()| driver.dispatch(Action::Restore(OnConflict::Keep))),
                },
                DialogAction::Confirm {
                    label: "Replace".into(),
                    danger: true,
                    on_confirm: Callback::new(move |()| driver.dispatch(Action::Restore(OnConflict::Replace))),
                },
            ],
            p { {conflict_names(&conflicts)} }
            p { {CONFLICT_BODY} }
        }
        HintDialog { open: screen_hint_open, title: "Settings",
            HintLine { "Settings only. Nothing here edits your counts" }
            HintLine { "Tap any row's " HintKey { color: "--palette-6", "?" } " to learn what it does" }
        }
    }
}

/// Performs what Settings' model asks: the preferences, the files, the
/// store's restore, the conflict dialog and the toasts.
#[derive(Clone, Copy)]
struct Driver {
    state: Signal<State>,
    store: CopyValue<SharedStore>,
    toasts: Toasts,
    prefs: Signal<Preferences>,
    conflict_open: Signal<bool>,
}

impl Driver {
    /// Runs `action` and every action its effects answer with.
    fn dispatch(self, action: Action) {
        let mut state = self.state;
        let mut queue = VecDeque::from([action]);
        while let Some(action) = queue.pop_front() {
            let effects = state.write().update(action);
            for effect in effects {
                if let Some(answer) = self.perform(effect) {
                    queue.push_back(answer);
                }
            }
        }
    }

    /// Performs one effect, returning the answer when it has one.
    fn perform(self, effect: Effect) -> Option<Action> {
        let Self {
            mut prefs,
            mut conflict_open,
            ..
        } = self;
        match effect {
            Effect::SetPrefs(next) => {
                prefs.set(next);
                None
            }
            Effect::ExportCsv => Some(Action::Exported(export_csv(&**self.store.read()))),
            Effect::SaveBackup => Some(Action::BackupSaved(write_backup(&**self.store.read()))),
            Effect::ReadBackup => Some(Action::BackupRead(read_backup(&**self.store.read()))),
            Effect::AskConflict => {
                conflict_open.set(true);
                None
            }
            Effect::Restore(backup, on_conflict) => Some(Action::Restored(
                self.store
                    .read()
                    .restore(&backup, on_conflict)
                    .map_err(|e| e.to_string()),
            )),
            Effect::Toast(toast) => {
                ui::raise(self.toasts, &toast);
                None
            }
            Effect::StoreChanged => {
                bump_store_version();
                None
            }
        }
    }
}

/// A preference button: `which` steps to its next value.
///
/// The prefs are copied out in a statement of their own: a `peek()` inside
/// the dispatch call keeps its read guard alive to the end of that call,
/// across the driver's write to the same signal.
fn step_pref(driver: Driver, which: Step) {
    let current = *driver.prefs.peek();
    driver.dispatch(Action::Step(which, current));
}

/// Writes one CSV per counter, day and count, into the export folder, and
/// returns how many and the folder, or says what failed.
fn export_csv(store: &(impl CounterStore + ?Sized)) -> Result<(usize, PathBuf), String> {
    let counters = store.list_counters().map_err(|e| e.to_string())?;
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

/// One dialog for the whole screen, its content picked by `which`.
#[component]
fn ConfigHintDialog(open: Signal<bool>, which: Option<Topic>) -> Element {
    let Some(which) = which else {
        return rsx! {};
    };
    match which {
        Topic::Theme => rsx! {
            HintDialog { open, title: "Theme",
                HintLine { "The app's palette. It previews as you tap" }
                HintLine { "The last four are color blind modes" }
                HintLine { HintKey { ModeIcon { dark: false } " light" } " " HintKey { ModeIcon { dark: true } " dark" } " switches between the theme's two palettes" }
            }
        },
        Topic::AppIcon => rsx! {
            HintDialog { open, title: "App icon",
                HintLine { "The icon on your home screen, in any theme's colors" }
                HintLine { HintKey { "Match theme" } " keeps it in your theme as you change it" }
            }
        },
        Topic::Mark => rsx! {
            HintDialog { open, title: "Mark",
                HintLine { "Which letter sits at the top of the home screen" }
                HintLine { "It changes nothing but the drawing" }
            }
        },
        Topic::Celebration => rsx! {
            HintDialog { open, title: "Goal animation",
                HintLine { "What plays when a counter finishes its day, clears the whole day, or passes another tenth of its year" }
                HintLine { "A counter can pick its own from its " HintKey { EditIcon {} } " sheet" }
                HintLine { HintKey { DockIcon { path: icons::PLAY } } " plays the one picked before " HintKey { DockIcon { path: icons::CHECK } } " saves it" }
            }
        },
        Topic::Dates => rsx! {
            HintDialog { open, title: "Dates",
                HintLine { "How every date is written" }
                HintBullets {
                    HintBullet { HintKey { "MM/DD/YY" } " month first" }
                    HintBullet { HintKey { "DD/MM/YY" } " day first" }
                }
            }
        },
        Topic::DayStarts => rsx! {
            HintDialog { open, title: "Day starts",
                HintLine { "When a new day begins. Taps after midnight but before this hour count for the day before" }
                HintLine { "Changing it re-sorts past taps. Totals don't move" }
            }
        },
        Topic::WeekStarts => rsx! {
            HintDialog { open, title: "Week starts",
                HintLine { "Which day opens the week: this week's chart, weekly totals, and the week number" }
                HintBullets {
                    HintBullet { HintKey { "Monday" } " is ISO" }
                    HintBullet { HintKey { "Sunday" } " is the US calendar" }
                }
            }
        },
        Topic::RestDays => rsx! {
            HintDialog { open, title: "Rest days",
                HintLine { "Days you don't train. They don't count against consistency, and a streak steps over them" }
                HintLine { "Logging on one still counts" }
            }
        },
        Topic::SortCounters => rsx! {
            HintDialog { open, title: "Sort counters",
                HintBullets {
                    HintBullet { HintKey { "Created" } " oldest first" }
                    HintBullet { HintKey { "Name" } " A to Z" }
                    HintBullet { HintKey { "Most active" } " this year's total" }
                    HintBullet { HintKey { "Lifetime" } " all-time total" }
                }
            }
        },
        Topic::ConfirmMinus => rsx! {
            HintDialog { open, title: "Confirm minus",
                HintLine { "Every minus button asks first. Worth it when the step is big" }
                HintLine { "Minus only touches today and never goes below zero" }
            }
        },
        Topic::Backup => rsx! {
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
        Topic::Export => rsx! {
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
    use std::cell::Cell;
    use std::sync::Arc;

    use orangui::ToastStack;

    use super::*;
    use crate::outbound::sqlite::SqliteStore;

    thread_local! {
        /// The driver the probe built, for the test to press its buttons.
        static DRIVER: Cell<Option<Driver>> = const { Cell::new(None) };
    }

    /// A Settings driver over an in-memory store, inside the toast stack its
    /// toasts need.
    #[component]
    fn Probe() -> Element {
        let store: SharedStore = Arc::new(SqliteStore::in_memory().unwrap());
        let driver = Driver {
            state: use_signal(State::default),
            store: use_hook(|| CopyValue::new(store)),
            toasts: use_toast(),
            prefs: use_signal(Preferences::default),
            conflict_open: use_signal(|| false),
        };
        DRIVER.with(|d| d.set(Some(driver)));
        rsx! {}
    }

    #[test]
    fn every_preference_button_steps_without_a_borrow_panic() {
        let mut dom = VirtualDom::new(|| rsx! { ToastStack { Probe {} } });
        dom.rebuild_in_place();
        let driver = DRIVER.with(Cell::get).unwrap();
        dom.in_scope(ScopeId::APP, || {
            let before = *driver.prefs.peek();
            for which in [
                Step::DayStarts,
                Step::WeekStarts,
                Step::SortCounters,
                Step::ConfirmMinus,
            ] {
                step_pref(driver, which);
            }
            let after = *driver.prefs.peek();
            assert_ne!(after.rollover_hour, before.rollover_hour);
            assert_ne!(after.week_start, before.week_start);
            assert_ne!(after.counter_order, before.counter_order);
            assert_ne!(after.confirm_minus, before.confirm_minus);
        });
    }
}
