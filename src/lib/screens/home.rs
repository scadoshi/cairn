//! Home: every counter as a card, the four tiles across them, and the taps
//! on the cards' step buttons.
//!
//! The counters and their entries load once per [`Effect::LoadCounters`];
//! the tiles, the order and a tap's outcome all compute from what the model
//! holds, so a render reads nothing from the store.

use std::cmp::Reverse;

use chrono::NaiveDate;

use crate::domain::counter::models::{Counter, CounterId, DayCount};
use crate::domain::counter::stats::{self, Across, CounterDays};
use crate::domain::preferences::{Celebration, CounterOrder, Preferences};
use crate::screens::saying::Saying;
use crate::screens::tap::{Outcome, Plan, Tap};
use crate::screens::toast::Toast;

/// One counter with every day it has logged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    counter: Counter,
    entries: Vec<DayCount>,
}

impl Row {
    /// A counter and its entries as the store has them.
    pub fn new(counter: Counter, entries: Vec<DayCount>) -> Self {
        Self { counter, entries }
    }

    /// The counter.
    pub fn counter(&self) -> &Counter {
        &self.counter
    }

    /// Every day the counter has logged.
    pub fn entries(&self) -> &[DayCount] {
        &self.entries
    }

    /// Adds `applied` to `day`'s count, as the store's write did.
    fn add(&mut self, day: NaiveDate, applied: i64) {
        let moved = |count: u32| {
            let after = i64::from(count).saturating_add(applied).max(0);
            u32::try_from(after).unwrap_or(u32::MAX)
        };
        match self.entries.iter_mut().find(|e| e.day == day) {
            Some(entry) => entry.count = moved(entry.count),
            None => self.entries.push(DayCount {
                day,
                count: moved(0),
            }),
        }
    }

    fn days(&self) -> CounterDays<'_> {
        CounterDays {
            goal: self.counter.goal,
            entries: &self.entries,
        }
    }
}

/// A tap written to the store and waiting for its answer.
#[derive(Debug, Clone, Copy)]
struct Pending {
    id: CounterId,
    day: NaiveDate,
    applied: i64,
    prefs: Preferences,
}

/// Home's model.
#[derive(Debug, Default)]
pub struct State {
    rows: Vec<Row>,
    error: Option<String>,
    pending: Option<Pending>,
}

/// What happened on Home.
#[derive(Debug, Clone)]
pub enum Action {
    /// The screen opened or came back to the foreground.
    Load,
    /// The store's answer to [`Effect::LoadCounters`]: every counter with
    /// its entries, or why the list could not be read.
    Loaded(Result<Vec<Row>, String>),
    /// A step button on a card: `delta` on counter `id`, counting on `day`.
    Tap {
        /// The counter tapped.
        id: CounterId,
        /// The step asked for, negative for a minus.
        delta: i64,
        /// The day the tap counts on.
        day: NaiveDate,
        /// The app's preferences at the tap.
        prefs: Preferences,
    },
    /// The store's answer to [`Effect::Adjust`].
    Adjusted(Result<(), String>),
    /// A card's name or numbers.
    Open(CounterId),
    /// A card's pencil.
    Edit(CounterId),
    /// The dock's plus.
    Create,
}

/// What Home asks the client to do.
#[derive(Debug, Clone)]
pub enum Effect {
    /// Read every counter and its entries, answering [`Action::Loaded`].
    LoadCounters,
    /// Add `delta` to counter `id` on `day`, answering [`Action::Adjusted`].
    Adjust {
        /// The counter.
        id: CounterId,
        /// The day it counts on.
        day: NaiveDate,
        /// The change, already trimmed so the day stays at zero or above.
        delta: i64,
    },
    /// Raise a toast.
    Toast(Toast),
    /// Play a celebration and toast its line.
    Celebrate {
        /// The animation.
        how: Celebration,
        /// What it says; `None` is the usual rotating line.
        saying: Option<Saying>,
    },
    /// Go to the counter's screen.
    Open(CounterId),
    /// Open the edit sheet on this counter.
    Edit(Counter),
    /// Open the create sheet.
    Create,
}

