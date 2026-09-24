//! One counter: the odometer, today's controls, stats, trends, years, rename,
//! delete.

use crate::{
    domain::counter::{
        Counter, CounterId, DayCount, Event, Goal, Step,
        format::{compact, compact_i64, thousands, thousands_i64},
        series,
        series::HourlyBasis,
        stats,
        stats::Summary,
    },
    inbound::ui::{
        TOAST_NORMAL, TOAST_QUICK, bump_store_version,
        components::{
            alert_dialog::ConfirmDialog,
            celebration::{CelebrationHost, celebrate},
            counter_form::EditSheet,
            hint::{HintBullet, HintBullets, HintDialog, HintKey, HintLine, use_screen_hint},
            line_chart::{LineChart, Point},
            tile::{Tile, TileGrid, rate},
        },
        now,
        router::Route,
        today, use_date_format, use_prefs, use_store,
    },
};
use chrono::{Datelike, Duration as ChronoDuration};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
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
    let best = use_signal(|| Best::Day);
    let date_format = use_date_format();

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
    let host = use_context::<CelebrationHost>();
    let celebrate_pref = use_prefs();
    let adjust = use_callback(move |delta: i64| {
        let today_count = entries()
            .iter()
            .find(|e| e.day == today())
            .map_or(0, |e| e.count);
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
        match adjust_store.adjust(id, now(), today(), applied) {
            Ok(_) => {
                // The tap that finishes the day says so, once.
                let done = counter().and_then(|c| c.goal).is_some_and(|g| {
                    let days = stats::days_in_year(today().year());
                    stats::crosses_goal(g, today_count, applied, days)
                });
                if done {
                    // The counter's own choice wins; None follows Config.
                    let how = counter()
                        .and_then(|c| c.celebration)
                        .unwrap_or_else(|| celebrate_pref().celebration);
                    if let Some(line) = celebrate(host, how) {
                        toast.success(
                            line.to_string(),
                            ToastOptions::default().duration(TOAST_NORMAL),
                        );
                    }
                } else {
                    // No counter name: you are already looking at the counter,
                    // and a long one pushed the toast off the screen edge.
                    toast.info(
                        if applied >= 0 {
                            format!("+{}", thousands_i64(applied))
                        } else {
                            format!("-{}", thousands_i64(-applied))
                        },
                        ToastOptions::default().duration(TOAST_QUICK),
                    );
                }
                reload.call(());
            }
            Err(e) => toast.error(e.to_string(), ToastOptions::default()),
        }
    });

    let Some(c) = counter() else {
        return rsx! {
            div { class: "screen-content",
                div { class: "profile-sections",
                    p { class: "pref-note", "No counter with id {id}." }
                }
            }
        };
    };
    let step = i64::from(c.step.get());
    let big = c.big_step.map(|b| i64::from(b.get()));
    let now = today();
    let prefs = use_prefs()();
    let summary = stats::summarize_with(&entries(), c.goal, now, &prefs);
    let week_total: u32 = series::this_week(&entries(), now, &prefs)
        .iter()
        .flatten()
        .sum();
    let last_week_total = series::window_total(
        &entries(),
        series::week_start(now, &prefs) - ChronoDuration::days(1),
        7,
    );
    let week_delta = match week_total.cmp(&last_week_total) {
        std::cmp::Ordering::Greater => format!(
            "up {} on last week",
            thousands(week_total - last_week_total)
        ),
        std::cmp::Ordering::Less => format!(
            "down {} on last week",
            thousands(last_week_total - week_total)
        ),
        std::cmp::Ordering::Equal => "level with last week".to_string(),
    };

    let mut confirm_delete = use_signal(|| false);
    let mut confirm_minus = use_signal(|| false);
    let mut confirm_big = use_signal(|| false);
    let mut rename_open = use_signal(|| false);
    let hint_open = use_signal(|| false);
    use_screen_hint(hint_open);
    let delete_name = c.name.to_string();
    let delete_store = store.clone();
    let delete = move |()| match delete_store.delete_counter(id) {
        Ok(()) => {
            toast.success(
                format!("Deleted {delete_name}"),
                ToastOptions::default().duration(TOAST_NORMAL),
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
                        span {
                            class: if summary.lifetime >= 10_000_000 { "odometer-value odometer-value-xl" } else if summary.lifetime >= 100_000 { "odometer-value odometer-value-l" } else { "odometer-value" },
                            "{thousands(summary.lifetime)}"
                        }
                        span { class: "odometer-label", "lifetime since {date_format().date(c.created_on)}" }
                    }
                    TileGrid {
                        Tile { label: "today", value: compact(summary.today) }
                        Tile { label: "this week", value: compact(week_total), hint: week_delta.clone() }
                        Tile { label: "this month", value: compact(summary.this_month) }
                    }
                    ActionBar {
                        // Big on the outside, matching the card's bar.
                        if let Some(big) = big {
                            Button {
                                variant: ButtonVariant::Util,
                                onclick: move |_| if prefs.confirm_minus { confirm_big.set(true) } else { adjust.call(-big) },
                                "-{big}"
                            }
                        }
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: move |_| if prefs.confirm_minus { confirm_minus.set(true) } else { adjust.call(-step) },
                            "-{step}"
                        }
                        Button { variant: ButtonVariant::Util, onclick: move |_| adjust.call(step), "+{step}" }
                        if let Some(big) = big {
                            Button {
                                variant: ButtonVariant::Util,
                                onclick: move |_| adjust.call(big),
                                "+{big}"
                            }
                        }
                    }
                }
                YearCard { summary: summary.clone() }
                if let Some(g) = c.goal {
                    GoalCard { summary: summary.clone(), goal: g }
                }
                TrendsCard { entries: entries(), events: events(), trend }
                BestsCard { entries: entries(), summary: summary.clone(), best }
                HabitCard { summary: summary.clone() }
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
                        nav.push(Route::Home {});
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
        HintDialog { open: hint_open, title: "One counter",
            HintLine { "The big number is everything you have ever logged here." }
            HintBullets {
                HintBullet { "The chips under Trends and Bests change what you are looking at." }
                HintBullet { HintKey { color: "--accent-primary", "Edit" } " changes the name, the goal, and the step." }
                HintBullet { HintKey { color: "--color-error", "Delete" } " removes the counter and its whole history." }
                HintBullet { "Subtracting stops at zero. It never runs a day negative." }
            }
        }
        EditSheet {
            open: rename_open,
            id,
            current_name: c.name.to_string(),
            current_goal: c.goal,
            current_step: c.step.get(),
            current_big_step: c.big_step.map(Step::get),
            current_celebration: c.celebration,
            on_saved: move |()| reload.call(()),
        }
        if let Some(big) = big {
            ConfirmDialog {
                open: confirm_big,
                title: format!("Take {big} off {}?", c.name),
                body: "This subtracts from today's count.".to_string(),
                confirm_label: format!("Take {big}"),
                on_confirm: move |()| adjust.call(-big),
            }
        }
        ConfirmDialog {
            open: confirm_minus,
            title: format!("Take {step} off {}?", c.name),
            body: "This subtracts from today's count.".to_string(),
            confirm_label: format!("Take {step}"),
            on_confirm: move |()| adjust.call(-step),
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
fn TrendsCard(entries: Vec<DayCount>, events: Vec<Event>, trend: Signal<Trend>) -> Element {
    let mut trend = trend;
    let now = today();
    let year = now.year();
    let df = use_date_format()();
    let mut prefs = use_prefs();
    let p = prefs();
    let labels = p.weekday_labels();

    let (points, overlay, unit, note) = match trend() {
        Trend::ThisWeek => {
            let week = series::this_week(&entries, now, &p);
            let points = week
                .iter()
                .zip(labels)
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
            let start = p.week_start.num_days_from_monday() as usize;
            let mut rotated = [None; 7];
            for (i, slot) in rotated.iter_mut().enumerate() {
                *slot = avg.get((start + i) % 7).copied().flatten();
            }
            let points = rotated
                .iter()
                .zip(labels)
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
                    label: df.short(*d),
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
            let weeks = series::weekly_totals(&entries, year, &p);
            let points = weeks
                .iter()
                .map(|(d, c)| Point {
                    label: df.short(*d),
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
            let basis = if p.hourly_all_days {
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
            let note = if p.hourly_all_days {
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
                        Chip { selected: !p.hourly_all_days, onclick: move |_| prefs.with_mut(|q| q.hourly_all_days = false), "Active days" }
                        Chip { selected: p.hourly_all_days, onclick: move |_| prefs.with_mut(|q| q.hourly_all_days = true), "All days" }
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

/// Which best the bests card shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Best {
    Day,
    Week,
    Month,
}

/// This year: total, per day, and where that lands by December.
#[component]
fn YearCard(summary: Summary) -> Element {
    let y = &summary.this_year;
    rsx! {
        div { class: "profile-list",
            div { class: "card-header",
                span { class: "card-title", "{y.year}, day {y.days_elapsed}" }
            }
            TileGrid {
                Tile { label: "total", value: compact(y.total) }
                Tile { label: "per day", value: rate(y.per_day) }
                Tile { label: "projected", value: compact(summary.projected_year_end), hint: "by year end at this pace".to_string() }
            }
        }
    }
}

/// Standing against the goal. Remaining and needed go negative past it.
#[component]
fn GoalCard(summary: Summary, goal: Goal) -> Element {
    let Some(p) = summary.this_year.pace else {
        return rsx! {};
    };
    let year_len = summary.this_year.days_in_year;
    let (pace_value, pace_hint) = if p.delta >= 0 {
        (format!("+{}", compact_i64(p.delta)), "ahead of pace")
    } else {
        (compact_i64(p.delta), "behind pace")
    };
    rsx! {
        div { class: "profile-list",
            div { class: "card-header",
                span { class: "card-title", "Goal" }
                div { class: "chip-tags",
                    for part in goal.parts(year_len) {
                        span { class: "stat-chip stat-chip-goal", "{part}" }
                    }
                }
            }
            TileGrid {
                Tile { label: "remaining", value: compact_i64(p.remaining) }
                Tile { label: "needed per day", value: rate(p.needed_per_day), hint: "to land on it".to_string() }
                Tile { label: "pace", value: pace_value, hint: pace_hint.to_string() }
            }
        }
    }
}

/// The best day, week, or month, picked by chips: how much, the rate, when.
#[component]
fn BestsCard(entries: Vec<DayCount>, summary: Summary, best: Signal<Best>) -> Element {
    let mut best = best;
    let now = today();
    let df = use_date_format()();
    // Read before the match: a hook inside one arm runs only for that chip,
    // and a hook count that changes between renders is fatal.
    let prefs = use_prefs()();
    let months = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let tiles = match best() {
        Best::Day => summary
            .best_day
            .map(|b| (thousands(b.count), None, df.date(b.day))),
        Best::Week => series::weekly_totals(&entries, now.year(), &prefs)
            .into_iter()
            .max_by_key(|(_, t)| *t)
            .map(|(monday, total)| {
                (
                    thousands(total),
                    Some(rate(f64::from(total) / 7.0)),
                    format!("w/c {}", df.short(monday)),
                )
            }),
        Best::Month => series::monthly_totals(&entries, now.year())
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.map(|(t, n)| (i, t, n)))
            .max_by_key(|(_, t, _)| *t)
            .map(|(i, total, active)| {
                (
                    thousands(total),
                    Some(rate(f64::from(total) / f64::from(active))),
                    months.get(i).copied().unwrap_or("").to_string(),
                )
            }),
    };
    rsx! {
        div { class: "profile-list",
            div { class: "card-header",
                span { class: "card-title", "Bests" }
                div { class: "chip-row chip-row-tight",
                    Chip { selected: best() == Best::Day, onclick: move |_| best.set(Best::Day), "Day" }
                    Chip { selected: best() == Best::Week, onclick: move |_| best.set(Best::Week), "Week" }
                    Chip { selected: best() == Best::Month, onclick: move |_| best.set(Best::Month), "Month" }
                }
            }
            if let Some((total, per_day, when)) = tiles {
                TileGrid {
                    Tile { label: "total", value: total }
                    if let Some(r) = per_day {
                        Tile { label: "per day", value: r }
                    }
                    Tile { label: "when", value: when }
                }
            } else {
                p { class: "pref-note", style: "padding: 1rem;", "Nothing logged this year yet." }
            }
        }
    }
}

/// How regular the habit is.
#[component]
fn HabitCard(summary: Summary) -> Element {
    let last = summary.days_since_last.map(|d| match d {
        0 => "today".to_string(),
        1 => "yesterday".to_string(),
        n => format!("{n} days ago"),
    });
    rsx! {
        div { class: "profile-list",
            div { class: "card-header",
                span { class: "card-title", "Habit" }
            }
            div { class: "tile-grid tile-grid-2",
                Tile { label: "streak", value: compact(summary.streak), hint: "days".to_string() }
                Tile { label: "longest streak", value: compact(summary.longest_streak), hint: "days".to_string() }
                Tile {
                    label: "consistency",
                    value: format!("{:.0}%", summary.consistency * 100.0),
                    hint: format!("{} of {} days this year", summary.this_year.active_days, summary.this_year.days_elapsed),
                }
                if let Some(l) = last {
                    Tile { label: "last logged", value: l }
                }
            }
        }
    }
}
