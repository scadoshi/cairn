//! The full backup: one JSON file holding every counter with its settings,
//! its daily entries and every timestamped tap, so a fresh install can take
//! back everything the store held.
//!
//! The file is versioned. [`parse`] reads the version before anything else,
//! so a backup from a newer Cairn is refused by name rather than by a shape
//! error, then checks every field with the same constructors the app uses, so
//! a [`Backup`] that exists is one the store can take whole.
//!
//! The wire structs below are plain data; only [`Backup`] and
//! [`CounterBackup`] carry validated domain types.

use std::collections::BTreeSet;

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{
    Counter, CounterName, CounterStore, DayCount, Event, Goal, Step, StoreError, check_big_step,
};
use crate::domain::preferences::Celebration;

/// What the `format` field holds, so a stray JSON file is not mistaken for a
/// backup.
pub const FORMAT: &str = "cairn-backup";

/// The version this build writes and the newest it reads.
pub const VERSION: u32 = 1;

/// How a day is written: the store's own text form.
pub const DAY_FORMAT: &str = "%Y-%m-%d";

/// How a tap's time is written: the store's own text form, to the second.
pub const AT_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

/// Why a file is not a backup this build can import. Nothing is written
/// while any of these stands.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BackupError {
    /// The text is not JSON, or not the JSON shape of a backup.
    #[error("not a cairn backup: {0}")]
    Shape(String),
    /// The JSON names some other format.
    #[error("not a cairn backup: format is {0:?}")]
    Format(String),
    /// A newer Cairn wrote it, in a version this build can't read.
    #[error("backup version {0} is newer than this cairn reads ({VERSION}); update the app")]
    Newer(u32),
    /// Version 0, which no Cairn writes.
    #[error("backup version {0} is not one cairn writes")]
    Version(u32),
    /// One counter in it is invalid; `counter` is its position, from 1.
    #[error("counter {counter} ({name}): {reason}")]
    Counter {
        /// Position in the file, from 1.
        counter: usize,
        /// The name as the file gives it.
        name: String,
        /// What is wrong.
        reason: String,
    },
}

/// Every counter in the store, each with all it holds. Built by
/// [`Backup::from_store`] or [`parse`]; written by [`Backup::to_json`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backup {
    /// The counters, in the store's order.
    pub counters: Vec<CounterBackup>,
}

/// One counter as a backup holds it: its settings, without the store id,
/// which the importing store assigns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterBackup {
    /// Display name.
    pub name: CounterName,
    /// Optional target.
    pub goal: Option<Goal>,
    /// How much one tap adds.
    pub step: Step,
    /// The second, larger step, if it has one.
    pub big_step: Option<Step>,
    /// Its own goal animation; `None` follows the app-wide setting.
    pub celebration: Option<Celebration>,
    /// Day the counter was created.
    pub created_on: NaiveDate,
    /// Daily totals, oldest first, one per day.
    pub entries: Vec<DayCount>,
    /// Every tap, oldest first.
    pub events: Vec<Event>,
}

impl Backup {
    /// Reads every counter, its entries and its taps out of `store`.
    ///
    /// # Errors
    ///
    /// The [`StoreError`] of the first query that fails.
    pub fn from_store(store: &(impl CounterStore + ?Sized)) -> Result<Self, StoreError> {
        let counters = store
            .list_counters()?
            .into_iter()
            .map(|c| CounterBackup::from_store(store, c))
            .collect::<Result<Vec<CounterBackup>, StoreError>>()?;
        Ok(Self { counters })
    }

    /// The backup as pretty-printed JSON, the file's whole text.
    ///
    /// # Errors
    ///
    /// [`BackupError::Shape`] if serialization fails, which plain strings and
    /// numbers do not.
    pub fn to_json(&self) -> Result<String, BackupError> {
        let wire = WireBackup {
            format: FORMAT.to_string(),
            version: VERSION,
            counters: self.counters.iter().map(WireCounter::from).collect(),
        };
        serde_json::to_string_pretty(&wire).map_err(|e| BackupError::Shape(e.to_string()))
    }

