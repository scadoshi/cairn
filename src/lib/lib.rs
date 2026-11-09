//! Cairn: a lifetime counter with per-day entries, yearly goals, and CSV export.
//!
//! Layout is hexagonal inside one crate. `domain` is pure and portable (the
//! part a future watch build reuses), `screens` decides what each screen
//! does with it (pure too, no Dioxus or SQLite), `inbound::ui` is the Dioxus
//! app, and `outbound` holds the SQLite adapter and platform paths.

#![warn(missing_docs)]

pub mod domain;
pub mod inbound;
pub mod outbound;
pub mod screens;