impl State {
    /// Applies `action` and returns what the client should do, in order.
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load => vec![Effect::LoadCounters],
            Action::Loaded(Ok(rows)) => {
                self.rows = rows;
                Vec::new()
            }
            // The last list stays up under the message.
            Action::Loaded(Err(e)) => {
                self.error = Some(e);
                Vec::new()
            }
            Action::Tap {
                id,
                delta,
                day,
                prefs,
            } => self.tap(id, delta, day, prefs),
            Action::Adjusted(result) => self.adjusted(result),
            Action::Open(id) => vec![Effect::Open(id)],
            Action::Edit(id) => self
                .row(id)
                .map(|r| Effect::Edit(r.counter.clone()))
                .into_iter()
                .collect(),
            Action::Create => vec![Effect::Create],
        }
    }

    fn row(&self, id: CounterId) -> Option<&Row> {
        self.rows.iter().find(|r| r.counter.id == id)
    }

    fn tap(
        &mut self,
        id: CounterId,
        delta: i64,
        day: NaiveDate,
        prefs: Preferences,
    ) -> Vec<Effect> {
        let Some(row) = self.row(id) else {
            return Vec::new();
        };
        match tap_of(row, day, &prefs).plan(delta) {
            Plan::Nothing(toast) => vec![Effect::Toast(toast)],
            Plan::Apply(applied) => {
                self.pending = Some(Pending {
                    id,
                    day,
                    applied,
                    prefs,
                });
                vec![Effect::Adjust {
                    id,
                    day,
                    delta: applied,
                }]
            }
        }
    }

    fn adjusted(&mut self, result: Result<(), String>) -> Vec<Effect> {
        let Some(pending) = self.pending.take() else {
            return Vec::new();
        };
        if let Err(e) = result {
            return vec![Effect::Toast(Toast::error(e))];
        }
        let Some(index) = self.rows.iter().position(|r| r.counter.id == pending.id) else {
            return vec![Effect::LoadCounters];
        };
        // The tap is asked of the row as it was before the write, and
        // every goal of the rows as they are after it.
        let outcome = self.rows.get(index).map(|before| {
            let tap = tap_of(before, pending.day, &pending.prefs);
            let mut after = self.rows.clone();
            if let Some(row) = after.get_mut(index) {
                row.add(pending.day, pending.applied);
            }
            tap.outcome(pending.applied, || {
                all_met(&after, pending.day, &pending.prefs)
            })
        });
        if let Some(row) = self.rows.get_mut(index) {
            row.add(pending.day, pending.applied);
        }
        let mut effects = Vec::new();
        if let Some(Outcome::Celebrate { how, saying }) = outcome {
            effects.push(Effect::Celebrate { how, saying });
        }
        effects.push(Effect::LoadCounters);
        effects
    }

    /// Whether there are no counters yet.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Why the list last failed to load, if it did.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// The four tiles: logged today, goals met, lifetime and streak.
    pub fn across(&self, day: NaiveDate, prefs: &Preferences) -> Across {
        across(&self.rows, day, prefs)
    }

    /// The rows in `order`, as of `day`.
    pub fn rows(&self, order: CounterOrder, day: NaiveDate) -> Vec<&Row> {
        let mut rows: Vec<&Row> = self.rows.iter().collect();
        match order {
            CounterOrder::Created => {}
            CounterOrder::Name => rows.sort_by_key(|r| r.counter.name.as_str().to_lowercase()),
            CounterOrder::MostActive | CounterOrder::Lifetime => {
                rows.sort_by_key(|r| Reverse(activity(order, &r.counter, &r.entries, day)));
            }
        }
        rows
    }
}

/// The tap on `row`, counting on `day`.
fn tap_of<'row>(row: &'row Row, day: NaiveDate, prefs: &Preferences) -> Tap<'row> {
    Tap {
        name: row.counter.name.as_str(),
        entries: &row.entries,
        goal: row.counter.goal,
        own: row.counter.celebration,
        app: prefs.celebration,
        day,
    }
}

fn across(rows: &[Row], day: NaiveDate, prefs: &Preferences) -> Across {
    let days: Vec<CounterDays<'_>> = rows.iter().map(Row::days).collect();
    stats::across_counters(&days, day, prefs)
}

/// Whether every counter with a goal has met it on `day`. False when none
/// has a goal: there is no day to finish.
fn all_met(rows: &[Row], day: NaiveDate, prefs: &Preferences) -> bool {
    let across = across(rows, day, prefs);
    across.with_goals > 0 && across.goals_met == across.with_goals
}

