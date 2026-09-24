//! What the app needs from storage, written from the app's side. The SQLite
//! adapter in `outbound` implements these; a watch build would add another.

use super::{Counter, CounterId, CounterName, DayCount, Event, Goal, Step};
use crate::domain::{date_format::DateFormat, preferences::Preferences};
use chrono::{NaiveDate, NaiveDateTime};
use thiserror::Error;
use zwipe_components::ThemeConfig;

/// Storage failure, already reduced to a message. Screens show it; they never
/// branch on it.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{0}")]
pub struct StoreError(pub String);

/// Counters and their daily entries.
pub trait CounterStore {
    /// Every counter, oldest first.
    fn list_counters(&self) -> Result<Vec<Counter>, StoreError>;
    /// One counter, or `None` if the id is unknown.
    fn get_counter(&self, id: CounterId) -> Result<Option<Counter>, StoreError>;
    /// Creates a counter starting today.
    fn create_counter(
        &self,
        name: &CounterName,
        goal: Option<Goal>,
        step: Step,
        big_step: Option<Step>,
        today: NaiveDate,
    ) -> Result<Counter, StoreError>;
    /// Changes a counter's name, goal, and step.
    fn update_counter(
        &self,
        id: CounterId,
        name: &CounterName,
        goal: Option<Goal>,
        step: Step,
        big_step: Option<Step>,
    ) -> Result<(), StoreError>;
    /// Deletes the counter and every entry under it.
    fn delete_counter(&self, id: CounterId) -> Result<(), StoreError>;
    /// Every entry for a counter, oldest first. Days with no row are absent.
    fn entries(&self, id: CounterId) -> Result<Vec<DayCount>, StoreError>;
    /// Every tap for a counter, oldest first.
    fn events(&self, id: CounterId) -> Result<Vec<Event>, StoreError>;
    /// Records a tap at `at` and adds `delta` to `day`'s total, clamped at
    /// zero. `day` is the caller's call (the rollover preference decides
    /// which day a late tap belongs to). Returns the new total. Creates the
    /// day row on first touch and removes it when it hits zero.
    fn adjust(
        &self,
        id: CounterId,
        at: NaiveDateTime,
        day: NaiveDate,
        delta: i64,
    ) -> Result<u32, StoreError>;
    /// Rebuilds every counter's daily entries from its events, assigning each
    /// tap to a day by `prefs`. Called when the rollover hour changes.
    fn rebuild_entries(&self, prefs: &Preferences) -> Result<(), StoreError>;
}

/// App-wide preferences.
pub trait SettingsStore {
    /// The saved theme, or `None` on first launch.
    fn theme(&self) -> Result<Option<ThemeConfig>, StoreError>;
    /// Persists the theme.
    fn set_theme(&self, theme: &ThemeConfig) -> Result<(), StoreError>;
    /// The saved date format, or the default.
    fn date_format(&self) -> Result<DateFormat, StoreError>;
    /// Persists the date format.
    fn set_date_format(&self, format: DateFormat) -> Result<(), StoreError>;
    /// The saved preferences, or the defaults.
    fn preferences(&self) -> Result<Preferences, StoreError>;
    /// Persists the preferences.
    fn set_preferences(&self, prefs: &Preferences) -> Result<(), StoreError>;
}

/// Both ports behind one object, which is what the UI holds in context.
pub trait Store: CounterStore + SettingsStore {}

impl<T: CounterStore + SettingsStore> Store for T {}
