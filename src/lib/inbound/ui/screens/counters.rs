//! The counter list. Each card shows the headline numbers and carries its own
//! bar with +1 and Edit; tapping the name and numbers opens the counter, so
//! the handler sits on that region rather than the whole card and never has
//! to compete with the buttons below it. The screen bar goes back home or to
//! the create form. Both sheets hang off the screen rather than off a card,
//! because a fixed overlay inside a card is clipped by its rounded corners.

use crate::{
    domain::{
        counter::{
            Counter,
            format::{compact, thousands_i64},
            stats,
            stats::days_in_year,
        },
        preferences::CounterOrder,
    },
    inbound::ui::{
        SharedStore, bump_store_version,
        components::{
            alert_dialog::ConfirmDialog,
            bottom_sheet::BottomSheet,
            counter_form::{CounterForm, CounterFormState, EditSheet},
            tile::{Tile, TileGrid, rate},
        },
        now,
        router::Route,
        today, use_prefs, use_store,
    },
};
use chrono::Datelike;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_components::{ActionBar, Button, ButtonVariant};

/// The counter list.
#[component]
pub fn Counters() -> Element {
    let store = use_store();
    let nav = use_navigator();
    let mut counters = use_signal(Vec::<Counter>::new);
    let mut error = use_signal(|| None::<String>);
    let mut create_open = use_signal(|| false);
    let mut edit_open = use_signal(|| false);
    let mut editing = use_signal(|| None::<Counter>);
    let prefs = use_prefs();

    let load_store = store.clone();
    let reload = use_callback(move |()| match load_store.list_counters() {
        Ok(list) => counters.set(list),
        Err(e) => error.set(Some(e.to_string())),
    });
    use_effect(move || reload.call(()));

    rsx! {
        div { class: "screen-content",
            div { class: "profile-sections content-enter",
                if let Some(e) = error() {
                    p { class: "form-error", "{e}" }
                }
                if counters().is_empty() {
                    p { class: "pref-note", "No counters yet. New counter starts one." }
                }
                for c in ordered(&counters(), &store, prefs().counter_order) {
                    CounterCard {
                        counter: c.clone(),
                        on_bump: move |()| reload.call(()),
                        on_open: move |id: i64| {
                            nav.push(Route::CounterScreen { id });
                        },
                        on_edit: {
                            let c = c.clone();
                            move |()| {
                                // Mount the sheet closed, then open it a beat
                                // later. One that mounts already open has no
                                // off-screen state to slide up from, so it
                                // appears in place instead of rising. The wait
                                // has to outlast BottomSheet's own premount
                                // guard, which drops `transition: none` after
                                // WebKit's first post-insert paint.
                                editing.set(Some(c.clone()));
                                spawn(async move {
                                    tokio::time::sleep(Duration::from_millis(70)).await;
                                    edit_open.set(true);
                                });
                            }
                        },
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
            Button {
                variant: ButtonVariant::Util,
                onclick: move |_| create_open.set(true),
                "Create"
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
                on_saved: move |()| reload.call(()),
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
    let summary = stats::summarize(&entries, counter.goal, today());
    let id = counter.id;
    let bump_store = store.clone();
    let toast = use_toast();
    let name = counter.name.to_string();
    let step = i64::from(counter.step.get());
    let confirm_minus = use_prefs()().confirm_minus;
    let mut confirm_open = use_signal(|| false);
    let bump = use_callback(
        move |delta: i64| match bump_store.adjust(id, now(), today(), delta) {
            Ok(_) => {
                toast.info(
                    if delta >= 0 {
                        format!("+{} to {name}", thousands_i64(delta))
                    } else {
                        format!("-{} from {name}", thousands_i64(-delta))
                    },
                    ToastOptions::default().duration(Duration::from_millis(900)),
                );
                on_bump.call(());
            }
            Err(e) => toast.error(e.to_string(), ToastOptions::default()),
        },
    );

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
                            for part in g.parts(days_in_year(today().year())) {
                                span { class: "stat-chip stat-chip-goal", "{part}" }
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
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| on_edit.call(()),
                    "Edit"
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

/// Create a counter in a sheet, the same form the edit sheet uses.
#[component]
fn CreateSheet(open: Signal<bool>, on_created: EventHandler<()>) -> Element {
    let mut open = open;
    let store = use_store();
    let toast = use_toast();
    let mut form = use_hook(CounterFormState::default);

    // Every open starts blank.
    use_effect(move || {
        if open() {
            form.load("", None, 1);
        }
    });

    let save = move |_| {
        let Some((name, goal, step)) = form.validate() else {
            return;
        };
        match store.create_counter(&name, goal, step, today()) {
            Ok(c) => {
                toast.success(
                    format!("Saved {}", c.name),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
                bump_store_version();
                on_created.call(());
                open.set(false);
            }
            Err(e) => form.error.set(Some(e.to_string())),
        }
    };

    rsx! {
        BottomSheet {
            open,
            title: "Create counter",
            footer: rsx! {
                Button { variant: ButtonVariant::Util, onclick: move |_| open.set(false), "Back" }
                Button { variant: ButtonVariant::Util, onclick: save, "Save" }
            },
            CounterForm { state: form }
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
