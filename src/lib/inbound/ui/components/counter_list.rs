//! The counter list, the main body of the home screen. Each card shows the
//! headline numbers and carries its own bar with plus, minus and Edit;
//! tapping the name and numbers opens the counter, so that handler sits on
//! that region rather than the whole card and never has to compete with the
//! buttons below it.
//!
//! The cards live here but the sheets do not: a fixed overlay inside a card
//! is clipped by the card's own rounded corners, so the screen owns those
//! and a card only says which counter it wants opened.

use crate::{
    domain::{
        counter::{
            Counter,
            format::{compact, thousands, thousands_i64},
            stats,
            stats::days_in_year,
        },
        preferences::CounterOrder,
    },
    inbound::ui::{
        SharedStore, TOAST_NORMAL, TOAST_QUICK,
        components::{
            alert_dialog::ConfirmDialog,
            celebration::{CelebrationHost, celebrate},
            tile::{Tile, TileGrid, rate},
        },
        now, today, use_prefs, use_store,
    },
};
use chrono::Datelike;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_components::{ActionBar, Button, ButtonVariant};

/// Every counter as a card, in the preferred order.
#[component]
pub fn CounterList(
    counters: Vec<Counter>,
    on_bump: EventHandler<()>,
    on_open: EventHandler<i64>,
    on_edit: EventHandler<Counter>,
) -> Element {
    let store = use_store();
    let prefs = use_prefs();

    rsx! {
        for c in ordered(&counters, &store, prefs().counter_order) {
            CounterCard {
                key: "{c.id.0}",
                counter: c.clone(),
                on_bump: move |()| on_bump.call(()),
                on_open: move |id: i64| on_open.call(id),
                on_edit: {
                    let c = c.clone();
                    move |()| on_edit.call(c.clone())
                },
            }
        }
    }
}

