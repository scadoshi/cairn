//! The one screen you land on: the mark and today's date together at the
//! top, a quote, then the counters.
//!
//! This used to be two screens, a landing page and a list. Tapping through to
//! reach the counters cost a tap on the thing the app exists for, and the
//! landing page had nothing on it you could act on.
//!
//! Both sheets hang off the screen rather than off a card, because a fixed
//! overlay inside a card is clipped by the card's rounded corners.

use crate::{
    domain::{
        counter::{Counter, DayCount, Goal, format::compact, stats},
        preferences::Logo,
    },
    inbound::ui::{
        TOAST_NORMAL, bump_store_version,
        components::{
            bottom_sheet::BottomSheet,
            counter_form::{CounterForm, CounterFormState, EditSheet},
            counter_list::CounterList,
            hint::{
                HintBullet, HintBullets, HintChip, HintDialog, HintKey, HintLine, use_screen_hint,
            },
            quote_card::QuoteCard,
            tile::Tile,
        },
        router::Route,
        today, use_date_format, use_prefs, use_store,
    },
};
use chrono::Datelike;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_components::{ActionBar, Button, ButtonVariant};

/// The C, the app's own mark.
const LOGO_CAIRN: &str = include_str!("../../../../../assets/c.txt");
/// The S, the owner's dev mark. This started as a personal app and the
/// wordmark is his name, so it stays available behind a setting.
const LOGO_SCADOSHI: &str = include_str!("../../../../../assets/s.txt");

/// The landing screen, which is also the counter list.
#[component]
pub fn Home() -> Element {
    let store = use_store();
    let nav = use_navigator();
    let mut counters = use_signal(Vec::<Counter>::new);
    let mut error = use_signal(|| None::<String>);
    let mut create_open = use_signal(|| false);
    let mut edit_open = use_signal(|| false);
    let mut editing = use_signal(|| None::<Counter>);
    let hint_open = use_signal(|| false);
    use_screen_hint(hint_open);

    // Hooks, read once. `use_prefs` and friends are context hooks, so calling
    // them inside the loop below would run a different number of hooks on the
    // render before the counters load than on the one after, which Dioxus
    // treats as a fatal hook-order change.
    let prefs = use_prefs();
    let date_format = use_date_format();

    let load_store = store.clone();
    let reload = use_callback(move |()| match load_store.list_counters() {
        Ok(list) => counters.set(list),
        Err(e) => error.set(Some(e.to_string())),
    });
    use_effect(move || reload.call(()));

    let (logo, logo_label) = match prefs().logo {
        Logo::Cairn => (LOGO_CAIRN, "Cairn"),
        Logo::Scadoshi => (LOGO_SCADOSHI, "scadoshi"),
    };

    let now = today();

    // Reading is this screen's job; the arithmetic is the domain's, where it
    // can be tested. See stats::across_counters.
    let list = counters();
    let loaded: Vec<(Option<Goal>, Vec<DayCount>)> = list
        .iter()
        .map(|c| (c.goal, store.entries(c.id).unwrap_or_default()))
        .collect();
    let across = stats::across_counters(
        &loaded
            .iter()
            .map(|(goal, entries)| stats::CounterDays {
                goal: *goal,
                entries,
            })
            .collect::<Vec<_>>(),
        now,
        &prefs(),
    );

    let date = format!("{} {}", now.format("%a"), date_format().date(now));
    let day = now.ordinal();
    let week = prefs().week_number(now);

    rsx! {
        div { class: "screen-content",
            div { class: "profile-sections content-enter",
                div { class: "home-hero",
                    div { class: "card-header home-hero-head",
                        pre { class: "logo", "aria-label": "{logo_label}", "{logo}" }
                        div { class: "home-hero-when",
                            span { class: "card-title", "{date}" }
                            div { class: "chip-tags",
                                span { class: "stat-chip stat-chip-goal", "day {day}" }
                                span { class: "stat-chip", "week {week}" }
                            }
                        }
                    }
                    if counters().is_empty() {
                        p { class: "pref-note", style: "padding: 1rem;",
                            "No counters yet. Create starts one."
                        }
                    } else {
                        div { class: "tile-grid tile-grid-2",
                            Tile { label: "logged today", value: compact(across.logged_today) }
                            Tile {
                                label: "goals met",
                                value: "{across.goals_met}",
                                hint: if across.with_goals == 0 { "no goals set".to_string() } else { format!("of {}", across.with_goals) },
                            }
                            Tile { label: "lifetime", value: compact(across.lifetime) }
                            Tile { label: "streak", value: compact(across.streak), hint: "days".to_string() }
                        }
                    }
                }
                QuoteCard {}
                if let Some(e) = error() {
                    p { class: "form-error", "{e}" }
                }
                CounterList {
                    counters: counters(),
                    on_bump: move |()| reload.call(()),
                    on_open: move |id: i64| {
                        nav.push(Route::CounterScreen { id });
                    },
                    on_edit: move |c: Counter| {
                        // Mount the sheet closed, then open it a beat later.
                        // One that mounts already open has no off-screen state
                        // to slide up from, so it appears in place instead of
                        // rising. The wait has to outlast BottomSheet's own
                        // premount guard, which drops `transition: none` after
                        // WebKit's first post-insert paint.
                        editing.set(Some(c));
                        spawn(async move {
                            tokio::time::sleep(Duration::from_millis(70)).await;
                            edit_open.set(true);
                        });
                    },
                }
            }
        }
        ActionBar {
            Button {
                variant: ButtonVariant::Util,
                onclick: move |_| create_open.set(true),
                "Create"
            }
            Button {
                variant: ButtonVariant::Util,
                onclick: move |_| {
                    nav.push(Route::Config {});
                },
                "Config"
            }
        }
        HintDialog { open: hint_open, title: "Counters",
            HintLine { "Tap a counter's name or numbers to open it." }
            HintBullets {
                HintBullet { HintKey { color: "--accent-primary", "+" } " and " HintKey { color: "--accent-primary", "-" } " log today. Each tap moves by that counter's step." }
                HintBullet { HintKey { color: "--accent-primary", "Edit" } " renames a counter or changes its goal and step." }
                HintBullet {
                    HintChip { class: "stat-chip-short", "40 to go" }
                    " is what today still owes. It becomes "
                    HintChip { class: "stat-chip-met", "goal met" }
                    " when you get there."
                }
                HintBullet { "Up top is every counter added together." }
            }
        }
        CreateSheet { open: create_open, on_created: move |()| reload.call(()) }
        if let Some(c) = editing() {
            EditSheet {
                key: "{c.id.0}",
                open: edit_open,
                id: c.id,
                current_name: c.name.to_string(),
                current_goal: c.goal,
                current_step: c.step.get(),
                current_big_step: c.big_step.map(crate::domain::counter::Step::get),
                current_celebration: c.celebration,
                on_saved: move |()| reload.call(()),
            }
        }
    }
}

