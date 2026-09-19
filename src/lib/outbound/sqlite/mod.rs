//! The SQLite adapter: schema, migrations, and both store ports.
//!
//! All SQL in the crate lives in this file. Rows are mapped to domain types
//! here and nowhere else.

use crate::domain::counter::{
    Counter, CounterId, CounterName, CounterStore, DayCount, Goal, SettingsStore, StoreError,
};
use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension, params};
use std::{
    path::Path,
    sync::{Mutex, MutexGuard},
};
use zwipe_components::ThemeConfig;

/// Schema version written to SQLite's `user_version` pragma. Bump it and add
/// a step in `migrate` for each schema change.
const SCHEMA_VERSION: i64 = 1;

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

const THEME_KEY: &str = "theme";

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
    let goal: Option<i64> = row.get(2)?;
    let created: String = row.get(3)?;
    let goal = match goal {
        Some(g) => {
            let g =
                u32::try_from(g).map_err(|_| StoreError(format!("bad goal {g} in database")))?;
            Some(Goal::per_year(g).map_err(|e| StoreError(e.to_string()))?)
        }
        None => None,
    };
    Ok(Counter {
        id: CounterId(id),
        name: CounterName::new(&name).map_err(|e| StoreError(e.to_string()))?,
        goal,
        created_on: parse_day(&created)?,
    })
}

const COUNTER_COLS: &str = "id, name, goal_per_year, created_on";

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
        today: NaiveDate,
    ) -> Result<Counter, StoreError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO counters (name, goal_per_year, created_on) VALUES (?1, ?2, ?3)",
            params![
                name.as_str(),
                goal.map(Goal::yearly),
                today.format("%Y-%m-%d").to_string()
            ],
        )?;
        Ok(Counter {
            id: CounterId(conn.last_insert_rowid()),
            name: name.clone(),
            goal,
            created_on: today,
        })
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

    fn adjust(&self, id: CounterId, day: NaiveDate, delta: i64) -> Result<u32, StoreError> {
        let conn = self.conn()?;
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

impl SettingsStore for SqliteStore {
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

    fn store() -> SqliteStore {
        SqliteStore::in_memory().unwrap()
    }

    #[test]
    fn create_list_get_delete() {
        let s = store();
        let name = CounterName::new("pull-ups").unwrap();
        let c = s
            .create_counter(&name, Some(Goal::per_year(5000).unwrap()), d(2026, 9, 19))
            .unwrap();
        assert_eq!(s.list_counters().unwrap(), vec![c.clone()]);
        assert_eq!(s.get_counter(c.id).unwrap(), Some(c.clone()));
        s.delete_counter(c.id).unwrap();
        assert!(s.list_counters().unwrap().is_empty());
        assert_eq!(s.get_counter(c.id).unwrap(), None);
    }

    #[test]
    fn adjust_upserts_clamps_and_removes_zero_rows() {
        let s = store();
        let c = s
            .create_counter(&CounterName::new("x").unwrap(), None, d(2026, 1, 1))
            .unwrap();
        let today = d(2026, 1, 1);
        assert_eq!(s.adjust(c.id, today, 3).unwrap(), 3);
        assert_eq!(s.adjust(c.id, today, 2).unwrap(), 5);
        assert_eq!(s.adjust(c.id, today, -10).unwrap(), 0);
        assert!(s.entries(c.id).unwrap().is_empty());
        s.adjust(c.id, d(2026, 1, 2), 7).unwrap();
        s.adjust(c.id, d(2026, 1, 1), 1).unwrap();
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
            .create_counter(&CounterName::new("x").unwrap(), None, d(2026, 1, 1))
            .unwrap();
        s.adjust(c.id, d(2026, 1, 1), 1).unwrap();
        s.delete_counter(c.id).unwrap();
        assert!(s.entries(c.id).unwrap().is_empty());
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
