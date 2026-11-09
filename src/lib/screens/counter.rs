//! One counter's screen: its load, the Trends and Bests picks, the step
//! buttons and Delete.
//!
//! The counter, its entries and its events load once per
//! [`Effect::LoadCounter`]; the cards draw from what the model holds, and a
//! tap is judged on it through [`crate::screens::tap`].

use chrono::NaiveDate;

use crate::domain::counter::models::{Counter, DayCount, Event};
use crate::domain::preferences::{Celebration, Preferences};
use crate::screens::saying::Saying;
use crate::screens::tap::{Outcome, Plan, Tap};
use crate::screens::toast::Toast;

/// Which series the Trends card draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Trend {
    /// Monday to today.
    #[default]
    ThisWeek,
    /// The last 60 days.
    Daily,
    /// Week by week.
    Weekly,
    /// Month by month.
    Monthly,
    /// The shape of a week.
    Weekday,
    /// The shape of a day.
    Hourly,
}

/// Which best the Bests card shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Best {
    /// The biggest day.
    #[default]
    Day,
    /// The biggest week.
    Week,
    /// The biggest month.
    Month,
}

/// The toast after a delete.
pub fn deleted(name: &str) -> String {
    format!("Deleted {name}")
}

/// The store's answer to [`Effect::LoadCounter`], one result per read.
#[derive(Debug, Clone)]
pub struct Loaded {
    /// The counter, `None` when no counter has the id.
    pub counter: Result<Option<Counter>, String>,
    /// Its entries.
    pub entries: Result<Vec<DayCount>, String>,
    /// Its taps.
    pub events: Result<Vec<Event>, String>,
}

/// A tap written to the store and waiting for its answer.
#[derive(Debug, Clone, Copy)]
struct Pending {
    day: NaiveDate,
    applied: i64,
    prefs: Preferences,
}

/// The counter screen's model.
#[derive(Debug, Default)]
pub struct State {
    counter: Option<Counter>,
    entries: Vec<DayCount>,
    events: Vec<Event>,
    notice: Option<String>,
    trend: Trend,
    best: Best,
    /// The tap waiting on the store. Its entries are [`State::entries`],
    /// which stay as they were before the write until the reload answers.
    pending: Option<Pending>,
}

/// What happened on the counter screen.
#[derive(Debug, Clone)]
pub enum Action {
    /// The screen opened on a counter, by mount or by moving from another
    /// counter: start fresh and load it.
    Open,
    /// The screen came back to the foreground, or the edit sheet saved.
    Load,
    /// The store's answer to [`Effect::LoadCounter`].
    Loaded(Loaded),
    /// A step button: `delta`, counting on `day`.
    Tap {
        /// The step asked for, negative for a minus.
        delta: i64,
        /// The day the tap counts on.
        day: NaiveDate,
        /// The app's preferences at the tap.
        prefs: Preferences,
    },
    /// The store's answer to [`Effect::Adjust`].
    Adjusted(Result<(), String>),
    /// The store's answer to [`Effect::AskAllMet`].
    AllMet(bool),
    /// A Trends chip.
    PickTrend(Trend),
    /// A Bests chip.
    PickBest(Best),
    /// Delete, confirmed.
    Delete,
    /// The store's answer to [`Effect::DeleteCounter`].
    Deleted(Result<(), String>),
}

