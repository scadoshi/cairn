//! The SQLite adapter: schema, migrations, and both store ports.
//!
//! All SQL in the crate lives in this file. Rows are mapped to domain types
//! here and nowhere else.

use crate::domain::{
    counter::{
        Counter, CounterId, CounterName, CounterStore, DayCount, Event, Goal, SettingsStore, Step,
        StoreError,
    },
    date_format::DateFormat,
    preferences::Preferences,
};
use chrono::{NaiveDate, NaiveDateTime};
use rusqlite::{Connection, OptionalExtension, params};
use std::{
    path::Path,
    sync::{Mutex, MutexGuard},
};
use zwipe_components::ThemeConfig;

/// Schema version written to SQLite's `user_version` pragma. Bump it and add
/// a step in `migrate` for each schema change.
const SCHEMA_VERSION: i64 = 6;

const SCHEMA_V1: &str = "
CREATE TABLE IF NOT EXISTS counters (
    id            INTEGER PRIMARY KEY,
    name          TEXT    NOT NULL,
    goal_per_year INTEGER,
    created_on    TEXT    NOT NULL
);
CREATE TABLE IF NOT EXISTS entries (
    counter_id INTEGER NOT NULL REFERENCES counters(id) ON DELETE CASCADE,
    day        TEXT    NOT NULL,
    count      INTEGER NOT NULL CHECK (count >= 0),
    PRIMARY KEY (counter_id, day)
);
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

/// v2: goals can be per day. Exactly one of goal_per_year / goal_per_day is
/// set, or neither.
const SCHEMA_V2: &str = "ALTER TABLE counters ADD COLUMN goal_per_day INTEGER;";

/// v3: every tap is kept with its local time, so hour-of-day stats exist.
/// Entries stay the source of truth for daily totals.
const SCHEMA_V3: &str = "
CREATE TABLE IF NOT EXISTS events (
    counter_id INTEGER NOT NULL REFERENCES counters(id) ON DELETE CASCADE,
    at         TEXT    NOT NULL,
    delta      INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS events_counter ON events(counter_id, at);
";

/// v4: how much one tap adds, per counter.
const SCHEMA_V4: &str = "ALTER TABLE counters ADD COLUMN step INTEGER NOT NULL DEFAULT 1;";

/// v5: goals can be per week. At most one of the three goal columns is set.
const SCHEMA_V5: &str = "ALTER TABLE counters ADD COLUMN goal_per_week INTEGER;";

/// v6: a counter can carry a second, larger step. Nullable, so a counter
/// without one keeps the three-button bar it had.
const SCHEMA_V6: &str = "ALTER TABLE counters ADD COLUMN big_step INTEGER;";

const THEME_KEY: &str = "theme";
const DATE_FORMAT_KEY: &str = "date_format";
const PREFS_KEY: &str = "prefs";

/// A connection behind a mutex. rusqlite is synchronous and every query here
/// is tiny, so the UI calls straight through.
pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        StoreError(e.to_string())
    }
}

impl SqliteStore {
    /// Opens (creating if needed) the database at `path` and migrates it.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    /// An in-memory store, for tests.
    pub fn in_memory() -> Result<Self, StoreError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self, StoreError> {
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn conn(&self) -> Result<MutexGuard<'_, Connection>, StoreError> {
        self.conn
            .lock()
            .map_err(|_| StoreError("database lock poisoned".to_string()))
    }
}

fn migrate(conn: &Connection) -> Result<(), StoreError> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version < 1 {
        conn.execute_batch(SCHEMA_V1)?;
    }
    if version < 2 {
        conn.execute_batch(SCHEMA_V2)?;
    }
    if version < 3 {
        conn.execute_batch(SCHEMA_V3)?;
    }
    if version < 4 {
        conn.execute_batch(SCHEMA_V4)?;
    }
    if version < 5 {
        conn.execute_batch(SCHEMA_V5)?;
    }
    if version < 6 {
        conn.execute_batch(SCHEMA_V6)?;
    }
    if version < SCHEMA_VERSION {
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    }
    Ok(())
}

