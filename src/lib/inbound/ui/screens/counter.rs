//! One counter: the odometer, today's controls, stats, trends, years, rename,
//! delete.

use crate::{
    domain::counter::{
        Counter, CounterId, CounterName, DayCount, Event, series, stats, stats::Summary,
    },
    inbound::ui::{
        bump_store_version,
        components::{
            alert_dialog::ConfirmDialog,
            bottom_sheet::BottomSheet,
            line_chart::{LineChart, Point},
            stat_tile::{StatTile, rate},
        },
        now,
        router::Route,
        today, use_store,
    },
};
use chrono::{Datelike, Duration as ChronoDuration};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_components::{ActionBar, Button, ButtonVariant, Chip};

/// Which series the trends card draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Trend {
    Daily,
    Weekly,
    Monthly,
    Hourly,
}

/// One counter's detail screen.
#[component]
pub fn CounterScreen(id: i64) -> Element {
    let store = use_store();
    let id = CounterId(id);
    let nav = use_navigator();
    let toast = use_toast();

    let mut counter = use_signal(|| None::<Counter>);
    let mut entries = use_signal(Vec::<DayCount>::new);
    let mut events = use_signal(Vec::<Event>::new);
    let mut notice = use_signal(|| None::<String>);
    let trend = use_signal(|| Trend::Daily);

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
        match load_store.events(id) {
            Ok(list) => events.set(list),
            Err(e) => notice.set(Some(e.to_string())),
        }
    });
    use_effect(move || reload.call(()));

    let adjust_store = store.clone();
    let adjust = use_callback(
        move |delta: i64| match adjust_store.adjust(id, now(), delta) {
            Ok(total) => {
                toast.info(
                    format!("{total} today"),
                    ToastOptions::default().duration(Duration::from_millis(900)),
                );
                reload.call(());
            }
            Err(e) => toast.error(e.to_string(), ToastOptions::default()),
        },
    );

    let Some(c) = counter() else {
        return rsx! {
            div { class: "screen-content",
                div { class: "profile-sections",
                    p { class: "pref-note", "No counter with id {id}." }
                }
            }
        };
    };
    let summary = stats::summarize(&entries(), c.goal, today());

    let mut confirm_delete = use_signal(|| false);
    let mut rename_open = use_signal(|| false);
    let delete_name = c.name.to_string();
    let delete_store = store.clone();
    let delete = move |()| match delete_store.delete_counter(id) {
        Ok(()) => {
            toast.success(
                format!("Deleted {delete_name}"),
                ToastOptions::default().duration(Duration::from_millis(1500)),
            );
            bump_store_version();
            nav.push(Route::Home {});
        }
        Err(e) => toast.error(e.to_string(), ToastOptions::default()),
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
                TrendsCard { entries: entries(), events: events(), trend }
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
            Button { variant: ButtonVariant::Util, onclick: move |_| rename_open.set(true), "Rename" }
            Button {
                variant: ButtonVariant::Util,
                danger: true,
                onclick: move |_| confirm_delete.set(true),
                "Delete"
            }
        }
        RenameSheet { open: rename_open, id, current: c.name.to_string(), on_renamed: move |()| reload.call(()) }
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

/// The trend chart with chips to pick the series.
#[component]
fn TrendsCard(entries: Vec<DayCount>, events: Vec<Event>, trend: Signal<Trend>) -> Element {
    let mut trend = trend;
    let now = today();
    let year = now.year();

    let (points, overlay, unit, note) = match trend() {
        Trend::Daily => {
            let from = now - ChronoDuration::days(59);
            let daily = series::daily(&entries, from, now);
            let smooth = series::rolling_average(&daily, 7);
            let points = daily
                .iter()
                .map(|(d, c)| Point {
                    label: d.format("%b %-d").to_string(),
                    value: Some(f64::from(*c)),
                })
                .collect();
            let overlay = smooth.iter().map(|(_, v)| Some(*v)).collect();
            (
                points,
                overlay,
                String::new(),
                "last 60 days, with the 7-day average",
            )
        }
        Trend::Weekly => {
            let weeks = series::weekly_totals(&entries, year);
            let points = weeks
                .iter()
                .map(|(d, c)| Point {
                    label: d.format("%b %-d").to_string(),
                    value: Some(f64::from(*c)),
                })
                .collect();
            (
                points,
                Vec::new(),
                String::new(),
                "total per week this year",
            )
        }
        Trend::Monthly => {
            let months = series::monthly_average(&entries, year);
            let names = [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ];
            let points = months
                .iter()
                .zip(names)
                .map(|(v, n)| Point {
                    label: n.to_string(),
                    value: *v,
                })
                .collect();
            (
                points,
                Vec::new(),
                String::new(),
                "average per active day, by month",
            )
        }
        Trend::Hourly => {
            let hours = series::hourly_average(&events);
            let points = hours
                .iter()
                .enumerate()
                .map(|(h, v)| Point {
                    label: format!("{h:02}:00"),
                    value: Some(*v),
                })
                .collect();
            (
                points,
                Vec::new(),
                String::new(),
                "average reps per hour of the day",
            )
        }
    };

    rsx! {
        div { class: "profile-list",
            div { class: "card-header",
                span { class: "card-title", "Trends" }
                div { class: "chip-row chip-row-tight",
                    Chip { selected: trend() == Trend::Daily, onclick: move |_| trend.set(Trend::Daily), "Day" }
                    Chip { selected: trend() == Trend::Weekly, onclick: move |_| trend.set(Trend::Weekly), "Week" }
                    Chip { selected: trend() == Trend::Monthly, onclick: move |_| trend.set(Trend::Monthly), "Month" }
                    Chip { selected: trend() == Trend::Hourly, onclick: move |_| trend.set(Trend::Hourly), "Hour" }
                }
            }
            div { class: "chart-body",
                LineChart { points, overlay, unit }
                p { class: "chart-note", "{note}" }
            }
        }
    }
}

/// Rename in a sheet, like zwiper's change-username sheet.
#[component]
fn RenameSheet(
    open: Signal<bool>,
    id: CounterId,
    current: String,
    on_renamed: EventHandler<()>,
) -> Element {
    let mut open = open;
    let store = use_store();
    let toast = use_toast();
    let mut name = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);

    let seed = current.clone();
    use_effect(move || {
        if open() {
            name.set(seed.clone());
            error.set(None);
        }
    });

    let save = move |_| match CounterName::new(&name()) {
        Err(e) => error.set(Some(e.to_string())),
        Ok(new_name) => match store.rename_counter(id, &new_name) {
            Ok(()) => {
                toast.success(
                    format!("Renamed to {new_name}"),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
                bump_store_version();
                on_renamed.call(());
                open.set(false);
            }
            Err(e) => error.set(Some(e.to_string())),
        },
    };

    rsx! {
        BottomSheet {
            open,
            title: "Rename",
            footer: rsx! {
                Button { variant: ButtonVariant::Util, onclick: move |_| open.set(false), "Back" }
                Button { variant: ButtonVariant::Util, onclick: save, "Save" }
            },
            input {
                class: "input",
                value: "{name}",
                maxlength: "{CounterName::MAX_LEN}",
                oninput: move |e| name.set(e.value()),
            }
            if let Some(e) = error() {
                p { class: "form-error", "{e}" }
            }
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
