//! One counter: the odometer, today's controls, stats, trends, years, rename,
//! delete.

use crate::{
    domain::counter::{
        Counter, CounterId, CounterName, DayCount, Event, Goal, Step, series, series::HourlyBasis,
        stats, stats::Summary,
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
                        span { class: "odometer-value", "{summary.lifetime}" }
                        span { class: "odometer-label", "lifetime since {c.created_on}" }
                    }
                    div { class: "stat-grid stat-grid-3",
                        StatTile { label: "Today", value: summary.today.to_string() }
                        StatTile { label: "Streak", value: format!("{} days", summary.streak) }
                        StatTile { label: "This month", value: summary.this_month.to_string() }
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

/// The numbers behind the charts, as ruled rows.
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
    let projection_note = goal.map(|g| {
        let target = g.yearly(year_len);
        if summary.projected_year_end >= target {
            format!("clears the {target} goal")
        } else {
            format!("short of the {target} goal")
        }
    });
    let delta = |a: u32, b: u32| -> String {
        match a.cmp(&b) {
            std::cmp::Ordering::Greater => format!("up {} on the week before", a - b),
            std::cmp::Ordering::Less => format!("down {} on the week before", b - a),
            std::cmp::Ordering::Equal => "level with the week before".to_string(),
        }
    };

    rsx! {
        div { class: "profile-list",
            div { class: "card-header",
                span { class: "card-title", "Numbers" }
            }
            NumberRow { label: "This week", value: week_total.to_string(), hint: format!("{}/day so far, {}", rate(f64::from(week_total) / f64::from(week_days)), delta(week_total, last_week_total)) }
            NumberRow { label: "Last 7 days", value: last7.to_string(), hint: delta(last7, prev7).replace("the week before", "the 7 before") }
            NumberRow { label: "Streak", value: format!("{} days", summary.streak), hint: format!("longest ever {} days", summary.longest_streak) }
            NumberRow { label: "Consistency", value: format!("{:.0}%", summary.consistency * 100.0), hint: format!("{} of {} days this year", summary.this_year.active_days, summary.this_year.days_elapsed) }
            NumberRow { label: "Projected year end", value: summary.projected_year_end.to_string(), hint: projection_note.unwrap_or_else(|| "at this year's pace".to_string()) }
            if let Some(d) = summary.days_since_last {
                NumberRow { label: "Last logged", value: if d == 0 { "today".to_string() } else { format!("{d} days ago") }, hint: String::new() }
            }
            if let Some((monday, total)) = best_week {
                NumberRow { label: "Best week", value: total.to_string(), hint: format!("week of {}", monday.format("%-d %b")) }
            }
            if let Some((i, avg)) = best_month {
                NumberRow { label: "Best month", value: format!("{}/day", rate(avg)), hint: months.get(i).copied().unwrap_or("").to_string() }
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

/// Name, goal, and step in a sheet, like zwiper's change-username sheet.
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
    let mut name = use_signal(String::new);
    let mut amount = use_signal(String::new);
    let mut per_day = use_signal(|| true);
    let mut step = use_signal(|| 1u32);
    let mut error = use_signal(|| None::<String>);

    let seed_name = current_name.clone();
    use_effect(move || {
        if open() {
            name.set(seed_name.clone());
            match current_goal {
                Some(Goal::PerDay(n)) => {
                    amount.set(n.to_string());
                    per_day.set(true);
                }
                Some(Goal::PerYear(n)) => {
                    amount.set(n.to_string());
                    per_day.set(false);
                }
                None => amount.set(String::new()),
            }
            step.set(current_step);
            error.set(None);
        }
    });

    let save = move |_| {
        let new_name = match CounterName::new(&name()) {
            Ok(n) => n,
            Err(e) => return error.set(Some(e.to_string())),
        };
        let raw = amount();
        let goal = if raw.trim().is_empty() {
            None
        } else {
            let parsed = raw.trim().parse::<u32>().ok().map(|n| {
                if per_day() {
                    Goal::per_day(n)
                } else {
                    Goal::per_year(n)
                }
            });
            match parsed {
                Some(Ok(g)) => Some(g),
                _ => {
                    return error.set(Some(
                        "goal must be a whole number of at least 1".to_string(),
                    ));
                }
            }
        };
        let new_step = match Step::new(step()) {
            Ok(s) => s,
            Err(e) => return error.set(Some(e.to_string())),
        };
        match store.update_counter(id, &new_name, goal, new_step) {
            Ok(()) => {
                toast.success(
                    format!("Saved {new_name}"),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
                bump_store_version();
                on_saved.call(());
                open.set(false);
            }
            Err(e) => error.set(Some(e.to_string())),
        }
    };

    rsx! {
        BottomSheet {
            open,
            title: "Edit",
            footer: rsx! {
                Button { variant: ButtonVariant::Util, onclick: move |_| open.set(false), "Back" }
                Button { variant: ButtonVariant::Util, onclick: save, "Save" }
            },
            p { class: "field-label", "Name" }
            input {
                class: "input",
                value: "{name}",
                maxlength: "{CounterName::MAX_LEN}",
                oninput: move |e| name.set(e.value()),
            }
            p { class: "field-label", "Goal (blank for none)" }
            input {
                class: "input",
                r#type: "number",
                min: "1",
                inputmode: "numeric",
                value: "{amount}",
                oninput: move |e| amount.set(e.value()),
            }
            div { class: "chip-row",
                Chip { selected: per_day(), onclick: move |_| per_day.set(true), "Per day" }
                Chip { selected: !per_day(), onclick: move |_| per_day.set(false), "Per year" }
            }
            p { class: "field-label", "Each tap adds" }
            div { class: "chip-row",
                for n in Step::ALLOWED {
                    Chip { selected: step() == n, onclick: move |_| step.set(n), "{n}" }
                }
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
                span { class: "card-title", "{y.year}, day {y.days_elapsed}" }
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
