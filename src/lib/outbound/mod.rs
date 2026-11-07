//! Adapters to the outside world: the SQLite store, platform paths and the
//! home-screen icon.

#[cfg(target_os = "android")]
pub mod android_fs;
pub mod app_icon;
pub mod paths;
pub mod sqlite;
