//! One counter: the odometer, today's controls, stats, trends, years, rename,
//! delete.

use crate::{
    domain::counter::{
        Counter, CounterId, DayCount, Event, Goal,
        format::{thousands, thousands_i64},
        series,
        series::HourlyBasis,
        stats,
        stats::Summary,
    },
    inbound::ui::{
        bump_store_version,
        components::{
            alert_dialog::ConfirmDialog,
            bottom_sheet::BottomSheet,
            counter_form::{CounterForm, CounterFormState},
            line_chart::{LineChart, Point},
            tile::{Tile, TileGrid, rate},
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
    ThisWeek,
    Daily,
    Weekly,
    Monthly,
    Weekday,
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
    let trend = use_signal(|| Trend::ThisWeek);
    let all_days = use_signal(|| false);

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
                    format!("{} today", thousands(total)),
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
    let step = i64::from(c.step.get());

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
            nav.push(Route::Counters {});
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
                        span { class: "odometer-value", "{thousands(summary.lifetime)}" }
                        span { class: "odometer-label", "lifetime since {c.created_on}" }
                    }
                    TileGrid {
                        Tile { label: "today", value: thousands(summary.today) }
                        Tile { label: "streak", value: format!("{} days", summary.streak) }
                        Tile { label: "this month", value: thousands(summary.this_month) }
                    }
                    ActionBar {
                        Button { variant: ButtonVariant::Util, onclick: move |_| adjust.call(-step), "-{step}" }
                        Button { variant: ButtonVariant::Util, onclick: move |_| adjust.call(step), "+{step}" }
                    }
                }
                TrendsCard { entries: entries(), events: events(), trend, all_days }
                NumbersCard { entries: entries(), summary: summary.clone(), goal: c.goal }
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
                                    td { "{thousands(y.total)}" }
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
                    if nav.can_go_back() {
                        nav.go_back();
                    } else {
                        nav.push(Route::Counters {});
                    }
                },
                "Back"
            }
            Button { variant: ButtonVariant::Util, onclick: move |_| rename_open.set(true), "Edit" }
            Button {
                variant: ButtonVariant::Util,
                danger: true,
                onclick: move |_| confirm_delete.set(true),
                "Delete"
            }
        }
        EditSheet {
            open: rename_open,
            id,
            current_name: c.name.to_string(),
            current_goal: c.goal,
            current_step: c.step.get(),
            on_saved: move |()| reload.call(()),
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

/// The trend chart with chips to pick the series.
#[component]
fn TrendsCard(
    entries: Vec<DayCount>,
    events: Vec<Event>,
    trend: Signal<Trend>,
    /// Hourly view only: divide by every day since the first tap rather than
    /// by days with taps.
    all_days: Signal<bool>,
) -> Element {
    let mut trend = trend;
    let mut all_days = all_days;
    let now = today();
    let year = now.year();

    let (points, overlay, unit, note) = match trend() {
        Trend::ThisWeek => {
            let week = series::this_week(&entries, now);
            let points = week
                .iter()
                .zip(series::WEEKDAYS)
                .map(|(v, n)| Point {
                    label: n.to_string(),
                    value: v.map(f64::from),
                })
                .collect();
            (
                points,
                Vec::new(),
                String::new(),
                "this week, Monday to today",
            )
        }
        Trend::Weekday => {
            let avg = series::weekday_average(&entries);
            let points = avg
                .iter()
                .zip(series::WEEKDAYS)
                .map(|(v, n)| Point {
                    label: n.to_string(),
                    value: *v,
                })
                .collect();
            (
                points,
                Vec::new(),
                String::new(),
                "average per active day, by weekday, all time",
            )
        }
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
            let basis = if all_days() {
                let first = events.first().map_or(now, |e| e.at.date());
                let days = u32::try_from((now - first).num_days() + 1).unwrap_or(1);
                HourlyBasis::AllDays { days }
            } else {
                HourlyBasis::ActiveDays
            };
            let hours = series::hourly_average(&events, basis);
            let points = hours
                .iter()
                .enumerate()
                .map(|(h, v)| Point {
                    label: format!("{h:02}:00"),
                    value: Some(*v),
                })
                .collect();
            let note = if all_days() {
                "average reps per hour, over every day since the first tap"
            } else {
                "average reps per hour, over days with taps"
            };
            (points, Vec::new(), String::new(), note)
        }
    };

    rsx! {
        div { class: "profile-list",
            div { class: "card-header",
                span { class: "card-title", "Trends" }
                div { class: "chip-row chip-row-tight",
                    Chip { selected: trend() == Trend::ThisWeek, onclick: move |_| trend.set(Trend::ThisWeek), "This week" }
                    Chip { selected: trend() == Trend::Daily, onclick: move |_| trend.set(Trend::Daily), "60 days" }
                    Chip { selected: trend() == Trend::Weekly, onclick: move |_| trend.set(Trend::Weekly), "Weeks" }
                    Chip { selected: trend() == Trend::Monthly, onclick: move |_| trend.set(Trend::Monthly), "Months" }
                    Chip { selected: trend() == Trend::Weekday, onclick: move |_| trend.set(Trend::Weekday), "Weekday" }
                    Chip { selected: trend() == Trend::Hourly, onclick: move |_| trend.set(Trend::Hourly), "Hour" }
                }
            }
            div { class: "chart-body",
                if trend() == Trend::Hourly {
                    div { class: "chip-row chip-row-tight chip-row-basis",
                        Chip { selected: !all_days(), onclick: move |_| all_days.set(false), "Active days" }
                        Chip { selected: all_days(), onclick: move |_| all_days.set(true), "All days" }
                    }
                }
                LineChart { points, overlay, unit }
                if trend() == Trend::Daily {
                    p { class: "chart-legend",
                        span { class: "legend-swatch legend-line" }
                        "each day"
                        span { class: "legend-swatch legend-overlay" }
                        "7-day average"
                    }
                }
                p { class: "chart-note", "{note}" }
            }
        }
    }
}