fn parse_day(text: &str) -> Result<NaiveDate, StoreError> {
    NaiveDate::parse_from_str(text, "%Y-%m-%d")
        .map_err(|e| StoreError(format!("bad date {text:?} in database: {e}")))
}

fn row_to_counter(row: &rusqlite::Row<'_>) -> Result<Counter, StoreError> {
    let id: i64 = row.get(0)?;
    let name: String = row.get(1)?;
    let per_year: Option<i64> = row.get(2)?;
    let per_day: Option<i64> = row.get(3)?;
    let created: String = row.get(4)?;
    let step: i64 = row.get(5)?;
    let per_week: Option<i64> = row.get(6)?;
    let big_step: Option<i64> = row.get(7)?;
    let to_u32 =
        |g: i64| u32::try_from(g).map_err(|_| StoreError(format!("bad goal {g} in database")));
    let bad = |e: super::super::domain::counter::ValidationError| StoreError(e.to_string());
    let goal = match (per_year, per_week, per_day) {
        (Some(g), _, _) => Some(Goal::per_year(to_u32(g)?).map_err(bad)?),
        (None, Some(g), _) => Some(Goal::per_week(to_u32(g)?).map_err(bad)?),
        (None, None, Some(g)) => Some(Goal::per_day(to_u32(g)?).map_err(bad)?),
        (None, None, None) => None,
    };
    Ok(Counter {
        id: CounterId(id),
        name: CounterName::new(&name).map_err(|e| StoreError(e.to_string()))?,
        goal,
        step: Step::new(to_u32(step)?).map_err(|e| StoreError(e.to_string()))?,
        big_step: big_step
            .map(|b| Step::new(to_u32(b)?).map_err(|e| StoreError(e.to_string())))
            .transpose()?,
        created_on: parse_day(&created)?,
    })
}

/// The nullable goal columns a goal maps to: (per year, per day, per week).
fn goal_columns(goal: Option<Goal>) -> (Option<u32>, Option<u32>, Option<u32>) {
    match goal {
        Some(Goal::PerYear(n)) => (Some(n), None, None),
        Some(Goal::PerDay(n)) => (None, Some(n), None),
        Some(Goal::PerWeek(n)) => (None, None, Some(n)),
        None => (None, None, None),
    }
}

const COUNTER_COLS: &str =
    "id, name, goal_per_year, goal_per_day, created_on, step, goal_per_week, big_step";