    /// The names of counters in this backup that `existing` already has,
    /// each once, in the backup's order. Empty means an import adds without
    /// touching anything.
    pub fn conflicts(&self, existing: &[Counter]) -> Vec<CounterName> {
        let have: BTreeSet<&str> = existing.iter().map(|c| c.name.as_str()).collect();
        let mut seen = BTreeSet::new();
        self.counters
            .iter()
            .filter(|c| have.contains(c.name.as_str()) && seen.insert(c.name.as_str()))
            .map(|c| c.name.clone())
            .collect()
    }
}

impl CounterBackup {
    fn from_store(store: &(impl CounterStore + ?Sized), c: Counter) -> Result<Self, StoreError> {
        Ok(Self {
            entries: store.entries(c.id)?,
            events: store.events(c.id)?,
            name: c.name,
            goal: c.goal,
            step: c.step,
            big_step: c.big_step,
            celebration: c.celebration,
            created_on: c.created_on,
        })
    }
}

/// What an import does with a backup counter whose name the store already
/// has. The rest are added either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnConflict {
    /// The store's counter stays as it is and the backup's is skipped.
    Keep,
    /// The store's counter, with its history, gives way to the backup's.
    Replace,
}

/// What an import did, counted in backup counters.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Restored {
    /// Added under a name the store did not have.
    pub added: usize,
    /// Added in place of the store's counters of the same name.
    pub replaced: usize,
    /// Skipped because the store kept its own.
    pub kept: usize,
}

/// Reads and checks a backup file's text: JSON, the format, the version, then
/// every counter's name, goal, steps, animation, dates and counts.
///
/// # Errors
///
/// The first [`BackupError`] found; nothing past it is checked.
pub fn parse(text: &str) -> Result<Backup, BackupError> {
    let header: WireHeader =
        serde_json::from_str(text).map_err(|e| BackupError::Shape(e.to_string()))?;
    if header.format != FORMAT {
        return Err(BackupError::Format(header.format));
    }
    match header.version {
        0 => return Err(BackupError::Version(0)),
        v if v > VERSION => return Err(BackupError::Newer(v)),
        _ => {}
    }
    let wire: WireBackup =
        serde_json::from_str(text).map_err(|e| BackupError::Shape(e.to_string()))?;
    let counters = wire
        .counters
        .into_iter()
        .enumerate()
        .map(|(i, c)| {
            let name = c.name.clone();
            c.validate().map_err(|reason| BackupError::Counter {
                counter: i.saturating_add(1),
                name,
                reason,
            })
        })
        .collect::<Result<Vec<CounterBackup>, BackupError>>()?;
    Ok(Backup { counters })
}

