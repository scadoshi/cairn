//! What the app needs from storage, written from the app's side. The SQLite
//! adapter in `outbound` implements these; a watch build would add another.

use super::{Counter, CounterId, CounterName, DayCount, Event, Goal, Step};
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
        today: NaiveDate,
    ) -> Result<Counter, StoreError>;
    /// Changes a counter's name, goal, and step.
    fn update_counter(
        &self,
        id: CounterId,
        name: &CounterName,
        goal: Option<Goal>,
        step: Step,
    ) -> Result<(), StoreError>;
    /// Deletes the counter and every entry under it.
    fn delete_counter(&self, id: CounterId) -> Result<(), StoreError>;
    /// Every entry for a counter, oldest first. Days with no row are absent.
    fn entries(&self, id: CounterId) -> Result<Vec<DayCount>, StoreError>;
    /// Every tap for a counter, oldest first.
    fn events(&self, id: CounterId) -> Result<Vec<Event>, StoreError>;
    /// Records a tap at `at` and adds `delta` to that day's total, clamped at
    /// zero. Returns the new total. Creates the day row on first touch and
    /// removes it when it hits zero.
    fn adjust(&self, id: CounterId, at: NaiveDateTime, delta: i64) -> Result<u32, StoreError>;
}

/// App-wide preferences.
pub trait SettingsStore {
    /// The saved theme, or `None` on first launch.
    fn theme(&self) -> Result<Option<ThemeConfig>, StoreError>;
    /// Persists the theme.
    fn set_theme(&self, theme: &ThemeConfig) -> Result<(), StoreError>;
}

/// Both ports behind one object, which is what the UI holds in context.
pub trait Store: CounterStore + SettingsStore {}

impl<T: CounterStore + SettingsStore> Store for T {}