impl CounterStore for SqliteStore {
    fn list_counters(&self) -> Result<Vec<Counter>, StoreError> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(&format!("SELECT {COUNTER_COLS} FROM counters ORDER BY id"))?;
        let rows = stmt.query_map([], |r| Ok(row_to_counter(r)))?;
        rows.map(|r| r?).collect()
    }

    fn get_counter(&self, id: CounterId) -> Result<Option<Counter>, StoreError> {
        let conn = self.conn()?;
        conn.query_row(
            &format!("SELECT {COUNTER_COLS} FROM counters WHERE id = ?1"),
            params![id.0],
            |r| Ok(row_to_counter(r)),
        )
        .optional()?
        .transpose()
    }

    fn create_counter(
        &self,
        name: &CounterName,
        goal: Option<Goal>,
        step: Step,
        big_step: Option<Step>,
        today: NaiveDate,
    ) -> Result<Counter, StoreError> {
        let conn = self.conn()?;
        let (per_year, per_day, per_week) = goal_columns(goal);
        conn.execute(
            "INSERT INTO counters (name, goal_per_year, goal_per_day, goal_per_week, created_on, step, big_step)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                name.as_str(),
                per_year,
                per_day,
                per_week,
                today.format("%Y-%m-%d").to_string(),
                step.get(),
                big_step.map(Step::get)
            ],
        )?;
        Ok(Counter {
            id: CounterId(conn.last_insert_rowid()),
            name: name.clone(),
            goal,
            step,
            big_step,
            created_on: today,
        })
    }

    fn update_counter(
        &self,
        id: CounterId,
        name: &CounterName,
        goal: Option<Goal>,
        step: Step,
        big_step: Option<Step>,
    ) -> Result<(), StoreError> {
        let (per_year, per_day, per_week) = goal_columns(goal);
        self.conn()?.execute(
            "UPDATE counters
             SET name = ?1, goal_per_year = ?2, goal_per_day = ?3, goal_per_week = ?4, step = ?5,
                 big_step = ?6
             WHERE id = ?7",
            params![
                name.as_str(),
                per_year,
                per_day,
                per_week,
                step.get(),
                big_step.map(Step::get),
                id.0
            ],
        )?;
        Ok(())
    }

    fn delete_counter(&self, id: CounterId) -> Result<(), StoreError> {
        self.conn()?
            .execute("DELETE FROM counters WHERE id = ?1", params![id.0])?;
        Ok(())
    }

    fn entries(&self, id: CounterId) -> Result<Vec<DayCount>, StoreError> {
        let conn = self.conn()?;
        let mut stmt =
            conn.prepare("SELECT day, count FROM entries WHERE counter_id = ?1 ORDER BY day")?;
        let rows = stmt.query_map(params![id.0], |r| {
            let day: String = r.get(0)?;
            let count: i64 = r.get(1)?;
            Ok((day, count))
        })?;
        rows.map(|r| {
            let (day, count) = r?;
            Ok(DayCount {
                day: parse_day(&day)?,
                count: u32::try_from(count)
                    .map_err(|_| StoreError(format!("bad count {count} in database")))?,
            })
        })
        .collect()
    }

    fn rebuild_entries(&self, prefs: &Preferences) -> Result<(), StoreError> {
        self.rebuild(prefs)
    }

    fn events(&self, id: CounterId) -> Result<Vec<Event>, StoreError> {
        let conn = self.conn()?;
        let mut stmt =
            conn.prepare("SELECT at, delta FROM events WHERE counter_id = ?1 ORDER BY at")?;
        let rows = stmt.query_map(params![id.0], |r| {
            let at: String = r.get(0)?;
            let delta: i64 = r.get(1)?;
            Ok((at, delta))
        })?;
        rows.map(|r| {
            let (at, delta) = r?;
            let at = NaiveDateTime::parse_from_str(&at, "%Y-%m-%dT%H:%M:%S")
                .map_err(|e| StoreError(format!("bad time {at:?} in database: {e}")))?;
            Ok(Event { at, delta })
        })
        .collect()
    }

    fn adjust(
        &self,
        id: CounterId,
        at: NaiveDateTime,
        day: NaiveDate,
        delta: i64,
    ) -> Result<u32, StoreError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO events (counter_id, at, delta) VALUES (?1, ?2, ?3)",
            params![id.0, at.format("%Y-%m-%dT%H:%M:%S").to_string(), delta],
        )?;
        let day = day.format("%Y-%m-%d").to_string();
        let current: i64 = conn
            .query_row(
                "SELECT count FROM entries WHERE counter_id = ?1 AND day = ?2",
                params![id.0, day],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or(0);
        let next = current.saturating_add(delta).max(0);
        if next == 0 {
            conn.execute(
                "DELETE FROM entries WHERE counter_id = ?1 AND day = ?2",
                params![id.0, day],
            )?;
        } else {
            conn.execute(
                "INSERT INTO entries (counter_id, day, count) VALUES (?1, ?2, ?3)
                 ON CONFLICT (counter_id, day) DO UPDATE SET count = excluded.count",
                params![id.0, day, next],
            )?;
        }
        Ok(u32::try_from(next).unwrap_or(u32::MAX))
    }
}