/// The key a counter sorts by in an activity order, highest first: this
/// year's total for Most active, the lifetime total for Lifetime, and zero
/// for the orders that need no figures.
pub fn activity(
    order: CounterOrder,
    counter: &Counter,
    entries: &[DayCount],
    day: NaiveDate,
) -> u32 {
    // Preferences only reach streaks and consistency, and neither is a sort
    // key, so the default is the whole truth.
    let summary = || stats::summarize(entries, counter.goal, day);
    match order {
        CounterOrder::Lifetime => summary().lifetime,
        CounterOrder::MostActive => summary().this_year.total,
        CounterOrder::Created | CounterOrder::Name => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::counter::models::{CounterName, Goal, Step};
    use crate::domain::preferences;
    use crate::screens::toast::ToastKind;

    fn day(m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, m, d).unwrap()
    }

    fn counter(id: i64, name: &str, goal: Option<Goal>) -> Counter {
        Counter {
            id: CounterId(id),
            name: CounterName::new(name).unwrap(),
            goal,
            step: Step::new(1).unwrap(),
            big_step: None,
            celebration: None,
            created_on: day(1, 1),
        }
    }

    fn row(id: i64, name: &str, goal: Option<Goal>, entries: &[(u32, u32, u32)]) -> Row {
        let entries = entries
            .iter()
            .map(|&(m, d, count)| DayCount {
                day: day(m, d),
                count,
            })
            .collect();
        Row::new(counter(id, name, goal), entries)
    }

    fn prefs() -> Preferences {
        Preferences {
            celebration: Celebration::Confetti,
            ..Preferences::default()
        }
    }

    fn loaded(rows: Vec<Row>) -> State {
        let mut state = State::default();
        assert!(matches!(
            state.update(Action::Load)[..],
            [Effect::LoadCounters]
        ));
        assert!(state.update(Action::Loaded(Ok(rows))).is_empty());
        state
    }

    fn tap(state: &mut State, id: i64, delta: i64) -> Vec<Effect> {
        state.update(Action::Tap {
            id: CounterId(id),
            delta,
            day: day(11, 8),
            prefs: prefs(),
        })
    }

    #[test]
    fn load_holds_the_rows_and_a_failure_keeps_them() {
        let mut state = loaded(vec![row(1, "Pushups", None, &[(11, 8, 3)])]);
        assert!(!state.is_empty());
        assert_eq!(state.error(), None);
        assert!(
            state
                .update(Action::Loaded(Err("disk".to_owned())))
                .is_empty()
        );
        assert_eq!(state.error(), Some("disk"));
        assert!(!state.is_empty(), "the last list stays on screen");
        assert!(State::default().is_empty());
    }

    #[test]
    fn the_tiles_are_the_domains_sum_of_the_held_rows() {
        let rows = vec![
            row(
                1,
                "Pushups",
                Some(Goal::PerDay(10)),
                &[(11, 8, 10), (11, 7, 4)],
            ),
            row(2, "Squats", Some(Goal::PerDay(10)), &[(11, 8, 2)]),
            row(3, "Pages", None, &[(11, 8, 5)]),
        ];
        let want = stats::across_counters(
            &rows
                .iter()
                .map(|r| CounterDays {
                    goal: r.counter.goal,
                    entries: &r.entries,
                })
                .collect::<Vec<_>>(),
            day(11, 8),
            &prefs(),
        );
        let got = loaded(rows).across(day(11, 8), &prefs());
        assert_eq!(got, want);
        assert_eq!(
            (got.logged_today, got.goals_met, got.with_goals),
            (17, 1, 2)
        );
    }

    #[test]
    fn rows_follow_the_order() {
        let state = loaded(vec![
            row(1, "squats", None, &[(11, 8, 5), (1, 2, 1)]),
            row(2, "Burpees", None, &[(11, 8, 1)]),
            row(3, "pages", None, &[(3, 1, 50)]),
        ]);
        let ids = |order| -> Vec<i64> {
            state
                .rows(order, day(11, 8))
                .iter()
                .map(|r| r.counter.id.0)
                .collect()
        };
        assert_eq!(ids(CounterOrder::Created), [1, 2, 3]);
        assert_eq!(ids(CounterOrder::Name), [2, 3, 1]);
        assert_eq!(ids(CounterOrder::Lifetime), [3, 1, 2]);
        assert_eq!(ids(CounterOrder::MostActive), [3, 1, 2]);
    }

    #[test]
    fn activity_reads_this_year_or_the_lifetime() {
        let r = row(1, "a", None, &[(11, 8, 5)]);
        let mut old = r.entries.clone();
        old.push(DayCount {
            day: NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
            count: 7,
        });
        assert_eq!(
            activity(CounterOrder::Lifetime, &r.counter, &old, day(11, 8)),
            12
        );
        assert_eq!(
            activity(CounterOrder::MostActive, &r.counter, &old, day(11, 8)),
            5
        );
        assert_eq!(
            activity(CounterOrder::Name, &r.counter, &old, day(11, 8)),
            0
        );
    }

    #[test]
    fn a_minus_on_an_empty_day_says_minus_zero_and_writes_nothing() {
        let mut state = loaded(vec![row(1, "Pushups", None, &[(11, 7, 3)])]);
        let effects = tap(&mut state, 1, -1);
        assert!(
            matches!(&effects[..], [Effect::Toast(t)] if *t == Toast::info("Pushups -0")),
            "{effects:?}"
        );
    }

    #[test]
    fn a_tap_writes_the_trimmed_delta() {
        let mut state = loaded(vec![row(1, "Pushups", None, &[(11, 8, 3)])]);
        let effects = tap(&mut state, 1, -5);
        assert!(
            matches!(&effects[..], [Effect::Adjust { id: CounterId(1), delta: -3, day: d }] if *d == day(11, 8)),
            "{effects:?}"
        );
    }

    #[test]
    fn a_plain_tap_reloads_and_says_nothing() {
        let mut state = loaded(vec![row(
            1,
            "Pushups",
            Some(Goal::PerDay(10)),
            &[(11, 8, 3)],
        )]);
        tap(&mut state, 1, 1);
        let effects = state.update(Action::Adjusted(Ok(())));
        assert!(
            matches!(&effects[..], [Effect::LoadCounters]),
            "{effects:?}"
        );
    }

    #[test]
    fn finishing_one_goal_of_two_celebrates_with_the_usual_line() {
        let mut state = loaded(vec![
            row(1, "Pushups", Some(Goal::PerDay(10)), &[(11, 8, 9)]),
            row(2, "Squats", Some(Goal::PerDay(10)), &[(11, 8, 2)]),
        ]);
        tap(&mut state, 1, 1);
        let effects = state.update(Action::Adjusted(Ok(())));
        assert!(
            matches!(
                &effects[..],
                [
                    Effect::Celebrate {
                        how: Celebration::Confetti,
                        saying: None
                    },
                    Effect::LoadCounters
                ]
            ),
            "{effects:?}"
        );
    }

    #[test]
    fn finishing_the_last_goal_says_all_done_from_the_held_rows() {
        let mut state = loaded(vec![
            row(1, "Pushups", Some(Goal::PerDay(10)), &[(11, 8, 9)]),
            row(2, "Squats", Some(Goal::PerDay(10)), &[(11, 8, 10)]),
            row(3, "Pages", None, &[]),
        ]);
        tap(&mut state, 1, 1);
        let effects = state.update(Action::Adjusted(Ok(())));
        let [
            Effect::Celebrate {
                saying: Some(Saying::Rotating(pick)),
                ..
            },
            Effect::LoadCounters,
        ] = &effects[..]
        else {
            panic!("{effects:?}");
        };
        assert_eq!(pick(3), preferences::all_done_line(3));
    }

    #[test]
    fn a_write_that_fails_says_why_and_does_not_reload() {
        let mut state = loaded(vec![row(
            1,
            "Pushups",
            Some(Goal::PerDay(10)),
            &[(11, 8, 9)],
        )]);
        tap(&mut state, 1, 1);
        let effects = state.update(Action::Adjusted(Err("locked".to_owned())));
        assert!(
            matches!(&effects[..], [Effect::Toast(t)] if t.kind() == ToastKind::Error && t.text() == "locked"),
            "{effects:?}"
        );
        assert!(
            state.update(Action::Adjusted(Ok(()))).is_empty(),
            "nothing is pending"
        );
    }

    #[test]
    fn open_edit_and_create_pass_through_and_an_unknown_counter_does_nothing() {
        let mut state = loaded(vec![row(1, "Pushups", None, &[])]);
        assert!(matches!(
            state.update(Action::Open(CounterId(1)))[..],
            [Effect::Open(CounterId(1))]
        ));
        assert!(
            matches!(&state.update(Action::Edit(CounterId(1)))[..], [Effect::Edit(c)] if c.id == CounterId(1))
        );
        assert!(matches!(state.update(Action::Create)[..], [Effect::Create]));
        assert!(state.update(Action::Edit(CounterId(9))).is_empty());
        assert!(tap(&mut state, 9, 1).is_empty());
    }

    #[test]
    fn a_row_adds_to_its_day_or_starts_one() {
        let mut r = row(1, "a", None, &[(11, 8, 3)]);
        r.add(day(11, 8), 2);
        r.add(day(11, 9), 4);
        r.add(day(11, 8), -5);
        assert_eq!(
            r.entries,
            [
                DayCount {
                    day: day(11, 8),
                    count: 0
                },
                DayCount {
                    day: day(11, 9),
                    count: 4
                }
            ]
        );
    }
}