/// One counter on the home list: name and goal up top, the headline numbers,
/// and a bar that adjusts today's count or edits the counter. Tapping the
/// numbers opens it.
#[component]
fn CounterCard(
    counter: Counter,
    on_bump: EventHandler<()>,
    on_open: EventHandler<i64>,
    on_edit: EventHandler<()>,
) -> Element {
    let store = use_store();
    let entries = store.entries(counter.id).unwrap_or_default();
    let prefs = use_prefs()();
    let summary = stats::summarize_with(&entries, counter.goal, today(), &prefs);
    let id = counter.id;
    let bump_store = store.clone();
    let toast = use_toast();
    let step = i64::from(counter.step.get());
    let confirm_minus = prefs.confirm_minus;
    let days = days_in_year(today().year());
    // `None` once the day is done, so the tag has two states and no zero.
    let left_today = counter
        .goal
        .map(|g| stats::remaining_today(g, summary.today, days))
        .filter(|left| *left > 0);
    let big = counter.big_step.map(|b| i64::from(b.get()));
    let mut confirm_open = use_signal(|| false);
    let mut confirm_big = use_signal(|| false);
    let today_count = summary.today;
    let host = use_context::<CelebrationHost>();
    let card_celebration = counter.celebration;
    let bump = use_callback(move |delta: i64| {
        // Report what actually happened rather than what was asked for: a
        // day cannot go below zero, so this makes an empty day read "-0"
        // without needing a case of its own.
        let applied = stats::applied_delta(today_count, delta);
        if applied == 0 {
            toast.info(
                "-0".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
            return;
        }
        match bump_store.adjust(id, now(), today(), applied) {
            Ok(_) => {
                // The tap that finishes the day says so, once. Firing from
                // here rather than from render state is what keeps it from
                // going off on every launch of an already-finished day.
                let done = counter
                    .goal
                    .is_some_and(|g| stats::crosses_goal(g, today_count, applied, days));
                if done {
                    // The counter's own choice wins; None follows Config.
                    let how = card_celebration.unwrap_or(prefs.celebration);
                    // Typewriter and Stamp put the words on screen themselves,
                    // so there is nothing left for a toast to add.
                    if let Some(line) = celebrate(host, how) {
                        toast.success(
                            line.to_string(),
                            ToastOptions::default().duration(TOAST_NORMAL),
                        );
                    }
                } else {
                    // No counter name: the toast sits over the card you just
                    // tapped, and a long name pushed it off the screen edge.
                    toast.info(
                        if applied >= 0 {
                            format!("+{}", thousands_i64(applied))
                        } else {
                            format!("-{}", thousands_i64(-applied))
                        },
                        ToastOptions::default().duration(TOAST_QUICK),
                    );
                }
                on_bump.call(());
            }
            Err(e) => toast.error(e.to_string(), ToastOptions::default()),
        }
    });

    rsx! {
        div { class: "profile-list",
            div {
                class: "card-tap",
                role: "button",
                onclick: move |_| on_open.call(id.0),
                div { class: "card-header",
                    span { class: "card-title", "{counter.name}" }
                    if let Some(g) = counter.goal {
                        div { class: "chip-tags",
                            for part in g.parts(days) {
                                span { class: "stat-chip stat-chip-goal", "{part}" }
                            }
                            // Red until the day's share is logged, green after.
                            // It carries the number so one tag answers both
                            // "am I done" and "how much is left".
                            if let Some(left) = left_today {
                                span { class: "stat-chip stat-chip-short", "{thousands(left)} to go" }
                            } else {
                                span { class: "stat-chip stat-chip-met", "goal met" }
                            }
                        }
                    }
                }
                TileGrid {
                    Tile { label: "lifetime", value: compact(summary.lifetime) }
                    Tile { label: "today", value: compact(summary.today) }
                    Tile {
                        label: "this year",
                        value: compact(summary.this_year.total),
                        hint: format!("{}/day", rate(summary.this_year.per_day)),
                    }
                }
            }
            ActionBar {
                // Big on the outside: distance from the centre matching
                // magnitude is what makes the bar readable without labels.
                if let Some(big) = big {
                    Button {
                        variant: ButtonVariant::Util,
                        onclick: move |_| if confirm_minus { confirm_big.set(true) } else { bump.call(-big) },
                        "-{big}"
                    }
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| if confirm_minus { confirm_open.set(true) } else { bump.call(-step) },
                    "-{step}"
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| bump.call(step),
                    "+{step}"
                }
                if let Some(big) = big {
                    Button {
                        variant: ButtonVariant::Util,
                        onclick: move |_| bump.call(big),
                        "+{big}"
                    }
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| on_edit.call(()),
                    "Edit"
                }
            }
            if let Some(big) = big {
                ConfirmDialog {
                    open: confirm_big,
                    title: format!("Take {big} off {}?", counter.name),
                    body: "This subtracts from today's count.".to_string(),
                    confirm_label: format!("Take {big}"),
                    on_confirm: move |()| bump.call(-big),
                }
            }
            ConfirmDialog {
                open: confirm_open,
                title: format!("Take {step} off {}?", counter.name),
                body: "This subtracts from today's count.".to_string(),
                confirm_label: format!("Take {step}"),
                on_confirm: move |()| bump.call(-step),
            }
        }
    }
}

/// The list in the preferred order. Activity orders need each counter's
/// figures, so they read the store; created order is free.
fn ordered(counters: &[Counter], store: &SharedStore, order: CounterOrder) -> Vec<Counter> {
    let mut list = counters.to_vec();
    match order {
        CounterOrder::Created => {}
        CounterOrder::Name => list.sort_by_key(|c| c.name.as_str().to_lowercase()),
        CounterOrder::MostActive | CounterOrder::Lifetime => {
            let key = |c: &Counter| {
                let entries = store.entries(c.id).unwrap_or_default();
                // Preferences only reach streaks and consistency, and
                // neither is a sort key, so the default is the whole truth.
                let s = stats::summarize(&entries, c.goal, today());
                if order == CounterOrder::Lifetime {
                    s.lifetime
                } else {
                    s.this_year.total
                }
            };
            list.sort_by_key(|c| std::cmp::Reverse(key(c)));
        }
    }
    list
}