impl SqliteStore {
    fn setting(&self, key: &str) -> Result<Option<String>, StoreError> {
        Ok(self
            .conn()?
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |r| r.get(0),
            )
            .optional()?)
    }

    fn set_setting(&self, key: &str, value: &str) -> Result<(), StoreError> {
        self.conn()?.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}

impl SqliteStore {
    /// Every counter's entries, re-derived from its events under `prefs`.
    fn rebuild(&self, prefs: &Preferences) -> Result<(), StoreError> {
        let mut conn = self.conn()?;
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM entries", [])?;
        let mut totals: std::collections::BTreeMap<(i64, NaiveDate), i64> =
            std::collections::BTreeMap::new();
        {
            let mut stmt = tx.prepare("SELECT counter_id, at, delta FROM events")?;
            let rows = stmt.query_map([], |r| {
                let id: i64 = r.get(0)?;
                let at: String = r.get(1)?;
                let delta: i64 = r.get(2)?;
                Ok((id, at, delta))
            })?;
            for row in rows {
                let (id, at, delta) = row?;
                let at = NaiveDateTime::parse_from_str(&at, "%Y-%m-%dT%H:%M:%S")
                    .map_err(|e| StoreError(format!("bad time {at:?} in database: {e}")))?;
                *totals.entry((id, prefs.day_of(at))).or_default() += delta;
            }
        }
        for ((id, day), total) in totals {
            if total > 0 {
                tx.execute(
                    "INSERT INTO entries (counter_id, day, count) VALUES (?1, ?2, ?3)",
                    params![id, day.format("%Y-%m-%d").to_string(), total],
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}

impl SettingsStore for SqliteStore {
    fn preferences(&self) -> Result<Preferences, StoreError> {
        Ok(self
            .setting(PREFS_KEY)?
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default())
    }

    fn set_preferences(&self, prefs: &Preferences) -> Result<(), StoreError> {
        let json = serde_json::to_string(prefs).map_err(|e| StoreError(e.to_string()))?;
        self.set_setting(PREFS_KEY, &json)
    }

    fn date_format(&self) -> Result<DateFormat, StoreError> {
        Ok(self
            .setting(DATE_FORMAT_KEY)?
            .map_or(DateFormat::default(), |k| DateFormat::from_key(&k)))
    }

    fn set_date_format(&self, format: DateFormat) -> Result<(), StoreError> {
        self.set_setting(DATE_FORMAT_KEY, format.key())
    }

    fn theme(&self) -> Result<Option<ThemeConfig>, StoreError> {
        let conn = self.conn()?;
        let raw: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![THEME_KEY],
                |r| r.get(0),
            )
            .optional()?;
        // A theme that no longer parses (an old shape, say) is just ignored;
        // the default takes over and the next pick overwrites it.
        Ok(raw.and_then(|s| serde_json::from_str(&s).ok()))
    }

    fn set_theme(&self, theme: &ThemeConfig) -> Result<(), StoreError> {
        let json = serde_json::to_string(theme).map_err(|e| StoreError(e.to_string()))?;
        self.conn()?.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
            params![THEME_KEY, json],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn at(y: i32, m: u32, day: u32) -> NaiveDateTime {
        d(y, m, day).and_hms_opt(7, 30, 0).unwrap()
    }

    fn store() -> SqliteStore {
        SqliteStore::in_memory().unwrap()
    }

    #[test]
    fn create_list_get_delete() {
        let s = store();
        let name = CounterName::new("pull-ups").unwrap();
        let c = s
            .create_counter(
                &name,
                Some(Goal::per_year(5000).unwrap()),
                Step::default(),
                None,
                d(2026, 9, 19),
            )
            .unwrap();
        assert_eq!(s.list_counters().unwrap(), vec![c.clone()]);
        assert_eq!(s.get_counter(c.id).unwrap(), Some(c.clone()));
        s.delete_counter(c.id).unwrap();
        assert!(s.list_counters().unwrap().is_empty());
        assert_eq!(s.get_counter(c.id).unwrap(), None);
    }

    #[test]
    fn update_persists_name_goal_and_step() {
        let s = store();
        let c = s
            .create_counter(
                &CounterName::new("x").unwrap(),
                None,
                Step::default(),
                None,
                d(2026, 1, 1),
            )
            .unwrap();
        s.update_counter(
            c.id,
            &CounterName::new("pull-ups").unwrap(),
            Some(Goal::per_day(20).unwrap()),
            Step::new(25).unwrap(),
            Step::new(50).ok(),
        )
        .unwrap();
        let got = s.get_counter(c.id).unwrap().unwrap();
        assert_eq!(got.name.as_str(), "pull-ups");
        assert_eq!(got.goal, Some(Goal::PerDay(20)));
        assert_eq!(got.step.get(), 25);
        assert_eq!(got.big_step, Step::new(50).ok(), "the big step round trips");
        s.update_counter(
            c.id,
            &got.name,
            Some(Goal::per_week(100).unwrap()),
            got.step,
            got.big_step,
        )
        .unwrap();
        assert_eq!(
            s.get_counter(c.id).unwrap().unwrap().goal,
            Some(Goal::PerWeek(100))
        );
        s.update_counter(c.id, &got.name, None, got.step, None)
            .unwrap();
        assert_eq!(s.get_counter(c.id).unwrap().unwrap().goal, None);
    }

    #[test]
    fn per_day_goal_round_trips() {
        let s = store();
        let c = s
            .create_counter(
                &CounterName::new("push-ups").unwrap(),
                Some(Goal::per_day(15).unwrap()),
                Step::default(),
                None,
                d(2026, 9, 19),
            )
            .unwrap();
        assert_eq!(
            s.get_counter(c.id).unwrap().unwrap().goal,
            Some(Goal::PerDay(15))
        );
    }

    #[test]
    fn adjust_upserts_clamps_and_removes_zero_rows() {
        let s = store();
        let c = s
            .create_counter(
                &CounterName::new("x").unwrap(),
                None,
                Step::default(),
                None,
                d(2026, 1, 1),
            )
            .unwrap();
        let today = at(2026, 1, 1);
        assert_eq!(s.adjust(c.id, today, today.date(), 3).unwrap(), 3);
        assert_eq!(s.adjust(c.id, today, today.date(), 2).unwrap(), 5);
        assert_eq!(s.adjust(c.id, today, today.date(), -10).unwrap(), 0);
        assert_eq!(s.events(c.id).unwrap().len(), 3);
        assert!(s.entries(c.id).unwrap().is_empty());
        s.adjust(c.id, at(2026, 1, 2), d(2026, 1, 2), 7).unwrap();
        s.adjust(c.id, at(2026, 1, 1), d(2026, 1, 1), 1).unwrap();
        assert_eq!(
            s.entries(c.id).unwrap(),
            vec![
                DayCount {
                    day: d(2026, 1, 1),
                    count: 1
                },
                DayCount {
                    day: d(2026, 1, 2),
                    count: 7
                },
            ]
        );
    }

    #[test]
    fn deleting_counter_cascades_entries() {
        let s = store();
        let c = s
            .create_counter(
                &CounterName::new("x").unwrap(),
                None,
                Step::default(),
                None,
                d(2026, 1, 1),
            )
            .unwrap();
        s.adjust(c.id, at(2026, 1, 1), d(2026, 1, 1), 1).unwrap();
        s.delete_counter(c.id).unwrap();
        assert!(s.entries(c.id).unwrap().is_empty());
        assert!(s.events(c.id).unwrap().is_empty());
    }

    #[test]
    fn theme_round_trips() {
        let s = store();
        assert!(s.theme().unwrap().is_none());
        let t = ThemeConfig {
            name: "dracula".to_string(),
            is_dark: false,
        };
        s.set_theme(&t).unwrap();
        // ThemeConfig has no Debug impl, so compare field by field.
        let got = s.theme().unwrap().unwrap();
        assert_eq!(got.name, "dracula");
        assert!(!got.is_dark);
    }

    #[test]
    fn date_format_round_trips_and_defaults() {
        let s = store();
        assert_eq!(s.date_format().unwrap(), DateFormat::MonthDayYear);
        s.set_date_format(DateFormat::DayMonthYear).unwrap();
        assert_eq!(s.date_format().unwrap(), DateFormat::DayMonthYear);
    }

    #[test]
    fn preferences_round_trip_and_default() {
        let s = store();
        assert_eq!(s.preferences().unwrap(), Preferences::default());
        let p = Preferences {
            rollover_hour: 4,
            ..Preferences::default()
        };
        s.set_preferences(&p).unwrap();
        assert_eq!(s.preferences().unwrap(), p);
    }

    #[test]
    fn rebuild_reassigns_late_taps_by_rollover() {
        let s = store();
        let c = s
            .create_counter(
                &CounterName::new("x").unwrap(),
                None,
                Step::default(),
                None,
                d(2026, 1, 1),
            )
            .unwrap();
        let late = d(2026, 1, 2).and_hms_opt(0, 30, 0).unwrap();
        s.adjust(c.id, late, d(2026, 1, 2), 5).unwrap();
        assert_eq!(s.entries(c.id).unwrap()[0].day, d(2026, 1, 2));
        let p = Preferences {
            rollover_hour: 4,
            ..Preferences::default()
        };
        s.rebuild_entries(&p).unwrap();
        let entries = s.entries(c.id).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].day, d(2026, 1, 1));
        assert_eq!(entries[0].count, 5);
    }

    /// Build a database at an older schema version, the way an installed app
    /// would have left it, then open it and see whether the upgrade lands.
    fn aged_db(dir: &std::path::Path, version: i64, steps: &[&str]) -> std::path::PathBuf {
        let path = dir.join("aged.db");
        let conn = Connection::open(&path).unwrap();
        for s in steps {
            conn.execute_batch(s).unwrap();
        }
        conn.execute(
            "INSERT INTO counters (id, name, goal_per_year, created_on) \
             VALUES (1, 'pushups', 3650, '2026-01-01')",
            [],
        )
        .unwrap();
        conn.pragma_update(None, "user_version", version).unwrap();
        drop(conn);
        path
    }

    /// Every other store test starts from `in_memory()`, which is version 0,
    /// so all five rungs always run and the version guards are never
    /// exercised. That leaves the path a real phone takes, an existing
    /// database being upgraded, with no coverage at all: changing
    /// `if version < 5` to `if version < 4` used to pass the whole suite
    /// while breaking every install that already had data.
    #[test]
    fn opens_a_database_left_at_every_older_version() {
        let ladder = [
            SCHEMA_V1, SCHEMA_V2, SCHEMA_V3, SCHEMA_V4, SCHEMA_V5, SCHEMA_V6,
        ];
        for version in 1..=SCHEMA_VERSION {
            let dir = std::env::temp_dir().join(format!("cairn-aged-{version}"));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            let steps: Vec<&str> = ladder
                .iter()
                .take(usize::try_from(version).unwrap())
                .copied()
                .collect();
            let path = aged_db(&dir, version, &steps);

            let s = SqliteStore::open(&path)
                .unwrap_or_else(|e| panic!("v{version} database would not open: {e}"));
            let list = s
                .list_counters()
                .unwrap_or_else(|e| panic!("v{version} database would not read: {e}"));
            assert_eq!(list.len(), 1, "v{version} lost the existing counter");
            assert_eq!(list[0].name.as_str(), "pushups");
            let _ = std::fs::remove_dir_all(&dir);
        }
    }

    #[test]
    fn migrate_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
        let v: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, SCHEMA_VERSION);
    }
}
