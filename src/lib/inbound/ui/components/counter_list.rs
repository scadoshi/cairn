//! The counter list, the main body of the home screen. Each card shows the
//! headline numbers and carries its own row with plus, minus and a pencil;
//! tapping the name and numbers opens the counter, so that handler sits on
//! that region rather than the whole card and never has to compete with the
//! buttons below it.
//!
//! The cards live here but the sheets do not: a fixed overlay inside a card
//! is clipped by the card's own rounded corners, so the screen owns those
//! and a card only says which counter it wants opened.

use std::sync::Arc;

use chrono::Datelike;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use orangui::icons::EditIcon;
use orangui::{Button, ButtonVariant};

use crate::domain::counter::{Counter, format, stats};
use crate::inbound::ui::components::alert_dialog::ConfirmDialog;
use crate::inbound::ui::components::celebration::{self, CelebrationHost};
use crate::inbound::ui::components::reveal::Reveal;
use crate::inbound::ui::components::tile::{self, Num, Tile, TileGrid};
use crate::inbound::ui::{self, use_prefs, use_store};
use crate::screens::tap::{Plan, Tap};

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
        for c in ui::ordered(&counters, &store, prefs().counter_order, ui::shown_day()) {
            CounterCard {
                key: "{c.id.0}",
                counter: c.clone(),
                on_bump: move |()| on_bump.call(()),
                on_open: move |id: i64| on_open.call(id),
                on_edit: move |()| on_edit.call(c.clone()),
            }
        }
    }
}

/// One counter on the home list: name and goal up top, the headline numbers,
/// and a row that adjusts today's count or edits the counter. Tapping the
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
    let shown = ui::shown_day();
    let summary = stats::summarize_with(&entries, counter.goal, shown, &prefs);
    let id = counter.id;
    let bump_store = Arc::clone(&store);
    let toast = use_toast();
    let step = i64::from(counter.step.get());
    let confirm_minus = prefs.confirm_minus;
    let days = stats::days_in_year(shown.year());
    // `None` once the day is done, so the tag has two states and no zero.
    let left_today = counter
        .goal
        .map(|g| stats::remaining_today(g, summary.today, days))
        .filter(|left| *left > 0);
    // The same question asked of the year rather than of an average day:
    // what today needs given how far ahead or behind the total is. Only
    // shown when it disagrees with the flat one, which is the only time it
    // says anything the tag beside it does not.
    let pace_today = summary
        .this_year
        .pace
        .as_ref()
        .map(|p| stats::pace_remaining_today(p, summary.today))
        .filter(|left| *left > 0 && Some(*left) != left_today);
    let big = counter.big_step.map(|b| i64::from(b.get()));
    let mut confirm_open = use_signal(|| false);
    let mut confirm_big = use_signal(|| false);
    // Owned, because the callback outlives this render.
    let name = counter.name.to_string();
    let host = use_context::<CelebrationHost>();
    let card_celebration = counter.celebration;
    let goal = counter.goal;
    let bump = use_callback(move |delta: i64| {
        // The day and its count as the store has them at the tap, not as
        // the last render drew them: the day may have turned since.
        let day = ui::today(&prefs);
        let stored = match bump_store.entries(id) {
            Ok(stored) => stored,
            Err(e) => {
                toast.error(e.to_string(), ToastOptions::default());
                return;
            }
        };
        let tap = Tap {
            name: &name,
            entries: &stored,
            goal,
            own: card_celebration,
            app: prefs.celebration,
            day,
        };
        let applied = match tap.plan(delta) {
            Plan::Nothing(said) => {
                ui::raise(toast, &said);
                return;
            }
            Plan::Apply(applied) => applied,
        };
        match bump_store.adjust(id, ui::now(), day, applied) {
            Ok(_) => {
                let outcome = tap.outcome(applied, || ui::all_goals_met(&bump_store, day, &prefs));
                if let Some(said) = celebration::play(host, outcome) {
                    ui::raise(toast, &said);
                }
                on_bump.call(());
            }
            Err(e) => toast.error(e.to_string(), ToastOptions::default()),
        }
    });

    rsx! {
        Reveal { class: "settings-list",
            div {
                class: "card-tap",
                role: "button",
                onclick: move |_| on_open.call(id.0),
                // The tags are siblings of the name, not a block beside it:
                // as a block they wrapped whole, dropping every tag below
                // the name as soon as one of them did not fit. Flat, they
                // fill the row and wrap one at a time.
                div { class: "settings-list-header card-header-flow",
                    span { class: "settings-list-title", "{counter.name}" }
                    if let Some(g) = counter.goal {
                        for part in g.parts(days) {
                            span { class: "stat-chip stat-chip-p4", "{part}" }
                        }
                        // Red until the day's share is logged, green after.
                        // It carries the number so one tag answers both
                        // "am I done" and "how much is left".
                        if let Some(left) = left_today {
                            span { class: "stat-chip stat-chip-error", Num { text: format::thousands(left) } " to go" }
                        } else {
                            span { class: "stat-chip stat-chip-success", "goal met" }
                        }
                        if let Some(left) = pace_today {
                            span { class: "stat-chip stat-chip-muted pace-chip", Num { text: format::thousands(left) } " to pace" }
                        }
                    }
                }
                TileGrid {
                    Tile { label: "lifetime", value: format::compact(summary.lifetime) }
                    Tile { label: "today", value: format::compact(summary.today) }
                    Tile {
                        label: "this year",
                        value: format::compact(summary.this_year.total),
                        hint: format!("{}/day", tile::rate(summary.this_year.per_day)),
                    }
                }
            }
            div { class: "action-bar step-row",
                // Big on the outside: distance from the center matching
                // magnitude is what makes the row readable without labels.
                if let Some(big) = big {
                    Button {
                        variant: ButtonVariant::Action,
                        onclick: move |_| if confirm_minus { confirm_big.set(true) } else { bump.call(-big) },
                        "-{big}"
                    }
                }
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| if confirm_minus { confirm_open.set(true) } else { bump.call(-step) },
                    "-{step}"
                }
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| bump.call(step),
                    "+{step}"
                }
                if let Some(big) = big {
                    Button {
                        variant: ButtonVariant::Action,
                        onclick: move |_| bump.call(big),
                        "+{big}"
                    }
                }
                button {
                    class: "btn-action btn-icon",
                    r#type: "button",
                    aria_label: "Edit",
                    onclick: move |_| on_edit.call(()),
                    EditIcon {}
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