/// The numbers behind the charts: single figures as tiles, comparisons and
/// dates as ruled rows underneath.
#[component]
fn NumbersCard(entries: Vec<DayCount>, summary: Summary, goal: Option<Goal>) -> Element {
    let now = today();
    let week = series::this_week(&entries, now);
    let week_total: u32 = week.iter().flatten().sum();
    let week_days = u32::try_from(week.iter().flatten().count())
        .unwrap_or(1)
        .max(1);
    let last_week_end = series::week_start(now) - ChronoDuration::days(1);
    let last_week_total = series::window_total(&entries, last_week_end, 7);
    let last7 = series::window_total(&entries, now, 7);
    let prev7 = series::window_total(&entries, now - ChronoDuration::days(7), 7);
    let best_week = series::weekly_totals(&entries, now.year())
        .into_iter()
        .max_by_key(|(_, t)| *t);
    let best_month = series::monthly_average(&entries, now.year())
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.map(|v| (i, v)))
        .max_by(|a, b| a.1.total_cmp(&b.1));
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let year_len = stats::days_in_year(now.year());
    let projection_hint = goal.map_or_else(
        || "at this pace".to_string(),
        |g| {
            let target = g.yearly(year_len);
            if summary.projected_year_end >= target {
                "clears the goal".to_string()
            } else {
                "short of the goal".to_string()
            }
        },
    );
    let delta = |a: u32, b: u32, than: &str| -> String {
        match a.cmp(&b) {
            std::cmp::Ordering::Greater => format!("up {} on {than}", thousands(a - b)),
            std::cmp::Ordering::Less => format!("down {} on {than}", thousands(b - a)),
            std::cmp::Ordering::Equal => format!("level with {than}"),
        }
    };

    rsx! {
        div { class: "profile-list",
            div { class: "card-header",
                span { class: "card-title", "Numbers" }
            }
            TileGrid {
                Tile { label: "this week", value: thousands(week_total), hint: format!("{}/day", rate(f64::from(week_total) / f64::from(week_days))) }
                Tile { label: "last 7 days", value: thousands(last7) }
                Tile { label: "consistency", value: format!("{:.0}%", summary.consistency * 100.0), hint: format!("{} of {} days", summary.this_year.active_days, summary.this_year.days_elapsed) }
                Tile { label: "projected", value: thousands(summary.projected_year_end), hint: projection_hint }
                Tile { label: "longest streak", value: format!("{} days", summary.longest_streak) }
                if let Some(b) = summary.best_day {
                    Tile { label: "best day", value: thousands(b.count), hint: b.day.format("%-d %b").to_string() }
                }
            }
            NumberRow { label: "This week", value: delta(week_total, last_week_total, "last week"), hint: String::new() }
            NumberRow { label: "Last 7 days", value: delta(last7, prev7, "the 7 before"), hint: String::new() }
            if let Some(d) = summary.days_since_last {
                NumberRow { label: "Last logged", value: match d { 0 => "today".to_string(), 1 => "yesterday".to_string(), n => format!("{n} days ago") }, hint: String::new() }
            }
            if let Some((monday, total)) = best_week {
                NumberRow { label: "Best week", value: format!("{}, w/c {}", thousands(total), monday.format("%-d %b")), hint: String::new() }
            }
            if let Some((i, avg)) = best_month {
                NumberRow { label: "Best month", value: format!("{}, {}/day", months.get(i).copied().unwrap_or(""), rate(avg)), hint: String::new() }
            }
        }
    }
}