/// Create a counter in a sheet, the same form the edit sheet uses.
#[component]
fn CreateSheet(open: Signal<bool>, on_created: EventHandler<()>) -> Element {
    let mut open = open;
    let store = use_store();
    let toast = use_toast();
    let mut form = use_hook(CounterFormState::default);
    let hint_open = use_signal(|| false);

    // Every open starts blank.
    use_effect(move || {
        if open() {
            form.load("", None, 1, None, None);
        }
    });

    let save = move |_| {
        let Some(v) = form.validate() else {
            return;
        };
        match store.create_counter(&v.name, v.goal, v.step, v.big_step, v.celebration, today()) {
            Ok(c) => {
                toast.success(
                    format!("Saved {}", c.name),
                    ToastOptions::default().duration(TOAST_NORMAL),
                );
                bump_store_version();
                on_created.call(());
                open.set(false);
            }
            Err(e) => form.error.set(Some(e.to_string())),
        }
    };

    rsx! {
        HintDialog { open: hint_open, title: "New counter",
            HintLine { "Name it after the thing you do, like pushups." }
            HintBullets {
                HintBullet { "Goal is optional. Pick a number and whether it is per day, week, or year." }
                HintBullet { "A yearly goal still shows a daily share, so " HintChip { class: "stat-chip-goal", "1,000/year" } " asks for 3 a day." }
                HintBullet { "Step is how much one tap adds. Set it to 10 and " HintKey { color: "--accent-primary", "+10" } " logs ten at a time." }
                HintBullet { "Big step adds a second, larger pair outside the first: " HintKey { color: "--accent-primary", "-20" } HintKey { color: "--accent-primary", "-10" } HintKey { color: "--accent-primary", "+10" } HintKey { color: "--accent-primary", "+20" } ". Leave it on None for one pair." }
                HintBullet { "Goal animation is this counter's own. " HintKey { color: "--accent-primary", "Default" } " follows the app-wide one in Config." }
            }
        }
        BottomSheet {
            open,
            title: "Create counter",
            hint: hint_open,
            footer: rsx! {
                Button { variant: ButtonVariant::Util, onclick: move |_| open.set(false), "Back" }
                Button { variant: ButtonVariant::Util, onclick: save, "Save" }
            },
            CounterForm { state: form }
        }
    }
}