/// What the counter screen asks the client to do.
#[derive(Debug, Clone)]
pub enum Effect {
    /// Read the counter, its entries and its events, answering
    /// [`Action::Loaded`].
    LoadCounter,
    /// Add `delta` on `day`, answering [`Action::Adjusted`].
    Adjust {
        /// The day it counts on.
        day: NaiveDate,
        /// The change, already trimmed so the day stays at zero or above.
        delta: i64,
    },
    /// Ask the store whether every counter's goal is met on `day`,
    /// answering [`Action::AllMet`].
    AskAllMet {
        /// The day.
        day: NaiveDate,
        /// The preferences at the tap.
        prefs: Preferences,
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
    /// Delete the counter, answering [`Action::Deleted`].
    DeleteCounter,
    /// Tell the rest of the app the store changed.
    StoreChanged,
    /// Go back to Home.
    GoHome,
}

impl State {
    /// Applies `action` and returns what the client should do, in order.
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Open => {
                *self = Self::default();
                vec![Effect::LoadCounter]
            }
            Action::Load => vec![Effect::LoadCounter],
            Action::Loaded(loaded) => {
                self.loaded(loaded);
                Vec::new()
            }
            Action::Tap { delta, day, prefs } => self.tap(delta, day, prefs),
            Action::Adjusted(result) => self.adjusted(result),
            Action::AllMet(met) => match self.pending.take() {
                Some(pending) => self.celebrate(pending, met),
                None => Vec::new(),
            },
            Action::PickTrend(trend) => {
                self.trend = trend;
                Vec::new()
            }
            Action::PickBest(best) => {
                self.best = best;
                Vec::new()
            }
            Action::Delete if self.counter.is_some() => vec![Effect::DeleteCounter],
            Action::Delete => Vec::new(),
            Action::Deleted(Ok(())) => {
                let name = self
                    .counter
                    .as_ref()
                    .map(|c| c.name.as_str())
                    .unwrap_or_default();
                vec![
                    Effect::Toast(Toast::success(deleted(name))),
                    Effect::StoreChanged,
                    Effect::GoHome,
                ]
            }
            Action::Deleted(Err(e)) => vec![Effect::Toast(Toast::error(e))],
        }
    }

    /// Takes each read that worked; a failed one keeps what was there and
    /// leaves its message, the last failure winning.
    fn loaded(&mut self, loaded: Loaded) {
        match loaded.counter {
            Ok(counter) => self.counter = counter,
            Err(e) => self.notice = Some(e),
        }
        match loaded.entries {
            Ok(entries) => self.entries = entries,
            Err(e) => self.notice = Some(e),
        }
        match loaded.events {
            Ok(events) => self.events = events,
            Err(e) => self.notice = Some(e),
        }
    }

    fn tap_on(&self, day: NaiveDate, prefs: &Preferences) -> Option<Tap<'_>> {
        let counter = self.counter.as_ref()?;
        Some(Tap {
            name: counter.name.as_str(),
            entries: &self.entries,
            goal: counter.goal,
            own: counter.celebration,
            app: prefs.celebration,
            day,
        })
    }

    fn tap(&mut self, delta: i64, day: NaiveDate, prefs: Preferences) -> Vec<Effect> {
        let Some(tap) = self.tap_on(day, &prefs) else {
            return Vec::new();
        };
        match tap.plan(delta) {
            Plan::Nothing(toast) => vec![Effect::Toast(toast)],
            Plan::Apply(applied) => {
                self.pending = Some(Pending {
                    day,
                    applied,
                    prefs,
                });
                vec![Effect::Adjust {
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
        let asks = self
            .tap_on(pending.day, &pending.prefs)
            .is_some_and(|tap| tap.needs_all_met(pending.applied));
        if asks {
            self.pending = Some(pending);
            return vec![Effect::AskAllMet {
                day: pending.day,
                prefs: pending.prefs,
            }];
        }
        self.celebrate(pending, false)
    }

    /// The applied tap's celebration, if any, then the reload.
    fn celebrate(&self, pending: Pending, all_met: bool) -> Vec<Effect> {
        let outcome = self
            .tap_on(pending.day, &pending.prefs)
            .map(|tap| tap.outcome(pending.applied, || all_met));
        let mut effects = Vec::new();
        if let Some(Outcome::Celebrate { how, saying }) = outcome {
            effects.push(Effect::Celebrate { how, saying });
        }
        effects.push(Effect::LoadCounter);
        effects
    }

    /// The counter, once loaded; `None` when no counter has the id.
    pub fn counter(&self) -> Option<&Counter> {
        self.counter.as_ref()
    }

    /// Every day the counter has logged.
    pub fn entries(&self) -> &[DayCount] {
        &self.entries
    }

    /// Every tap, for the hour-of-day series.
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Why a read last failed, if one did.
    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    /// The Trends pick.
    pub fn trend(&self) -> Trend {
        self.trend
    }

    /// The Bests pick.
    pub fn best(&self) -> Best {
        self.best
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::counter::models::{CounterId, CounterName, Goal, Step};
    use crate::domain::preferences;
    use crate::screens::toast::ToastKind;

    fn day(m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, m, d).unwrap()
    }

    fn counter(goal: Option<Goal>) -> Counter {
        Counter {
            id: CounterId(1),
            name: CounterName::new("Pushups").unwrap(),
            goal,
            step: Step::new(1).unwrap(),
            big_step: None,
            celebration: None,
            created_on: day(1, 1),
        }
    }

    fn prefs() -> Preferences {
        Preferences {
            celebration: Celebration::Confetti,
            ..Preferences::default()
        }
    }

    fn loaded(goal: Option<Goal>, entries: &[(u32, u32, u32)]) -> State {
        let entries = entries
            .iter()
            .map(|&(m, d, count)| DayCount {
                day: day(m, d),
                count,
            })
            .collect();
        let mut state = State::default();
        assert!(matches!(
            state.update(Action::Load)[..],
            [Effect::LoadCounter]
        ));
        let answer = Loaded {
            counter: Ok(Some(counter(goal))),
            entries: Ok(entries),
            events: Ok(Vec::new()),
        };
        assert!(state.update(Action::Loaded(answer)).is_empty());
        state
    }

    fn tap(state: &mut State, delta: i64) -> Vec<Effect> {
        state.update(Action::Tap {
            delta,
            day: day(11, 8),
            prefs: prefs(),
        })
    }

    #[test]
    fn the_copy_matches_the_app() {
        assert_eq!(deleted("Pushups"), "Deleted Pushups");
    }

    #[test]
    fn load_holds_the_counter_and_each_failure_leaves_a_notice() {
        let state = loaded(None, &[(11, 8, 3)]);
        assert_eq!(state.counter().map(|c| c.id), Some(CounterId(1)));
        assert_eq!(state.entries().len(), 1);
        assert_eq!(state.notice(), None);

        let mut state = loaded(None, &[(11, 8, 3)]);
        let answer = Loaded {
            counter: Err("counter".to_owned()),
            entries: Err("entries".to_owned()),
            events: Ok(Vec::new()),
        };
        state.update(Action::Loaded(answer));
        assert_eq!(state.notice(), Some("entries"), "the last failure wins");
        assert!(
            state.counter().is_some(),
            "a failed read keeps what was there"
        );
        assert_eq!(state.entries().len(), 1);
    }

    #[test]
    fn no_counter_with_the_id_loads_as_none() {
        let mut state = State::default();
        let answer = Loaded {
            counter: Ok(None),
            entries: Ok(Vec::new()),
            events: Ok(Vec::new()),
        };
        state.update(Action::Loaded(answer));
        assert!(state.counter().is_none());
    }

    #[test]
    fn the_pickers_hold_their_pick() {
        let mut state = State::default();
        assert_eq!((state.trend(), state.best()), (Trend::ThisWeek, Best::Day));
        assert!(state.update(Action::PickTrend(Trend::Hourly)).is_empty());
        assert!(state.update(Action::PickBest(Best::Month)).is_empty());
        assert_eq!((state.trend(), state.best()), (Trend::Hourly, Best::Month));
    }

    #[test]
    fn a_minus_on_an_empty_day_says_minus_zero() {
        let mut state = loaded(None, &[(11, 7, 3)]);
        let effects = tap(&mut state, -1);
        assert!(
            matches!(&effects[..], [Effect::Toast(t)] if *t == Toast::info("Pushups -0")),
            "{effects:?}"
        );
    }

    #[test]
    fn a_plain_tap_writes_and_reloads_quietly() {
        let mut state = loaded(Some(Goal::PerDay(10)), &[(11, 8, 3)]);
        let effects = tap(&mut state, -5);
        assert!(
            matches!(&effects[..], [Effect::Adjust { delta: -3, .. }]),
            "{effects:?}"
        );
        let effects = state.update(Action::Adjusted(Ok(())));
        assert!(matches!(&effects[..], [Effect::LoadCounter]), "{effects:?}");
    }

    #[test]
    fn finishing_the_day_asks_the_store_about_every_goal() {
        let mut state = loaded(Some(Goal::PerDay(10)), &[(11, 8, 9)]);
        tap(&mut state, 1);
        let effects = state.update(Action::Adjusted(Ok(())));
        assert!(
            matches!(&effects[..], [Effect::AskAllMet { day: d, .. }] if *d == day(11, 8)),
            "{effects:?}"
        );
        let effects = state.update(Action::AllMet(false));
        assert!(
            matches!(
                &effects[..],
                [
                    Effect::Celebrate {
                        how: Celebration::Confetti,
                        saying: None
                    },
                    Effect::LoadCounter
                ]
            ),
            "{effects:?}"
        );
    }

    #[test]
    fn finishing_every_goal_says_all_done() {
        let mut state = loaded(Some(Goal::PerDay(10)), &[(11, 8, 9)]);
        tap(&mut state, 1);
        state.update(Action::Adjusted(Ok(())));
        let effects = state.update(Action::AllMet(true));
        let [
            Effect::Celebrate {
                saying: Some(Saying::Rotating(pick)),
                ..
            },
            Effect::LoadCounter,
        ] = &effects[..]
        else {
            panic!("{effects:?}");
        };
        assert_eq!(pick(1), preferences::all_done_line(1));
    }

    #[test]
    fn a_year_mark_celebrates_without_asking() {
        let mut state = loaded(Some(Goal::PerYear(1000)), &[(11, 1, 99)]);
        tap(&mut state, 1);
        let effects = state.update(Action::Adjusted(Ok(())));
        assert!(
            matches!(&effects[..], [Effect::Celebrate { saying: Some(Saying::Fixed(t)), .. }, Effect::LoadCounter] if t == "10% of the year"),
            "{effects:?}"
        );
    }

    #[test]
    fn a_failed_write_says_why() {
        let mut state = loaded(Some(Goal::PerDay(10)), &[(11, 8, 9)]);
        tap(&mut state, 1);
        let effects = state.update(Action::Adjusted(Err("locked".to_owned())));
        assert!(
            matches!(&effects[..], [Effect::Toast(t)] if t.kind() == ToastKind::Error && t.text() == "locked"),
            "{effects:?}"
        );
        assert!(
            state.update(Action::AllMet(true)).is_empty(),
            "nothing is pending"
        );
    }

    #[test]
    fn delete_toasts_tells_the_app_and_goes_home() {
        let mut state = loaded(None, &[]);
        assert!(matches!(
            state.update(Action::Delete)[..],
            [Effect::DeleteCounter]
        ));
        let effects = state.update(Action::Deleted(Ok(())));
        assert!(
            matches!(&effects[..], [Effect::Toast(t), Effect::StoreChanged, Effect::GoHome] if *t == Toast::success("Deleted Pushups")),
            "{effects:?}"
        );
        let effects = state.update(Action::Deleted(Err("busy".to_owned())));
        assert!(
            matches!(&effects[..], [Effect::Toast(t)] if *t == Toast::error("busy")),
            "{effects:?}"
        );
    }

    #[test]
    fn nothing_happens_before_the_counter_loads() {
        let mut state = State::default();
        assert!(tap(&mut state, 1).is_empty());
        assert!(state.update(Action::Delete).is_empty());
    }

    #[test]
    fn opening_another_counter_starts_fresh() {
        let mut state = loaded(Some(Goal::PerDay(10)), &[(11, 8, 3)]);
        state.update(Action::PickTrend(Trend::Hourly));
        state.update(Action::PickBest(Best::Week));
        let failed = Loaded {
            counter: Err("gone".to_owned()),
            entries: Ok(Vec::new()),
            events: Ok(Vec::new()),
        };
        state.update(Action::Loaded(failed));
        assert!(matches!(
            state.update(Action::Open)[..],
            [Effect::LoadCounter]
        ));
        assert!(state.counter().is_none(), "the last counter is not drawn");
        assert_eq!(state.entries().len(), 0, "no entries are drawn");
        assert_eq!(state.notice(), None);
        assert_eq!((state.trend(), state.best()), (Trend::ThisWeek, Best::Day));
    }
}