/// The two fields read first, whatever else the file holds.
#[derive(Deserialize)]
struct WireHeader {
    format: String,
    version: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireBackup {
    format: String,
    version: u32,
    counters: Vec<WireCounter>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireCounter {
    name: String,
    goal: Option<WireGoal>,
    step: u32,
    big_step: Option<u32>,
    celebration: Option<String>,
    created_on: String,
    entries: Vec<WireDay>,
    events: Vec<WireEvent>,
}

/// A goal as `{"per": "year" | "week" | "day", "amount": n}`.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireGoal {
    per: String,
    amount: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireDay {
    day: String,
    count: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireEvent {
    at: String,
    delta: i64,
}

impl From<&CounterBackup> for WireCounter {
    fn from(c: &CounterBackup) -> Self {
        let goal = c.goal.map(|g| {
            let (per, amount) = match g {
                Goal::PerYear(n) => ("year", n),
                Goal::PerWeek(n) => ("week", n),
                Goal::PerDay(n) => ("day", n),
            };
            WireGoal {
                per: per.to_string(),
                amount,
            }
        });
        Self {
            name: c.name.as_str().to_string(),
            goal,
            step: c.step.get(),
            big_step: c.big_step.map(Step::get),
            celebration: c.celebration.map(|k| k.key().to_string()),
            created_on: c.created_on.format(DAY_FORMAT).to_string(),
            entries: c
                .entries
                .iter()
                .map(|e| WireDay {
                    day: e.day.format(DAY_FORMAT).to_string(),
                    count: e.count,
                })
                .collect(),
            events: c
                .events
                .iter()
                .map(|e| WireEvent {
                    at: e.at.format(AT_FORMAT).to_string(),
                    delta: e.delta,
                })
                .collect(),
        }
    }
}

impl WireCounter {
    /// The counter with every field checked, or what is wrong with it.
    fn validate(self) -> Result<CounterBackup, String> {
        let name = CounterName::new(&self.name).map_err(|e| e.to_string())?;
        let goal = self
            .goal
            .map(|g| match g.per.as_str() {
                "year" => Goal::per_year(g.amount).map_err(|e| e.to_string()),
                "week" => Goal::per_week(g.amount).map_err(|e| e.to_string()),
                "day" => Goal::per_day(g.amount).map_err(|e| e.to_string()),
                other => Err(format!("goal per {other:?} is not year, week or day")),
            })
            .transpose()?;
        let step = Step::new(self.step).map_err(|e| format!("step {}: {e}", self.step))?;
        let big_step = self
            .big_step
            .map(|b| Step::new(b).map_err(|e| format!("big step {b}: {e}")))
            .transpose()?;
        let big_step = check_big_step(step, big_step).map_err(|e| e.to_string())?;
        let celebration = self
            .celebration
            .map(|k| {
                Celebration::from_key(&k).ok_or_else(|| format!("unknown goal animation {k:?}"))
            })
            .transpose()?;
        let created_on = day(&self.created_on, "created on")?;
        let mut days = BTreeSet::new();
        let mut entries = self
            .entries
            .into_iter()
            .map(|e| {
                let d = day(&e.day, "entry")?;
                if e.count == 0 {
                    return Err(format!("entry {} has a count of 0", e.day));
                }
                if !days.insert(d) {
                    return Err(format!("entry {} appears twice", e.day));
                }
                Ok(DayCount {
                    day: d,
                    count: e.count,
                })
            })
            .collect::<Result<Vec<DayCount>, String>>()?;
        entries.sort_by_key(|e| e.day);
        let mut events = self
            .events
            .into_iter()
            .map(|e| {
                let at = NaiveDateTime::parse_from_str(&e.at, AT_FORMAT)
                    .map_err(|err| format!("tap time {:?}: {err}", e.at))?;
                Ok(Event { at, delta: e.delta })
            })
            .collect::<Result<Vec<Event>, String>>()?;
        events.sort_by_key(|e| e.at);
        Ok(CounterBackup {
            name,
            goal,
            step,
            big_step,
            celebration,
            created_on,
            entries,
            events,
        })
    }
}

/// A `YYYY-MM-DD` day, or what is wrong with `text`, naming `what` it is.
fn day(text: &str, what: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(text, DAY_FORMAT).map_err(|e| format!("{what} date {text:?}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn sample() -> Backup {
        Backup {
            counters: vec![CounterBackup {
                name: CounterName::new("Push-ups").unwrap(),
                goal: Some(Goal::per_day(50).unwrap()),
                step: Step::new(10).unwrap(),
                big_step: Some(Step::new(25).unwrap()),
                celebration: Celebration::ALL.first().copied(),
                created_on: d(2026, 1, 2),
                entries: vec![DayCount {
                    day: d(2026, 1, 2),
                    count: 30,
                }],
                events: vec![Event {
                    at: d(2026, 1, 2).and_hms_opt(7, 30, 5).unwrap(),
                    delta: 30,
                }],
            }],
        }
    }

    fn with(edit: impl FnOnce(&mut serde_json::Value)) -> String {
        let mut v: serde_json::Value = serde_json::from_str(&sample().to_json().unwrap()).unwrap();
        edit(&mut v);
        v.to_string()
    }

    #[test]
    fn json_reads_back_to_the_same_backup() {
        let backup = sample();
        assert_eq!(parse(&backup.to_json().unwrap()).unwrap(), backup);
    }

    #[test]
    fn the_file_names_its_format_and_version() {
        let v: serde_json::Value = serde_json::from_str(&sample().to_json().unwrap()).unwrap();
        assert_eq!(v["format"], FORMAT);
        assert_eq!(v["version"], VERSION);
    }

    #[test]
    fn a_newer_version_is_refused_by_name() {
        let text = with(|v| {
            v["version"] = 2.into();
            v["counters"] = "a shape v1 can't read".into();
        });
        assert_eq!(parse(&text), Err(BackupError::Newer(2)));
    }

    #[test]
    fn other_json_is_not_a_backup() {
        assert!(matches!(parse("{\"a\": 1}"), Err(BackupError::Shape(_))));
        assert!(matches!(parse("day,count\n"), Err(BackupError::Shape(_))));
        let text = with(|v| v["format"] = "something-else".into());
        assert_eq!(
            parse(&text),
            Err(BackupError::Format("something-else".into()))
        );
        let text = with(|v| v["version"] = 0.into());
        assert_eq!(parse(&text), Err(BackupError::Version(0)));
    }

    /// A change to a backup's JSON.
    type Edit = Box<dyn Fn(&mut serde_json::Value)>;

    #[test]
    fn every_field_is_checked() {
        let bad: [(&str, Edit); 9] = [
            ("name", Box::new(|v| v["counters"][0]["name"] = "".into())),
            ("step", Box::new(|v| v["counters"][0]["step"] = 7.into())),
            (
                "big step",
                Box::new(|v| v["counters"][0]["big_step"] = 5.into()),
            ),
            (
                "goal",
                Box::new(|v| v["counters"][0]["goal"]["per"] = "month".into()),
            ),
            (
                "animation",
                Box::new(|v| v["counters"][0]["celebration"] = "fireworks!".into()),
            ),
            (
                "created on",
                Box::new(|v| v["counters"][0]["created_on"] = "2026-02-30".into()),
            ),
            (
                "entry day",
                Box::new(|v| v["counters"][0]["entries"][0]["day"] = "01/02/26".into()),
            ),
            (
                "tap time",
                Box::new(|v| v["counters"][0]["events"][0]["at"] = "noon".into()),
            ),
            (
                "unknown field",
                Box::new(|v| v["counters"][0]["colour"] = "red".into()),
            ),
        ];
        for (what, edit) in bad {
            let text = with(|v| edit(v));
            assert!(parse(&text).is_err(), "a bad {what} was accepted");
        }
    }

    #[test]
    fn a_doubled_or_empty_day_is_refused() {
        let text = with(|v| {
            let entry = v["counters"][0]["entries"][0].clone();
            v["counters"][0]["entries"]
                .as_array_mut()
                .unwrap()
                .push(entry);
        });
        assert!(parse(&text).is_err(), "a day twice");
        let text = with(|v| v["counters"][0]["entries"][0]["count"] = 0.into());
        assert!(parse(&text).is_err(), "a zero day");
    }

    #[test]
    fn conflicts_name_each_shared_counter_once() {
        let mut backup = sample();
        let again = backup.counters[0].clone();
        backup.counters.push(again);
        let existing = vec![Counter {
            id: super::super::CounterId(1),
            name: CounterName::new("Push-ups").unwrap(),
            goal: None,
            step: Step::default(),
            big_step: None,
            celebration: None,
            created_on: d(2025, 1, 1),
        }];
        assert_eq!(
            backup.conflicts(&existing),
            vec![CounterName::new("Push-ups").unwrap()]
        );
        assert_eq!(backup.conflicts(&[]), Vec::<CounterName>::new());
    }
}