/// One ruled row: label left, value right, a muted hint under the value.
#[component]
fn NumberRow(label: String, value: String, hint: String) -> Element {
    rsx! {
        div { class: "profile-row",
            span { class: "profile-row-label hero-label", "{label}" }
            div { class: "profile-row-value number-value",
                span { "{value}" }
                if !hint.is_empty() {
                    span { class: "number-hint", "{hint}" }
                }
            }
        }
    }
}

/// Name, goal, and step in a sheet, the same form the create sheet uses.
#[component]
fn EditSheet(
    open: Signal<bool>,
    id: CounterId,
    current_name: String,
    current_goal: Option<Goal>,
    current_step: u32,
    on_saved: EventHandler<()>,
) -> Element {
    let mut open = open;
    let store = use_store();
    let toast = use_toast();
    let mut form = use_hook(CounterFormState::default);

    let seed_name = current_name.clone();
    use_effect(move || {
        if open() {
            form.load(&seed_name, current_goal, current_step);
        }
    });

    let save = move |_| {
        let Some((name, goal, step)) = form.validate() else {
            return;
        };
        match store.update_counter(id, &name, goal, step) {
            Ok(()) => {
                toast.success(
                    format!("Saved {name}"),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
                bump_store_version();
                on_saved.call(());
                open.set(false);
            }
            Err(e) => form.error.set(Some(e.to_string())),
        }
    };

    rsx! {
        BottomSheet {
            open,
            title: "Edit counter",
            footer: rsx! {
                Button { variant: ButtonVariant::Util, onclick: move |_| open.set(false), "Back" }
                Button { variant: ButtonVariant::Util, onclick: save, "Save" }
            },
            CounterForm { state: form }
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
                span { class: "card-title", "{y.year}, day {y.days_elapsed}" }
            }
            TileGrid {
                Tile { label: "total", value: thousands(y.total) }
                Tile { label: "per day", value: rate(y.per_day) }
                Tile { label: "lifetime per day", value: rate(summary.lifetime_per_day) }
                if let Some(p) = &y.pace {
                    Tile { label: "goal", value: thousands(p.goal), hint: format!("{} remaining", thousands(p.remaining)) }
                    Tile {
                        label: "target today",
                        value: thousands(p.target_today),
                        hint: if p.delta >= 0 { format!("{} ahead", thousands_i64(p.delta)) } else { format!("{} behind", thousands_i64(-p.delta)) },
                    }
                    Tile { label: "needed per day", value: p.needed_per_day.map_or_else(|| "done".to_string(), rate) }
                }
            }
        }
    }
}
