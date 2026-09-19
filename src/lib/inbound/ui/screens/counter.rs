//! One counter: the odometer, today's controls, stats, years, export, delete.

use crate::{
    domain::counter::{Counter, CounterId, DayCount, stats, stats::Summary},
    inbound::ui::{
        components::{
            alert_dialog::ConfirmDialog,
            stat_tile::{StatTile, rate},
        },
        router::Route,
        today, use_store,
    },
};
use dioxus::prelude::*;
use zwipe_components::{ActionBar, Button, ButtonVariant};

/// One counter's detail screen.
#[component]
pub fn CounterScreen(id: i64) -> Element {
    let store = use_store();
    let id = CounterId(id);
    let nav = use_navigator();

    let mut counter = use_signal(|| None::<Counter>);
    let mut entries = use_signal(Vec::<DayCount>::new);
    let mut notice = use_signal(|| None::<String>);

    let load_store = store.clone();
    let reload = use_callback(move |()| {
        match load_store.get_counter(id) {
            Ok(c) => counter.set(c),
            Err(e) => notice.set(Some(e.to_string())),
        }
        match load_store.entries(id) {
            Ok(list) => entries.set(list),
            Err(e) => notice.set(Some(e.to_string())),
        }
    });
    use_effect(move || reload.call(()));

    let adjust_store = store.clone();
    let adjust = use_callback(
        move |delta: i64| match adjust_store.adjust(id, today(), delta) {
            Ok(_) => reload.call(()),
            Err(e) => notice.set(Some(e.to_string())),
        },
    );

    let Some(c) = counter() else {
        return rsx! {
            div { class: "profile-sections",
                p { class: "pref-note", "No counter with id {id}." }
            }
        };
    };
    let summary = stats::summarize(&entries(), c.goal, today());

    let mut confirm_delete = use_signal(|| false);
    let delete_store = store.clone();
    let delete = move |()| match delete_store.delete_counter(id) {
        Ok(()) => {
            nav.push(Route::Home {});
        }
        Err(e) => notice.set(Some(e.to_string())),
    };

    rsx! {
        div { class: "screen-content",
            div { class: "profile-sections content-enter",
                if let Some(n) = notice() {
                    p { class: "pref-note", "{n}" }
                }
                div { class: "profile-list",
                    div { class: "odometer",
                        span { class: "odometer-value", "{summary.lifetime}" }
                        span { class: "odometer-label", "lifetime since {c.created_on}" }
                    }
                    div { class: "stat-grid stat-grid-3",
                        StatTile { label: "Today", value: summary.today.to_string() }
                        StatTile { label: "Streak", value: format!("{} days", summary.streak) }
                        StatTile { label: "This month", value: summary.this_month.to_string() }
                    }
                    ActionBar {
                        Button { variant: ButtonVariant::Util, onclick: move |_| adjust.call(-1), "-1" }
                        Button { variant: ButtonVariant::Util, onclick: move |_| adjust.call(1), "+1" }
                        Button { variant: ButtonVariant::Util, onclick: move |_| adjust.call(5), "+5" }
                        Button { variant: ButtonVariant::Util, onclick: move |_| adjust.call(10), "+10" }
                    }
                }
                YearCard { summary: summary.clone() }
                div { class: "profile-list",
                    div { class: "card-header",
                        span { class: "card-title", "By year" }
                    }
                    table { class: "year-table",
                        thead { tr { th { "Year" } th { "Total" } th { "Per day" } th { "Active" } } }
                        tbody {
                            for y in summary.years.iter() {
                                tr {
                                    td { "{y.year}" }
                                    td { "{y.total}" }
                                    td { "{rate(y.per_day)}" }
                                    td { "{y.active_days} / {y.days_elapsed}" }
                                }
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
                    nav.push(Route::Home {});
                },
                "Back"
            }
            Button {
                variant: ButtonVariant::Util,
                danger: true,
                onclick: move |_| confirm_delete.set(true),
                "Delete"
            }
        }
        ConfirmDialog {
            open: confirm_delete,
            title: format!("Delete {}?", c.name),
            body: format!(
                "This removes the counter and all {} of its logged days. There is no undo.",
                entries().len()
            ),
            confirm_label: "Delete",
            on_confirm: delete,
        }
    }
}

/// This year's totals and, if a goal is set, standing against it.
#[component]
fn YearCard(summary: Summary) -> Element {
    let y = &summary.this_year;
    rsx! {
        div { class: "profile-list",
            div { class: "card-header",
                span { class: "card-title", "{y.year}, day {y.days_elapsed} of {y.days_in_year}" }
            }
            div { class: "stat-grid stat-grid-3",
                StatTile { label: "Total", value: y.total.to_string() }
                StatTile { label: "Per day", value: rate(y.per_day) }
                StatTile {
                    label: "Lifetime/day",
                    value: rate(summary.lifetime_per_day),
                    hint: summary.best_day.map(|b| format!("best {} on {}", b.count, b.day)),
                }
            }
            if let Some(p) = &y.pace {
                div { class: "stat-grid stat-grid-3",
                    StatTile {
                        label: "Goal",
                        value: p.goal.to_string(),
                        hint: format!("{} remaining", p.remaining),
                    }
                    StatTile {
                        label: "Target today",
                        value: p.target_today.to_string(),
                        hint: if p.delta >= 0 { format!("{} ahead", p.delta) } else { format!("{} behind", -p.delta) },
                    }
                    StatTile {
                        label: "Needed per day",
                        value: p.needed_per_day.map_or_else(|| "done".to_string(), rate),
                    }
                }
            }
        }
    }
}
